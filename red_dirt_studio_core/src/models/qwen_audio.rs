use anyhow::Result;
use crate::models::DriverEvent;

#[derive(Debug, Clone)]
pub struct QwenConfig {
    pub model_name: String,
    pub device: String,
    pub use_cuda: bool,
}

impl Default for QwenConfig {
    fn default() -> Self {
        Self {
            model_name: "Qwen/Qwen2-Audio-1.5B".to_string(),
            device: "cpu".to_string(),
            use_cuda: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QwenAudioModel {
    #[allow(dead_code)]
    config: QwenConfig,
}

impl QwenAudioModel {
    pub async fn new(config: QwenConfig) -> Result<Self> {
        tracing::info!("Initializing QwenAudioModel with config: {:?}", config);
        Ok(Self { config })
    }
    
    pub async fn embed_audio(&self, audio_buffer: &[f32], _sample_rate: usize) -> Result<Vec<f32>> {
        let target_len = (audio_buffer.len() as f64 * 0.01) as usize;
        let step = if audio_buffer.len() > target_len {
            audio_buffer.len() / target_len
        } else {
            1
        };
        
        let embedding: Vec<f32> = audio_buffer
            .iter()
            .step_by(step)
            .take(target_len)
            .copied()
            .collect();
        
        Ok(embedding)
    }
    
    pub async fn generate_residual(&self, sfz_buffer: &[f32], scale: f32) -> Result<Vec<f32>> {
        let embedding = self.embed_audio(sfz_buffer, 48000).await?;
        
        let target_len = sfz_buffer.len();
        let mut residual = vec![0.0f32; target_len];
        
        for i in 0..target_len {
            let idx = ((i as f64 / target_len as f64) * embedding.len() as f64) as usize;
            if idx < embedding.len() {
                residual[i] = embedding[idx] * scale;
            }
        }
        
        let max_val = residual.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        if max_val > 0.0 {
            for v in &mut residual {
                *v /= max_val;
            }
        }
        
        Ok(residual)
    }
    
    pub async fn band_brain_follow_vocal(
        &self,
        vocal_audio: &[f32],
        sample_rate: usize,
        chord_progression: &[String],
        bpm: f64,
    ) -> Result<Vec<DriverEvent>> {
        let window_size = (0.05 * sample_rate as f64) as usize;
        let mut envelope = vec![0.0f32; vocal_audio.len()];
        
        for i in 0..vocal_audio.len() {
            let start = i.saturating_sub(window_size);
            let end = (i + window_size).min(vocal_audio.len());
            let window_energy: f32 = vocal_audio[start..end].iter().map(|&s| s.abs()).sum();
            envelope[i] = window_energy / ((end - start) as f32);
        }
        
        let mean_energy: f32 = envelope.iter().sum::<f32>() / envelope.len() as f32;
        let threshold = mean_energy * 2.0;
        
        let mut onsets = Vec::new();
        for i in 1..envelope.len() {
            if envelope[i] >= threshold && envelope[i - 1] < threshold {
                onsets.push(i);
            }
        }
        
        let _sec_per_beat = 60.0 / bpm;
        let mut events = Vec::new();
        
        for (i, &onset) in onsets.iter().enumerate() {
            let time_sec = onset as f64 / sample_rate as f64;
            let chord_idx = i % chord_progression.len();
            let chord = &chord_progression[chord_idx];
            let root_pitch = Self::chord_root_to_midi(chord);
            
            events.push(DriverEvent {
                time_sec,
                pitch: root_pitch,
                velocity: 100,
                articulation: "sustain".to_string(),
                group: "band".to_string(),
                instrument: "band".to_string(),
                chord: Some(chord.clone()),
                meta: serde_json::json!({"source": "vocal_follow", "onset_index": i}),
            });
        }
        
        Ok(events)
    }
    
    fn chord_root_to_midi(chord: &str) -> u8 {
        let root = chord.chars().next().unwrap_or('C').to_ascii_uppercase();
        match root {
            'C' => 60,
            'D' => 62,
            'E' => 64,
            'F' => 65,
            'G' => 67,
            'A' => 69,
            'B' => 71,
            _ => 60,
        }
    }
}