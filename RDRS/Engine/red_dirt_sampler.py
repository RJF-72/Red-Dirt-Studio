import numpy as np
import soundfile as sf
import mido
from pathlib import Path
import math
import re

# =========================
# SFZ PARSER (BASIC SUBSET)
# =========================

NOTE_NAME_TO_SEMITONE = {
    "c": 0, "c#": 1, "db": 1, "d": 2, "d#": 3, "eb": 3, "e": 4, "f": 5,
    "f#": 6, "gb": 6, "g": 7, "g#": 8, "ab": 8, "a": 9, "a#": 10, "bb": 10, "b": 11,
}

def note_name_to_midi(note_str):
    match = re.match(r"([a-g]#?)(\d+)", note_str.lower())
    if not match:
        return 60  # default to middle C
    semitone, octave = match.groups()
    return NOTE_NAME_TO_SEMITONE.get(semitone, 0) + (int(octave) + 1) * 12

class SFZRegion:
    def __init__(self, params, base_dir: Path):
        self.params = params
        self.sample_rel = params.get("sample")
        self.sample_path = (base_dir / self.sample_rel).resolve() if self.sample_rel else None

        self.lokey = self._parse_key(params.get("lokey", 0))
        self.hikey = self._parse_key(params.get("hikey", 127))
        self.lovel = int(params.get("lovel", 0))
        self.hivel = int(params.get("hivel", 127))

        self.pitch_keycenter = self._parse_key(params.get("pitch_keycenter", 60))
        self.offset = int(params.get("offset", 0))

        self.loop_mode = params.get("loop_mode", "no_loop")

        self.ampeg_attack = float(params.get("ampeg_attack", 0.001))
        self.ampeg_decay = float(params.get("ampeg_decay", 0.1))
        self.ampeg_sustain = float(params.get("ampeg_sustain", 1.0))
        self.ampeg_release = float(params.get("ampeg_release", 0.2))

        self.seq_position = int(params.get("seq_position", 1))

    def _parse_key(self, value):
        if isinstance(value, int):
            return value
        try:
            return int(value)
        except (ValueError, TypeError):
            return note_name_to_midi(str(value))

def parse_sfz(sfz_path: Path):
    base_dir = sfz_path.parent
    regions = []
    current = {}
    in_region = False

    for line in sfz_path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = line.strip()
        if not line or line.startswith("//"):
            continue

        if line.startswith("<region>"):
            if current and in_region:
                regions.append(SFZRegion(current, base_dir))
            current = {}
            in_region = True
            continue

        if line.startswith("<"):
            if current and in_region:
                regions.append(SFZRegion(current, base_dir))
            current = {}
            in_region = False
            continue

        if in_region and "=" in line:
            # Handle multiple key=value pairs on one line (space-separated)
            for pair in line.split():
                if "=" in pair:
                    k, v = pair.split("=", 1)
                    # Strip inline comments
                    v = v.split("//")[0].strip()
                    if v:
                        current[k.strip()] = v

    if current and in_region:
        regions.append(SFZRegion(current, base_dir))

    return regions

# =========================
# SAMPLE LOADING
# =========================

def load_flac(path: Path):
    audio, sr = sf.read(str(path))
    if audio.ndim == 1:
        audio = np.stack([audio, audio], axis=1)
    return audio.astype(np.float32), sr

# =========================
# VOICE WITH ADSR
# =========================

class Voice:
    def __init__(self, region: SFZRegion, sample: np.ndarray, sr: int,
                 note: int, velocity: int, start_frame: int):
        self.region = region
        self.sample = sample
        self.sr = sr
        self.note = note
        self.velocity = velocity
        self.start_frame = start_frame

        semitones = note - region.pitch_keycenter
        self.pitch_ratio = 2.0 ** (semitones / 12.0)

        self.play_pos = region.offset
        self.active = True
        self.released = False
        self.release_frame = None

        self.env_attack = int(region.ampeg_attack * sr)
        self.env_decay = int(region.ampeg_decay * sr)
        self.env_release = int(region.ampeg_release * sr)
        self.env_sustain = region.ampeg_sustain

        self.env_level = 0.0
        self.env_phase = "attack"
        self.env_pos = 0

        self.vel_gain = velocity / 127.0

    def note_off(self, frame_idx: int):
        if not self.released:
            self.released = True
            self.release_frame = frame_idx
            self.env_phase = "release"
            self.env_pos = 0

    def _env_step(self, n_frames: int):
        env = np.zeros(n_frames, dtype=np.float32)
        for i in range(n_frames):
            if self.env_phase == "attack":
                if self.env_attack > 0:
                    self.env_level = min(1.0, self.env_pos / self.env_attack)
                else:
                    self.env_level = 1.0
                self.env_pos += 1
                if self.env_pos >= self.env_attack:
                    self.env_phase = "decay"
                    self.env_pos = 0

            elif self.env_phase == "decay":
                if self.env_decay > 0:
                    t = self.env_pos / self.env_decay
                    self.env_level = 1.0 + (self.env_sustain - 1.0) * t
                else:
                    self.env_level = self.env_sustain
                self.env_pos += 1
                if self.env_pos >= self.env_decay:
                    self.env_phase = "sustain"
                    self.env_level = self.env_sustain

            elif self.env_phase == "sustain":
                self.env_level = self.env_sustain

            elif self.env_phase == "release":
                if self.env_release > 0:
                    t = self.env_pos / self.env_release
                    self.env_level = max(0.0, self.env_level * (1.0 - t))
                else:
                    self.env_level = 0.0
                self.env_pos += 1
                if self.env_pos >= self.env_release or self.env_level <= 1e-4:
                    self.env_level = 0.0
                    self.active = False

            env[i] = self.env_level

        return env

    def render(self, global_frame: int, n_frames: int):
        if not self.active:
            return np.zeros((n_frames, 2), dtype=np.float32)

        rel_start = max(0, global_frame - self.start_frame)
        if rel_start >= n_frames:
            return np.zeros((n_frames, 2), dtype=np.float32)

        frames_to_render = n_frames - rel_start

        idx = self.play_pos + np.arange(frames_to_render) * self.pitch_ratio
        idx = idx.astype(np.int64)

        valid = idx < len(self.sample)
        if not np.any(valid):
            self.active = False
            return np.zeros((n_frames, 2), dtype=np.float32)

        idx = idx[valid]
        audio = self.sample[idx] * self.vel_gain

        env = self._env_step(len(audio))
        audio *= env[:, None]

        out = np.zeros((n_frames, 2), dtype=np.float32)
        out[rel_start:rel_start + len(audio)] += audio

        self.play_pos += int(frames_to_render * self.pitch_ratio)
        if self.play_pos >= len(self.sample) and self.env_phase != "release":
            self.env_phase = "release"
            self.env_pos = 0

        if not self.active:
            pass

        return out

