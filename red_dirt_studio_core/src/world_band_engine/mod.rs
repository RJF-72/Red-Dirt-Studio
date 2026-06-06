pub mod sfz_parser;
pub mod flac_aligner;
pub mod qwen_audio;
pub mod driver;
pub mod presets;
pub mod renderer;

use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub use sfz_parser::{SFZParser, SFZRegion};
pub use flac_aligner::{FLACAligner, LabeledSegment, DatasetBuilder};
pub use qwen_audio::{QwenAudioModel, QwenConfig};
pub use driver::{BandDriver, DriverEvent};
pub use presets::NASHVILLE_PRESETS;
pub use renderer::{HybridRenderer, SampleCache, SFZPlayer};

#[derive(Debug, Clone)]
pub struct WorldBandEngine {
    pub sfz_parser: SFZParser,
    pub flac_aligner: FLACAligner,
    pub qwen_model: QwenAudioModel,
    pub renderer: HybridRenderer,
    pub driver: BandDriver,
    pub preset: PresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub name: String,
    pub bpm: f64,
    pub chords: Vec<String>,
    pub instruments: Vec<String>,
}

impl WorldBandEngine {
    pub async fn build(
        sfz_path: &Path,
        sfz_root: &Path,
        flac_root: &Path,
        preset_name: &str,
    ) -> Result<Self> {
        let mut sfz_parser = SFZParser::new(sfz_path, sfz_root);
        let regions = sfz_parser.parse()?;
        
        let flac_aligner = FLACAligner::new(flac_root);
        
        let dataset_builder = DatasetBuilder::new(sfz_parser.clone(), flac_aligner.clone());
        let dataset = dataset_builder.build_dataset()?;
        tracing::info!("Dataset built with {} segments", dataset.len());
        
        let preset = presets::get_preset(preset_name)
            .ok_or_else(|| anyhow::anyhow!("Preset not found: {}", preset_name))?;
        
        let qwen_model = QwenAudioModel::new(QwenConfig::default()).await?;
        
        let sample_cache = SampleCache::new();
        let sfz_player = SFZPlayer::new(regions, sample_cache);
        
        let renderer = HybridRenderer::new(sfz_player.clone(), qwen_model.clone(), 48000);
        
        let driver = BandDriver::new(preset.bpm);
        
        Ok(Self {
            sfz_parser,
            flac_aligner,
            qwen_model,
            renderer,
            driver,
            preset: preset.clone(),
        })
    }
    
    pub async fn render_from_preset(&mut self) -> Result<Vec<f32>> {
        let chords = &self.preset.chords;
        let instruments = &self.preset.instruments;
        
        let mut all_events = Vec::new();
        
        for instrument in instruments {
            let events = self.driver.generate_phrase_plan(
                chords,
                8,
                instrument,
            );
            all_events.extend(events);
        }
        
        all_events.sort_by(|a, b| a.time_sec.partial_cmp(&b.time_sec).unwrap());
        
        let audio = self.renderer.render_events(&all_events).await?;
        Ok(audio)
    }
    
    pub async fn follow_vocal(&mut self, vocal_path: &Path) -> Result<Vec<f32>> {
        use crate::audio::read_audio_auto;
        
        let vocal_buffer = read_audio_auto(vocal_path)?;
        let vocal_audio = vocal_buffer.data.as_slice().unwrap();
        
        let events = self.qwen_model.band_brain_follow_vocal(
            vocal_audio,
            vocal_buffer.sample_rate as usize,
            &self.preset.chords,
            self.preset.bpm,
        ).await?;
        
        let events: Vec<DriverEvent> = events.into_iter()
            .map(|mut e| {
                e.instrument = "acoustic_gtr".to_string();
                e
            })
            .collect();
        
        let audio = self.renderer.render_events(&events).await?;
        Ok(audio)
    }
    
    pub fn export_wav(&self, audio: &[f32], output_path: &Path) -> Result<()> {
        self.renderer.export_wav(audio, output_path)
    }
}