import subprocess
import re
import mido
import sys
import shutil
from pathlib import Path

from red_dirt_sampler import render_sfz_midi_to_wav

SONG_TITLE = "Dust_on_the_Dashboard"

SONG_LYRICS = (
    "There’s dust on the dashboard, coffee in the seat\n"
    "A map of old mistakes lyin’ crumpled at my feet\n"
    "Sun’s barely risin’, but it’s burnin’ through the haze\n"
    "Feels like every mile remembers better days\n"
)

SONG_TEMPO_BPM = 96
SONG_KEY = "KEY_E"

SONG_SECTIONS = [
    ("SECTION_INTRO", 4),
    ("SECTION_VERSE", 16),
    ("SECTION_CHORUS", 16),
    ("SECTION_VERSE", 16),
    ("SECTION_CHORUS", 16),
    ("SECTION_BRIDGE", 8),
    ("SECTION_CHORUS", 16),
    ("SECTION_OUTRO", 8),
]

ALL_INSTRUMENTS = {
    "Acoustic_Guitar": "Acoustic_Campfire",
    "Resonator_Guitar": "Resonator_Slide",
    "Acoustic_Bass": "Acoustic_Bass",
    "Electric_Guitar": "Texas_Lead",
    "Electric_Bass": "Electric_Bass",
    "Fiddle": "Fiddle",
    "Mandolin": "Mandolin",
    "Harmonica": "Harmonica",
    "Piano": "Piano",
    "Keyboard": "Pad_Keys",
}

SFZ_MAP = {
    "Acoustic_Guitar": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Guitar\RedDirtTrak_Guitar_Guitar_Acoustic_Campfire.sfz",
    "Resonator_Guitar": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Guitar\RedDirtTrak_Guitar_Resonator_Guitar_Americana.sfz",
    "Acoustic_Bass": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Bass\RedDirtTrak_Bass_Bass_Acoustic_Americana.sfz",
    "Electric_Guitar": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Guitar\RedDirtTrak_Guitar_Electric_Clean_Detroit_Funk.sfz",
    "Electric_Bass": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Bass\RedDirtTrak_Bass_Electric_Nashville_Radio.sfz",
    "Fiddle": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Fiddle\RedDirtTrak_Fiddle_Fiddle_Americana.sfz",
    "Mandolin": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Guitar\RedDirtTrak_Guitar_Mandolin_Ballad.sfz",
    "Harmonica": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Harmonica\RedDirtTrak_Harmonica_Harmonica_Blues.sfz",
    "Piano": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Piano\Acoustic Grand Piano.sfz",
    "Keyboard": r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Instruments\Piano\Jazz Organ.sfz",
}

SELECTED_INSTRUMENTS = list(ALL_INSTRUMENTS.keys())

TPB = 480
MS_TO_TICKS = 0.48

def run_model(style, section_type, emotion):
    cmd = [
        sys.executable,
        r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Models\Guitar\tiny_phrase_gpt_full.py",
        "--base_dir", ".",
        "--style", style,
        "--generate_only",
        "--emotion", emotion,
        "--section", section_type,
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout

def extract_tokens(output):
    pattern = r"Generated tokens:\s*\[(.*?)\]"
    m = re.search(pattern, output, re.S)
    if not m:
        return []
    return re.findall(r"[A-Z_]+(?:_\d+MS|_\d+)?", m.group(1))

def tokens_to_midi_track(tokens, channel=0):
    track = mido.MidiTrack()
    vel = 64
    time_accum = 0

    for tok in tokens:
        if tok.startswith("VEL_"):
            vel_val = int(tok.split("_")[1]) * 20
            vel = max(1, min(127, vel_val))

        elif tok.startswith("NOTE_ON_"):
            note = int(tok.split("_")[2])
            msg = mido.Message(
                "note_on",
                note=note,
                velocity=vel,
                time=time_accum,
                channel=channel,
            )
            track.append(msg)
            time_accum = 0

        elif tok.startswith("NOTE_OFF_"):
            note = int(tok.split("_")[2])
            msg = mido.Message(
                "note_off",
                note=note,
                velocity=0,
                time=time_accum,
                channel=channel,
            )
            track.append(msg)
            time_accum = 0

        elif tok.startswith("TIME_SHIFT_"):
            ms = int(tok.split("_")[2].replace("MS", ""))
            time_accum += int(ms * MS_TO_TICKS)

    return track

def train_instrument(style):
    cmd = [
        sys.executable,
        r"C:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS\Models\Guitar\tiny_phrase_gpt_full.py",
        "--base_dir",
        ".",
        "--style",
        style,
    ]
    subprocess.run(cmd)

def zip_song_package(stems_dir):
    zip_base = stems_dir.parent / f"{SONG_TITLE}_package"
    shutil.make_archive(str(zip_base), "zip", stems_dir)

def build_band():
    mid = mido.MidiFile(ticks_per_beat=TPB)
    stems_dir = Path("stems")
    stems_dir.mkdir(exist_ok=True)

    lyrics_path = stems_dir / f"{SONG_TITLE}_lyrics.txt"
    lyrics_path.write_text(SONG_LYRICS, encoding="utf-8")

    for ch, inst in enumerate(SELECTED_INSTRUMENTS):
        style = ALL_INSTRUMENTS[inst]
        print(f"Training {inst} ({style})")
        train_instrument(style)

        print(f"Generating {inst}")
        all_tokens = []

        for section_type, bars in SONG_SECTIONS:
            if "VERSE" in section_type:
                emotion = "EMO_INTIMATE"
            else:
                emotion = "EMO_BIG"
            out = run_model(style, section_type, emotion)
            tokens = extract_tokens(out)
            all_tokens.extend(tokens)

        track = tokens_to_midi_track(all_tokens, channel=ch)
        track.name = inst
        mid.tracks.append(track)

        stem_mid = mido.MidiFile(ticks_per_beat=TPB)
        stem_mid.tracks.append(track)

        stem_mid_path = stems_dir / f"{SONG_TITLE}_{inst}.mid"
        stem_mid.save(stem_mid_path)

        if inst in SFZ_MAP:
            sfz_path = Path(SFZ_MAP[inst])
            wav_path = stems_dir / f"{SONG_TITLE}_{inst}.wav"
            print(f"Rendering WAV for {inst}")
            render_sfz_midi_to_wav(sfz_path, stem_mid_path, wav_path)

    band_mid_path = stems_dir / f"{SONG_TITLE}_band.mid"
    mid.save(band_mid_path)

    band_wav_path = stems_dir / f"{SONG_TITLE}_band.wav"
    print("Rendering full band WAV")
    render_sfz_midi_to_wav(Path(SFZ_MAP["Acoustic_Guitar"]), band_mid_path, band_wav_path)

    zip_song_package(stems_dir)

if __name__ == "__main__":
    print(f"Red Dirt Band – Song: {SONG_TITLE}")
    build_band()
    print("Done.")

