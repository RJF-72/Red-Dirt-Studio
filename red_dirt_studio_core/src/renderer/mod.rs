use std::path::Path;
use anyhow::Result;
use crate::audio::AudioBuffer;
use crate::audio::wav::write_wav;

pub struct Renderer {
    sample_rate: u32,
    channels: u16,
}

impl Renderer {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self { sample_rate, channels }
    }
    
    pub fn render(&self, audio: &AudioBuffer, output_path: &Path) -> Result<()> {
        write_wav(output_path, audio)
    }
    
    pub fn mix_tracks(&self, tracks: &[AudioBuffer]) -> Result<AudioBuffer> {
        if tracks.is_empty() {
            return Ok(AudioBuffer::new(
                ndarray::Array1::zeros(0),
                self.sample_rate,
                self.channels,
            ));
        }
        
        let max_len = tracks.iter().map(|t| t.data.len()).max().unwrap_or(0);
        let mut mixed = ndarray::Array1::zeros(max_len);
        
        for track in tracks {
            for i in 0..track.data.len().min(max_len) {
                mixed[i] += track.data[i];
            }
        }
        
        let max_peak = mixed.iter().fold(0.0_f32, |a, &b: &f32| a.max(b.abs()));
        if max_peak > 1.0 {
            mixed.mapv_inplace(|x| x / max_peak);
        }
        
        Ok(AudioBuffer::new(mixed, self.sample_rate, self.channels))
    }
}