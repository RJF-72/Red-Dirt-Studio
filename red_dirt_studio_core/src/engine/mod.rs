pub mod renderer;
pub mod driver;

use anyhow::Result;
use crate::audio::AudioBuffer;
use crate::models::DriverEvent;

pub struct BandEngine {
    sample_rate: u32,
}

impl BandEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
    
    pub fn render_events(&self, events: &[DriverEvent], duration: f64) -> Result<AudioBuffer> {
        let num_samples = (duration * self.sample_rate as f64) as usize;
        let mut buffer = AudioBuffer::new(
            ndarray::Array1::zeros(num_samples),
            self.sample_rate,
            2,
        );
        
        // Simple envelope for each event
        let attack_samples = (self.sample_rate as f64 * 0.01) as usize;
        let decay_samples = (self.sample_rate as f64 * 0.5) as usize;
        
        for event in events {
            let start_sample = (event.time_sec * self.sample_rate as f64) as usize;
            if start_sample >= num_samples {
                continue;
            }
            
            let end_sample = (start_sample + decay_samples).min(num_samples);
            
            // Simple AD envelope
            for i in start_sample..end_sample {
                let envelope = if i - start_sample < attack_samples {
                    (i - start_sample) as f32 / attack_samples as f32
                } else {
                    1.0 - ((i - start_sample - attack_samples) as f32 / decay_samples as f32).min(1.0)
                };
                
                let value = envelope * event.velocity as f32 / 127.0 * 0.3;
                buffer.data[i] += value;
                if i + 1 < num_samples {
                    buffer.data[i + 1] += value;
                }
            }
        }
        
        Ok(buffer)
    }
}