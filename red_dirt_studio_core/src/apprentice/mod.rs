pub mod logger;
pub mod dataset;
pub mod trainer;
pub mod oracle;
pub mod config;

pub use logger::*;
pub use dataset::*;
pub use trainer::*;
pub use oracle::*;
pub use config::*;

use anyhow::Result;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// The Rust Apprentice: a self-improving brain that learns your band's style
/// and guides song generation over time.
pub struct RustApprentice {
    pub logger: Arc<Mutex<SongLogger>>,
    pub dataset: Arc<Mutex<SongDataset>>,
    pub trainer: Arc<Mutex<ApprenticeTrainer>>,
    pub oracle: ApprenticeOracle,
    pub config: ApprenticeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SongLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub key: String,
    pub tempo_bpm: f32,
    pub sections: Vec<String>,  // e.g., ["INTRO", "VERSE1", "CHORUS1", ...]
    pub chords: Vec<String>,    // serialized chord sequences
    pub drum_pattern_ids: Vec<String>,
    pub bass_movement_ids: Vec<String>,
    pub acoustic_pattern_ids: Vec<String>,
    pub arrangement_density: f32,  // 0.0..1.0 avg instruments per section
    pub user_rating: Option<f32>,  // optional: 1-5 stars
    pub style_alignment: f32,       // apprentice's confidence this matches style (0-1)
}

impl RustApprentice {
    pub fn new(config: ApprenticeConfig) -> Self {
        Self {
            logger: Arc::new(Mutex::new(SongLogger::new())),
            dataset: Arc::new(Mutex::new(SongDataset::new(config.buffer_size))),
            trainer: Arc::new(Mutex::new(ApprenticeTrainer::new(config.clone()))),
            oracle: ApprenticeOracle::new(config.clone()),
            config,
        }
    }

    /// Called after each song is generated: log it, optionally learn from it
    pub fn observe_song(&self, song_log: SongLog) -> Result<()> {
        let mut dataset = self.dataset.lock().unwrap();
        dataset.add_song(song_log.clone());

        // Check if we should trigger a training update
        if self.config.auto_train_enabled {
            let mut trainer = self.trainer.lock().unwrap();
            trainer.maybe_update(&dataset, &self.config)?;
        }

        Ok(())
    }

    /// Called before each song: get style-aware suggestions
    pub fn suggest_song_plan(
        &self,
        prompt: &str,
        user_key: Option<String>,
        user_tempo: Option<f32>,
        style_strength: f32,
    ) -> Result<crate::instruments::SongPlan> {
        let dataset = self.dataset.lock().unwrap();
        self.oracle.suggest_song_plan(prompt, user_key, user_tempo, style_strength, &dataset)
    }

    /// Mark a song as a favorite (anchor) for continuous learning
    pub fn pin_favorite(&self, song_id: String) -> Result<()> {
        let mut dataset = self.dataset.lock().unwrap();
        dataset.pin_favorite(song_id);
        Ok(())
    }

    /// Save current apprentice state (weights + metadata)
    pub fn save_checkpoint(&self, version: String) -> Result<()> {
        let trainer = self.trainer.lock().unwrap();
        trainer.save_checkpoint(version)
    }

    /// Load a previous apprentice version
    pub fn load_checkpoint(&mut self, version: String) -> Result<()> {
        let mut trainer = self.trainer.lock().unwrap();
        trainer.load_checkpoint(version)
    }

    /// Get current apprentice version
    pub fn get_version(&self) -> String {
        let trainer = self.trainer.lock().unwrap();
        trainer.get_version()
    }
}
