use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use ndarray::Array1;
use crate::audio::wav::write_wav;
use crate::audio::AudioBuffer;
use super::sfz_parser::SFZRegion;
use super::qwen_audio::QwenAudioModel;
use super::driver::DriverEvent;

#[derive(Debug, Clone)]
pub struct SampleCache {
    cache: HashMap<String, (Vec<f32>, u32)>,
}

impl SampleCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn load(&mut self, path: &Path) -> Result<(Vec<f32>, u32)> {
        if let Some(cached) = self.cache.get(path.to_str().unwrap_or("")) {
            return Ok(cached.clone());
        }
        
use symphonia::core::io::MediaSourceStream;
         use symphonia::default::get_codecs;
         
         let file = std::fs::File::open(path)?;
         let mss = MediaSourceStream::new(Box::new(file), Default::default());
         let probed = symphonia::default::get_probe().format(&Default::default(), mss, &Default::default(), &Default::default())?;
         let mut demux = probed.format;
        
        let track = demux.default_track().ok_or_else(|| anyhow::anyhow!("No default track"))?;
        let mut decoder = get_codecs().make(&track.codec_params, &Default::default())?;
        
        let mut samples = Vec::new();
        let sample_rate = track.codec_params.sample_rate.unwrap_or(48000);
        
        while let Ok(packet) = demux.next_packet() {
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    use symphonia::core::audio::SampleBuffer;
                    let spec = *audio_buf.spec();
                    let frames = audio_buf.frames() as u64;
                    let mut sample_buf = SampleBuffer::<f32>::new(frames, spec);
                    sample_buf.copy_interleaved_ref(audio_buf);
                    samples.extend_from_slice(sample_buf.samples());
                }
                Err(_) => continue,
            }
        }
        
        let result = (samples, sample_rate);
        self.cache.insert(path.to_str().unwrap_or("").to_string(), result.clone());
        
        Ok(result)
    }
}

impl Default for SampleCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SFZPlayer {
    regions: Vec<SFZRegion>,
    cache: SampleCache,
}

impl SFZPlayer {
    pub fn new(regions: Vec<SFZRegion>, cache: SampleCache) -> Self {
        Self { regions, cache }
    }
    
    pub fn find_region(
        &self,
        pitch: u8,
        velocity: u8,
        instrument: &str,
        articulation: &str,
    ) -> Option<&SFZRegion> {
        for region in &self.regions {
            if region.instrument == instrument
                && region.lokey <= pitch && pitch <= region.hikey
                && region.lovel <= velocity && velocity <= region.hivel
                && region.articulation == articulation
            {
                return Some(region);
            }
        }
        
        for region in &self.regions {
            if region.instrument == instrument {
                return Some(region);
            }
        }
        
        self.regions.first()
    }
    
    pub async fn render_note(
        &mut self,
        pitch: u8,
        velocity: u8,
        instrument: &str,
        articulation: &str,
        length_sec: f64,
        target_sr: u32,
    ) -> Result<Vec<f32>> {
        let region = self.find_region(pitch, velocity, instrument, articulation);
        
        if let Some(region) = region {
            let sample_path = region.sample_path.clone();
            let pkc = region.pitch_keycenter;
            let (audio, _sample_rate) = self.cache.load(&sample_path)?;
            
            let semitones = pitch as i32 - pkc as i32;
            let factor = 2.0_f64.powf(semitones as f64 / 12.0);
            
            let target_len = (length_sec * target_sr as f64) as usize;
            let src_len = (target_len as f64 / factor) as usize;
            let src_len = src_len.min(audio.len());
            
            let src = &audio[..src_len];
            
            let mut resampled = vec![0.0f32; target_len];
            for i in 0..target_len {
                let src_idx = (i as f64 * factor) as usize;
                if src_idx < src_len {
                    resampled[i] = src[src_idx];
                }
            }
            
            let amp = velocity as f32 / 127.0;
            for sample in &mut resampled {
                *sample *= amp;
            }
            
            Ok(resampled)
        } else {
            Ok(vec![0.0f32; (length_sec * target_sr as f64) as usize])
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridRenderer {
    sfz_player: SFZPlayer,
    qwen_model: QwenAudioModel,
    sample_rate: u32,
}

impl HybridRenderer {
    pub fn new(sfz_player: SFZPlayer, qwen_model: QwenAudioModel, sample_rate: u32) -> Self {
        Self {
            sfz_player,
            qwen_model,
            sample_rate,
        }
    }
    
    pub async fn render_events(&mut self, events: &[DriverEvent]) -> Result<Vec<f32>> {
        if events.is_empty() {
            return Ok(vec![0.0f32]);
        }
        
        let frames_per_note = (0.5 * self.sample_rate as f64) as usize;
        let max_time = events.iter().map(|e| e.time_sec).fold(0.0, f64::max);
        let total_frames = (max_time * self.sample_rate as f64) as usize + frames_per_note;
        
        let mut mix = vec![0.0f32; total_frames];
        
        for event in events {
            let note = self.sfz_player.render_note(
                event.pitch,
                event.velocity,
                &event.instrument,
                &event.articulation,
                0.5,
                self.sample_rate,
            ).await?;
            
            let start = (event.time_sec * self.sample_rate as f64) as usize;
            let end = (start + note.len()).min(total_frames);
            
            for i in start..end {
                mix[i] += note[i - start];
            }
        }
        
        let residual = self.qwen_model.generate_residual(&mix, 0.05).await?;
        for i in 0..mix.len().min(residual.len()) {
            mix[i] += residual[i];
        }
        
        let max_val = mix.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        if max_val > 0.0 {
            for sample in &mut mix {
                *sample /= max_val;
            }
        }
        
        Ok(mix)
    }
    
    pub fn export_wav(&self, audio: &[f32], output_path: &Path) -> Result<()> {
        let buffer = AudioBuffer::new(
            Array1::from(audio.to_vec()),
            self.sample_rate,
            1,
        );
        write_wav(output_path, &buffer)
    }
}