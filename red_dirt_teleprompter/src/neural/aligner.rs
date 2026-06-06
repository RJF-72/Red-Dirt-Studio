use candle_core::{Tensor, Device, Result as CandleResult, DType};
use candle_core::Module;
use candle_nn::{VarBuilder, VarMap};
use std::sync::Arc;

use crate::{TeleprompterConfig, TeleprompterError, LyricLine, LyricWord};

/// Neural lyric alignment system
/// 
/// NOTE: This is currently a stub implementation using randomly initialized
/// weights. For production use, you MUST load pretrained weights or train
/// the network on aligned lyric-audio pairs.
pub struct NeuralAligner {
    /// Alignment neural network
    alignment_network: AlignmentNetwork,
    
    /// Feature extractor for audio
    feature_extractor: AudioFeatureExtractor,
    
    /// Lyric timing predictor
    timing_predictor: TimingPredictor,
    
    /// Device for computation
    device: Arc<Device>,
    
    /// Configuration
    config: NeuralConfig,
}

#[derive(Debug, Clone)]
pub struct NeuralConfig {
    pub sample_rate: f32,
    pub neural_resolution: usize,
    pub feature_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
}

impl NeuralConfig {
    fn default() -> Self {
        Self {
            sample_rate: 96000.0,
            neural_resolution: 1024,
            feature_dim: 256,
            hidden_dim: 512,
            output_dim: 2, // start_time, end_time
        }
    }
}

impl NeuralAligner {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        let device = Device::Cpu; // Could be Cuda/Metal in real implementation
        let device = Arc::new(device);
        
        // VarMap holds the model variables. VarBuilder gets a cloned Arc so it
        // remains valid even after the local `varmap` binding is dropped.
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        
        let alignment_network = AlignmentNetwork::new(&NeuralConfig::default(), vb.pp("alignment"))?;
        let feature_extractor = AudioFeatureExtractor::new(device.clone());
        let timing_predictor = TimingPredictor::new(config.sample_rate);

