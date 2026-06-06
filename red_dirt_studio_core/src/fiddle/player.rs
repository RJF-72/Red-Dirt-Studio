use rand::random;
use anyhow::Result;
use ndarray::Array1;
use crate::audio::AudioBuffer;
use crate::models::DriverEvent;

#[derive(Debug, Clone)]
pub struct FiddleVoice {
    sample_rate: u32,
    phase: f64,
    vibrato_phase: f64,
    glide_current: f64,
    glide_target: f64,
    envelope: f64,
}

impl FiddleVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            vibrato_phase: 0.0,
            glide_current: 440.0,
            glide_target: 440.0,
            envelope: 0.0,
        }
    }

    fn set_target_freq(&mut self, freq: f64) {
        self.glide_target = freq;
    }

    fn next_sample(&mut self, intensity: f32, articulation: &str) -> f32 {
        let max_brightness = 0.85f32;

        let brightness = match articulation {
            "chop" => max_brightness * 0.9,
            "solo_frag" | "bend" => max_brightness * 0.8,
            "double_stop" | "trill" => max_brightness * 0.7,
            "slide" => max_brightness * 0.75,
            "drone" => max_brightness * 0.5,
            "pizz" => max_brightness * 0.6,
            "staccato" => max_brightness * 0.65,
            _ => max_brightness * 0.7,
        };

        let glide_speed = match articulation {
            "slide" => 0.02f64,
            "bend" => 0.01f64,
            _ => 0.2f64,
        };

        self.glide_current += (self.glide_target - self.glide_current) * glide_speed;

        let vibrato_rate = 5.5f64;
        let vibrato_depth = match articulation {
            "sustain" | "pad" => 0.015f64,
            "bend" => 0.025f64,
            "trill" => 0.0f64,
            _ => 0.01f64,
        };

        self.vibrato_phase += vibrato_rate / self.sample_rate as f64;
        if self.vibrato_phase > 1.0 {
            self.vibrato_phase -= 1.0;
        }

        let vibrato = (self.vibrato_phase * std::f64::consts::TAU).sin() * vibrato_depth;
        let current_freq = self.glide_current * (1.0 + vibrato);

        self.phase += current_freq / self.sample_rate as f64;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        let t = self.phase * std::f64::consts::TAU;

        let saw = ((t.sin() + (2.0 * t).sin() * 0.5 + (3.0 * t).sin() * 0.25
            + (4.0 * t).sin() * 0.125
            + (5.0 * t).sin() * 0.0625)
            / 1.9375)
            * 0.5
            + 0.5;

        let saw_blend = brightness as f64;
        let tri = (t.sin().asin() * 2.0 / std::f64::consts::PI) * 0.5 + 0.5;
        let wave = saw * saw_blend + tri * (1.0 - saw_blend);

        let envelope_target = intensity as f64 * 0.3;
        match articulation {
            "chop" => {
                self.envelope *= 0.92;
            }
            "pizz" => {
                self.envelope *= 0.96;
            }
            "staccato" => {
                self.envelope += (envelope_target - self.envelope) * 0.1;
                if self.envelope > envelope_target * 0.8 {
                    self.envelope *= 0.98;
                }
            }
            _ => {
                self.envelope += (envelope_target - self.envelope) * 0.005;
            }
        }

        let noise = (random::<f32>() - 0.5) * 0.01 * intensity;
        ((wave * 2.0 - 1.0) * self.envelope) as f32 + noise
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.vibrato_phase = 0.0;
        self.envelope = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct FiddlePlayer {
    sample_rate: u32,
    voice: FiddleVoice,
    double_stop_voice: Option<Box<FiddleVoice>>,
}

impl FiddlePlayer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            voice: FiddleVoice::new(sample_rate),
            double_stop_voice: Some(Box::new(FiddleVoice::new(sample_rate))),
        }
    }

    fn midi_to_freq(midi: u8) -> f64 {
        440.0 * 2.0_f64.powf((midi as f64 - 69.0) / 12.0)
    }

    fn render_articulation(
        &mut self,
        pitch: u8,
        velocity: u8,
        articulation: &str,
        duration_sec: f64,
        intensity: f32,
    ) -> Vec<f32> {
        let num_samples = (duration_sec * self.sample_rate as f64) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let base_freq = Self::midi_to_freq(pitch);

        match articulation {
            "chop" => self.render_chop(&mut samples, base_freq, velocity, intensity),
            "staccato" => {
                self.render_staccato(&mut samples, base_freq, velocity, intensity)
            }
            "slide" => {
                let target_pitch = (pitch as i16 + 5).clamp(24, 96) as u8;
                let target_freq = Self::midi_to_freq(target_pitch);
                for i in 0..num_samples {
                    let t = i as f64 / num_samples as f64;
                    let freq = base_freq + (target_freq - base_freq) * t;
                    self.voice.set_target_freq(freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity, "slide");
                    samples.push(s);
                }
            }
            "bend" => {
                for i in 0..num_samples {
                    let t = i as f64 / num_samples as f64;
                    let bend_amount = (t * std::f64::consts::PI).sin() * 2.0;
                    let freq = base_freq * 2.0_f64.powf(bend_amount / 12.0);
                    self.voice.set_target_freq(freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity, "bend");
                    samples.push(s);
                }
            }
            "double_stop" => {
                let second = (pitch as i16 + 7).clamp(24, 96) as u8;
                let freq2 = Self::midi_to_freq(second);
                if let Some(ref mut ds) = self.double_stop_voice {
                    ds.phase = self.voice.phase;
                    for _i in 0..num_samples {
                        self.voice.set_target_freq(base_freq);
                        let s1 = self.voice.next_sample(velocity as f32 / 127.0 * intensity, "double_stop");
                        ds.set_target_freq(freq2);
                        let s2 = ds.next_sample(velocity as f32 / 127.0 * intensity * 0.7, "double_stop");
                        samples.push((s1 + s2) * 0.6);
                    }
                } else {
                    return self.render_articulation(pitch, velocity, "sustain", duration_sec, intensity);
                }
            }
            "trill" => {
                let upper = (pitch as i16 + 2).clamp(24, 96) as u8;
                let freq2 = Self::midi_to_freq(upper);
                let rate = 8.0;
                for i in 0..num_samples {
                    let t = i as f64 / self.sample_rate as f64;
                    let toggle = ((t * rate * std::f64::consts::PI).sin() + 1.0) * 0.5;
                    let freq = base_freq + (freq2 - base_freq) * toggle;
                    self.voice.set_target_freq(freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity * 0.8, "trill");
                    samples.push(s);
                }
            }
            "drone" => {
                for _i in 0..num_samples {
                    self.voice.set_target_freq(base_freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity * 0.4, "drone");
                    samples.push(s);
                }
            }
            "pizz" => {
                for i in 0..num_samples {
                    let decay = (-3.0 * i as f64 / num_samples as f64).exp();
                    self.voice.set_target_freq(base_freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity * 0.5, "pizz");
                    samples.push(s * decay as f32);
                }
            }
            _ => {
                for _i in 0..num_samples {
                    self.voice.set_target_freq(base_freq);
                    let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity, "sustain");
                    samples.push(s);
                }
            }
        }

        let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs())).max(1e-6);
        let scale = (velocity as f32 / 127.0 * 0.5) / peak;
        for s in &mut samples {
            *s *= scale;
        }

        samples
    }

    fn render_staccato(&mut self, samples: &mut Vec<f32>, base_freq: f64, velocity: u8, intensity: f32) {
        let count = (0.08 * self.sample_rate as f64) as usize;
        for i in 0..count {
            let env = (-8.0 * i as f64 / count as f64).exp();
            self.voice.set_target_freq(base_freq);
            let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity, "staccato");
            samples.push(s * env as f32);
        }
    }

    fn render_chop(&mut self, samples: &mut Vec<f32>, base_freq: f64, velocity: u8, intensity: f32) {
        let count = (0.04 * self.sample_rate as f64) as usize;
        for i in 0..count {
            let env = (-12.0 * i as f64 / count as f64).exp();
            self.voice.set_target_freq(base_freq);
            let noise = (random::<f32>() - 0.5) * 0.3;
            let s = self.voice.next_sample(velocity as f32 / 127.0 * intensity * 0.3, "chop");
            samples.push((s + noise) * env as f32);
        }
    }

    pub fn render_event(&mut self, event: &DriverEvent, duration_sec: f64) -> Vec<f32> {
        let intensity = event.meta.get("intensity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;
        self.render_articulation(
            event.pitch,
            event.velocity,
            &event.articulation,
            duration_sec,
            intensity,
        )
    }

    pub fn render_events(&mut self, events: &[DriverEvent], total_duration: f64) -> Result<AudioBuffer> {
        if events.is_empty() {
            return Ok(AudioBuffer::new(
                Array1::zeros(0),
                self.sample_rate,
                2,
            ));
        }

        let total_samples = (total_duration * self.sample_rate as f64) as usize;
        let mut mix = vec![0.0f32; total_samples * 2];

        let _sec_per_beat = 60.0 / 120.0;

        for event in events {
            let duration = event.meta.get("duration_sec")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            let note_samples = self.render_event(event, duration);
            let start = (event.time_sec * self.sample_rate as f64) as usize;

            for (i, &s) in note_samples.iter().enumerate() {
                let idx = start + i;
                if idx * 2 + 1 < mix.len() {
                    mix[idx * 2] += s;
                    mix[idx * 2 + 1] += s * 0.9;
                }
            }
        }

        let peak = mix.iter().fold(0.0f32, |a, &b| a.max(b.abs())).max(1e-6);
        let scale = 0.75 / peak;
        for s in &mut mix {
            *s *= scale;
        }

        Ok(AudioBuffer::new(
            Array1::from_vec(mix),
            self.sample_rate,
            2,
        ))
    }

    pub fn reset(&mut self) {
        self.voice.reset();
        if let Some(ref mut ds) = self.double_stop_voice {
            ds.reset();
        }
    }
}