# =========================
# REGION SELECTION (KEY/VEL/RR)
# =========================

def select_region(regions, note, velocity, rr_counter):
    candidates = []
    for r in regions:
        if r.lokey <= note <= r.hikey and r.lovel <= velocity <= r.hivel:
            candidates.append(r)
    if not candidates:
        return None

    rr = (rr_counter % len(candidates)) + 1
    for r in candidates:
        if r.seq_position == rr:
            return r
    return candidates[0]

# =========================
# MIDI → VOICES
# =========================

def build_voices_from_midi(sfz_regions, sfz_samples, midi_path: Path, sr: int):
    mid = mido.MidiFile(str(midi_path))
    tempo = 500000  # default 120 bpm
    ticks_per_beat = mid.ticks_per_beat

    def ticks_to_frames(ticks):
        beats = ticks / ticks_per_beat
        seconds = beats * (tempo / 1_000_000.0)
        return int(seconds * sr)

    voices = []
    rr_counter = 0
    current_time_ticks = 0
    note_on_frames = {}

    for msg in mid:
        current_time_ticks += msg.time
        if msg.type == "set_tempo":
            tempo = msg.tempo
        elif msg.type == "note_on" and msg.velocity > 0:
            frame = ticks_to_frames(current_time_ticks)
            rr_counter += 1
            region = select_region(sfz_regions, msg.note, msg.velocity, rr_counter)
            if region is None or region.sample_path not in sfz_samples:
                continue
            sample_audio = sfz_samples[region.sample_path]
            v = Voice(region, sample_audio, sr, msg.note, msg.velocity, frame)
            voices.append(v)
            note_on_frames[(msg.channel, msg.note)] = v
        elif (msg.type == "note_off") or (msg.type == "note_on" and msg.velocity == 0):
            key = (msg.channel, msg.note)
            if key in note_on_frames:
                v = note_on_frames[key]
                frame = ticks_to_frames(current_time_ticks)
                v.note_off(frame)
                del note_on_frames[key]

    return voices

# =========================
# RENDER ENGINE
# =========================

def render_voices_to_wav(voices, sr: int, out_path: Path, tail_seconds: float = 2.0):
    if not voices:
        sf.write(str(out_path), np.zeros((1, 2), dtype=np.float32), sr)
        return

    max_frame = max(v.start_frame for v in voices) + int(sr * 10)
    total_frames = max_frame + int(tail_seconds * sr)

    block = 1024
    out = np.zeros((total_frames, 2), dtype=np.float32)

    frame = 0
    while frame < total_frames:
        n = min(block, total_frames - frame)
        block_mix = np.zeros((n, 2), dtype=np.float32)
        for v in voices:
            if v.active or frame < v.start_frame + n:
                block_mix += v.render(frame, n)
        out[frame:frame + n] += block_mix
        frame += n

    peak = np.max(np.abs(out))
    if peak > 0.99:
        out /= peak / 0.99

    sf.write(str(out_path), out, sr)

# =========================
# HIGH-LEVEL ENTRY POINT
# =========================

def render_sfz_midi_to_wav(sfz_path: Path, midi_path: Path, wav_path: Path):
    sfz_regions = parse_sfz(sfz_path)

    sfz_samples = {}
    sr = None
    for r in sfz_regions:
        if r.sample_path and r.sample_path not in sfz_samples:
            audio, srate = load_flac(r.sample_path)
            sfz_samples[r.sample_path] = audio
            if sr is None:
                sr = srate
            elif sr != srate:
                raise ValueError("Sample rate mismatch across regions")

    if sr is None:
        sr = 44100

    voices = build_voices_from_midi(sfz_regions, sfz_samples, midi_path, sr)
    render_voices_to_wav(voices, sr, wav_path)

# =========================
# EXAMPLE: YOUR FINGER-PICKED ACOUSTIC
# =========================

if __name__ == "__main__":
    sfz_file = Path(r"E:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Guitar\Finger Picked Acoustic.sfz")
    midi_file = Path(r"E:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Models\Guitar\test_fingerpicked.mid")
    wav_file = Path(r"E:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Models\Guitar\test_fingerpicked.wav")

    render_sfz_midi_to_wav(sfz_file, midi_file, wav_file)
    print("Rendered:", wav_file)