        Ok(Self {
            alignment_network,
            feature_extractor,
            timing_predictor,
            device,
            config: NeuralConfig::default(),
        })
    }

    /// Align lyrics to audio using neural network
    pub fn align_lyrics_to_audio(
        &self,
        lyrics: &[LyricLine],
        audio: &[f32],
    ) -> Result<Vec<LyricLine>, TeleprompterError> {
        if audio.is_empty() {
            return Err(TeleprompterError::AudioDecodeError);
        }

        let mut aligned_lyrics = Vec::with_capacity(lyrics.len());
        
        for (_line_idx, lyric_line) in lyrics.iter().enumerate() {
            // Extract audio features around expected timing
            let expected_time = lyric_line.start_time;
            let audio_segment = self.extract_audio_segment(audio, expected_time)?;
            
            if audio_segment.is_empty() {
                // Fallback: keep original timing if no audio segment available
                aligned_lyrics.push(lyric_line.clone());
                continue;
            }

            // Extract features from audio segment
            let features = self.feature_extractor.extract_features(&audio_segment)?;
            
            // Predict actual timing using neural network
            let predicted_times = self.predict_timing(&features, lyric_line)?;
            
            // Create aligned line
            let mut aligned_line = lyric_line.clone();
            aligned_line.start_time = predicted_times.0;
            aligned_line.end_time = predicted_times.1;
            
            // Align individual words
            aligned_line.words = self.align_words(&lyric_line.words, &audio_segment, expected_time)?;
            
            aligned_lyrics.push(aligned_line);
        }
        
        Ok(aligned_lyrics)
    }

    fn extract_audio_segment(&self, audio: &[f32], expected_time_ms: f32) -> Result<Vec<f32>, TeleprompterError> {
        if audio.is_empty() {
            return Ok(Vec::new());
        }

        // Convert time to sample indices
        let expected_sample = (expected_time_ms * self.config.sample_rate / 1000.0) as usize;
        let segment_length = (2000.0 * self.config.sample_rate / 1000.0) as usize; // 2 second segment
        
        let start = expected_sample.saturating_sub(segment_length / 2);
        let end = (expected_sample + segment_length / 2).min(audio.len());
        
        // Ensure start <= end to avoid panic on empty/small slices
        let start = start.min(end);
        
        Ok(audio[start..end].to_vec())
    }

    fn predict_timing(&self, features: &Tensor, lyric_line: &LyricLine) -> Result<(f32, f32), TeleprompterError> {
        // Pass features through alignment network
        let timing_tensor = self.alignment_network.forward(features)?;
        let timing_values = timing_tensor.to_vec1::<f32>()?;
        
        if timing_values.len() < 2 {
            return Err(TeleprompterError::NeuralModelError("Insufficient output dimensions".to_string()));
        }

        // Convert neural output to actual times
        let start_offset = timing_values[0] * 100.0; // ±100ms offset
        let duration_offset = timing_values[1] * 50.0; // ±50ms duration change
        
        let predicted_start = lyric_line.start_time + start_offset;
        let predicted_end = predicted_start + (lyric_line.end_time - lyric_line.start_time) + duration_offset;
        
        Ok((predicted_start, predicted_end))
    }

    fn align_words(
        &self,
        words: &[LyricWord],
        audio_segment: &[f32],
        base_time: f32,
    ) -> Result<Vec<LyricWord>, TeleprompterError> {
        let mut aligned_words = Vec::with_capacity(words.len());
        
        // Use onset detection to find actual word boundaries
        let onset_times = self.detect_onsets(audio_segment)?;
        
        // Align words to detected onsets
        let mut onset_idx = 0;
        
        for (i, word) in words.iter().enumerate() {
            let mut aligned_word = word.clone();
            
            if onset_idx < onset_times.len() {
                let onset_time = onset_times[onset_idx] as f32 / self.config.sample_rate * 1000.0;
                aligned_word.start_time = base_time + onset_time;
                
                // Calculate end time based on next onset or word duration
                if onset_idx + 1 < onset_times.len() {
                    let next_onset_time = onset_times[onset_idx + 1] as f32 / self.config.sample_rate * 1000.0;
                    aligned_word.end_time = base_time + next_onset_time;
                } else {
                    // Use average word duration if no next onset
                    aligned_word.end_time = aligned_word.start_time + 300.0; // 300ms average
                }
                
                onset_idx += 1;
            } else {
                // Fallback: use original timing
                aligned_word.start_time = base_time + (i as f32 * 300.0);
                aligned_word.end_time = aligned_word.start_time + 300.0;
            }
            
            aligned_words.push(aligned_word);
        }
        
        Ok(aligned_words)
    }

    fn detect_onsets(&self, audio: &[f32]) -> Result<Vec<usize>, TeleprompterError> {
        if audio.is_empty() {
            return Ok(Vec::new());
        }

        let mut onsets = Vec::new();
        
        // Calculate short-term energy
        let frame_size = (self.config.sample_rate as usize / 100).max(44); // 10ms frames
        let mut prev_energy = 0.0f32;
        
        for frame_start in (0..audio.len()).step_by(frame_size) {
            let frame_end = (frame_start + frame_size).min(audio.len());
            let frame = &audio[frame_start..frame_end];
            
            if frame.is_empty() {
                continue;
            }

            let energy: f32 = frame.iter().map(|&x| x * x).sum();
            let normalized_energy = energy / frame.len() as f32;
            
            // Detect onset if energy increases significantly
            if normalized_energy > prev_energy * 2.0 && normalized_energy > 0.001 {
                onsets.push(frame_start);
            }
            
            prev_energy = normalized_energy;
        }
        
        Ok(onsets)
    }
}

/// Neural network for timing alignment
struct AlignmentNetwork {
    layers: Vec<NeuralLayer>,
    config: NeuralConfig,
}

impl AlignmentNetwork {
    fn new(config: &NeuralConfig, vb: VarBuilder) -> CandleResult<Self> {
        let mut layers = Vec::new();
        
        // Input layer
        layers.push(NeuralLayer::new(config.feature_dim, config.hidden_dim, vb.pp("input"))?);
        
        // Hidden layers
        for i in 0..2 {
            layers.push(NeuralLayer::new(config.hidden_dim, config.hidden_dim, vb.pp(&format!("hidden_{}", i)))?);
        }
        
        // Output layer
        layers.push(NeuralLayer::new(config.hidden_dim, config.output_dim, vb.pp("output"))?);
        
        Ok(Self {
            layers,
            config: config.clone(),
        })
    }

