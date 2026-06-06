use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;
use crate::instruments::tokens::*;
use crate::instruments::models::*;
use crate::readiness::ReadinessReport;

/// Auto-swap manager that handles safe model promotion and versioning
pub struct AutoSwapManager {
    models_dir: PathBuf,
    owner_only: bool,
}

impl AutoSwapManager {
    pub fn new(owner_only: bool) -> Self {
        Self {
            models_dir: PathBuf::from("models"),
            owner_only,
        }
    }

    /// Archive old model and activate new one
    pub fn promote_model(
        &self,
        instrument: &str,
        version_id: &str,
        _report: &ReadinessReport,
    ) -> Result<String> {
        let _ = instrument;
        let _ = version_id;
        // Create archive directory if needed
        let archive_dir = self.models_dir.join("archive");
        std::fs::create_dir_all(&archive_dir)?;

        // In production, this would:
        // 1. Move old active model to archive
        // 2. Copy new model to active location
        // 3. Update model registry

        Ok(format!("{} promoted successfully", version_id))
    }

    /// Check if auto-swap is allowed
    pub fn can_auto_swap(&self) -> bool {
        self.owner_only
    }

    /// Get model weight path for an instrument
    pub fn get_model_path(&self, instrument: &str, version: &str) -> PathBuf {
        self.models_dir
            .join(instrument)
            .join(format!("{}.safetensors", version))
    }

    /// Check for pending model approvals and auto-swap if enabled
    pub fn check_and_swap(
        &self,
        model_manager: &crate::model_manager::ModelManager,
    ) -> Result<Option<String>> {
        if !self.can_auto_swap() {
            return Ok(None);
        }

        // Return next instrument to train if no pending approvals
        if let Some(next) = model_manager.suggest_next_instrument() {
            return Ok(Some(next));
        }

        Ok(None)
    }
}

/// Instrument model registry for managing loaded models
pub struct InstrumentModelRegistry {
    drum_model: Option<Arc<dyn DrumModel + Send + Sync>>,
    bass_model: Option<Arc<dyn BassModel + Send + Sync>>,
    acoustic_model: Option<Arc<dyn AcousticModel + Send + Sync>>,
    fiddle_model: Option<Arc<dyn FiddleModel + Send + Sync>>,
}

impl InstrumentModelRegistry {
    pub fn new() -> Self {
        Self {
            drum_model: Some(Arc::new(StubDrumModel::new("v001".to_string()))),
            bass_model: Some(Arc::new(StubBassModel::new("v001".to_string()))),
            acoustic_model: Some(Arc::new(StubAcousticModel::new("v001".to_string()))),
            fiddle_model: Some(Arc::new(StubFiddleModel::new("v001".to_string()))),
        }
    }

    pub fn generate_drum_plan(
        &self,
        section: &SectionSpec,
    ) -> Result<DrumPlan> {
        match &self.drum_model {
            Some(model) => model.generate_drums(section),
            None => Err(anyhow::anyhow!("No drum model loaded")),
        }
    }

    pub fn generate_bass_plan(
        &self,
        section: &SectionSpec,
        chords: &[ChordToken],
        drums: &DrumPlan,
    ) -> Result<BassPlan> {
        match &self.bass_model {
            Some(model) => model.generate_bass(section, chords, drums),
            None => Err(anyhow::anyhow!("No bass model loaded")),
        }
    }

    pub fn generate_acoustic_plan(
        &self,
        section: &SectionSpec,
        chords: &[ChordToken],
    ) -> Result<AcousticPlan> {
        match &self.acoustic_model {
            Some(model) => model.generate_acoustic(section, chords),
            None => Err(anyhow::anyhow!("No acoustic model loaded")),
        }
    }

    pub fn generate_fiddle_plan(
        &self,
        section: &SectionSpec,
        chords: &[ChordToken],
        vocal_map: Option<&[(f64, f64)]>,
    ) -> Result<FiddlePlan> {
        match &self.fiddle_model {
            Some(model) => model.generate_fiddle(section, chords, vocal_map),
            None => Err(anyhow::anyhow!("No fiddle model loaded")),
        }
    }

    pub fn set_drum_model(&mut self, model: Option<Arc<dyn DrumModel + Send + Sync>>) {
        self.drum_model = model;
    }

    pub fn set_bass_model(&mut self, model: Option<Arc<dyn BassModel + Send + Sync>>) {
        self.bass_model = model;
    }

    pub fn set_acoustic_model(&mut self, model: Option<Arc<dyn AcousticModel + Send + Sync>>) {
        self.acoustic_model = model;
    }

    pub fn set_fiddle_model(&mut self, model: Option<Arc<dyn FiddleModel + Send + Sync>>) {
        self.fiddle_model = model;
    }
}

impl Default for InstrumentModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}