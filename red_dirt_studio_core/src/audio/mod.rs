pub mod wav;

pub use wav::{read_wav, read_flac, read_audio_auto};

use anyhow::Result;
use ndarray::Array1;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioBuffer {
    pub data: Array1<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioBuffer {
    pub fn new(data: Array1<f32>, sample_rate: u32, channels: u16) -> Self {
        Self { data, sample_rate, channels }
    }

    pub fn duration(&self) -> f64 {
        self.data.len() as f64 / self.sample_rate as f64
    }

    pub fn normalize(&mut self, target_peak: f32) {
        let max_peak = self.data.iter().fold(0.0_f32, |a, &b| a.max(b.abs()));
        if max_peak > 0.0 {
            let gain = target_peak / max_peak;
            self.data.mapv_inplace(|x| x * gain);
        }
    }

    pub fn resample(&self, target_rate: u32) -> Result<Self> {
        use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters};
        
        if self.sample_rate == target_rate {
            return Ok(self.clone());
        }

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: rubato::WindowFunction::BlackmanHarris2,
        };

        let mut resampler = SincFixedIn::<f32>::new(
            target_rate as f64 / self.sample_rate as f64,
            1.0,
            params,
            self.data.len(),
            1,
        )?;

        let wave_in = vec![self.data.as_slice().unwrap().to_vec()];
        let wave_out = resampler.process(&wave_in, None)?;
        
        Ok(Self {
            data: Array1::from(wave_out[0].clone()),
            sample_rate: target_rate,
            channels: self.channels,
        })
    }

    pub fn mix(&self, other: &Self) -> Result<Self> {
        let max_len = self.data.len().max(other.data.len());
        let mut mixed = Array1::zeros(max_len);
        
        for i in 0..self.data.len() {
            mixed[i] += self.data[i];
        }
        for i in 0..other.data.len() {
            mixed[i] += other.data[i];
        }
        
        Ok(Self {
            data: mixed,
            sample_rate: self.sample_rate,
            channels: self.channels.max(other.channels),
        })
    }
}