    fn forward(&self, x: &Tensor) -> CandleResult<Tensor> {
        let mut x = x.clone();
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x)?;
            // Apply activation except for last layer
            if i + 1 < self.layers.len() {
                x = x.relu()?;
            }
        }
        Ok(x)
    }
}

/// Audio feature extractor
struct AudioFeatureExtractor {
    device: Arc<Device>,
}

impl AudioFeatureExtractor {
    fn new(device: Arc<Device>) -> Self {
        Self { device }
    }

    fn extract_features(&self, audio: &[f32]) -> Result<Tensor, TeleprompterError> {
        use rustfft::{FftPlanner, num_complex::Complex};

        if audio.is_empty() {
            return Err(TeleprompterError::NeuralModelError("Cannot extract features from empty audio".to_string()));
        }
        
        // Use a fixed-size FFT to avoid memory blow-up on very long segments.
        const FFT_SIZE: usize = 2048;
        let fft_input_len = audio.len().min(FFT_SIZE);
        
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_input_len);
        
        let mut buffer: Vec<Complex<f32>> = audio.iter().take(fft_input_len)
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        
        // Pad to FFT size if needed
        buffer.resize(fft_input_len, Complex::new(0.0, 0.0));
        
        fft.process(&mut buffer);
        
        // Extract key features
        let mut features = Vec::with_capacity(256);
        
        // Spectral centroid
        let magnitude_spectrum: Vec<f32> = buffer.iter()
            .map(|c| c.norm())
            .collect();
        
        let mut centroid_sum = 0.0;
        let mut total_power = 0.0;
        for (i, &mag) in magnitude_spectrum.iter().enumerate() {
            centroid_sum += i as f32 * mag;
            total_power += mag;
        }
        let centroid = if total_power > 0.0 { centroid_sum / total_power } else { 0.0 };
        features.push(centroid);
        
        // Spectral rolloff
        let cumulative_power: Vec<f32> = magnitude_spectrum.iter()
            .scan(0.0, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        
        let total_power = cumulative_power.last().copied().unwrap_or(1.0);
        let rolloff_threshold = total_power * 0.95;
        
        let rolloff_idx = cumulative_power.iter()
            .position(|&power| power >= rolloff_threshold)
            .unwrap_or(magnitude_spectrum.len());
        
        features.push(rolloff_idx as f32 / magnitude_spectrum.len().max(1) as f32);
        
        // Zero crossing rate
        let mut zcr = 0.0;
        for i in 1..audio.len() {
            if (audio[i] >= 0.0) != (audio[i-1] >= 0.0) {
                zcr += 1.0;
            }
        }
        zcr /= audio.len() as f32;
        features.push(zcr);
        
        // RMS energy
        let rms = (audio.iter().map(|&x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        features.push(rms);
        
        // Pad to feature dimension
        features.resize(256, 0.0);
        
        Tensor::from_vec(features, (1, 256), &self.device)
            .map_err(|e| TeleprompterError::NeuralModelError(e.to_string()))
    }
}

/// Timing predictor for word-level alignment
struct TimingPredictor {
    sample_rate: f32,
}

impl TimingPredictor {
    fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    fn predict_word_timing(&self, word: &str, context: &str) -> (f32, f32) {
        // Simple prediction based on word length and context
        let word_duration_ms = (word.len() as f32 * 50.0).max(100.0).min(800.0); // 100-800ms
        let pause_before = if context.ends_with(['.', '?', '!']) { 200.0 } else { 50.0 }; // Pause after sentence
        
        (pause_before, pause_before + word_duration_ms)
    }
}

/// Generic neural layer
struct NeuralLayer {
    weight: candle_nn::Linear,
}

impl NeuralLayer {
    fn new(input_size: usize, output_size: usize, vb: VarBuilder) -> CandleResult<Self> {
        let weight = candle_nn::linear(input_size, output_size, vb.pp("weight"))?;
        
        Ok(Self {
            weight,
        })
    }

    fn forward(&self, x: &Tensor) -> CandleResult<Tensor> {
        self.weight.forward(x)
    }
}
