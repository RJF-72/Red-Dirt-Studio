use anyhow::Result;
use red_dirt_studio_core::apprentice::{RustApprentice, SongLog, ApprenticeConfig};
use red_dirt_studio_core::model_manager::ModelManager;
use red_dirt_studio_core::readiness::ModelReadinessEngine;
use red_dirt_studio_core::instruments::InstrumentModelRegistry;
use red_dirt_studio_core::instruments::tokens::InstrumentPlan;
use red_dirt_studio_core::training::InstrumentBuilder;
use chrono::Utc;
use uuid::Uuid;
use crate::pipeline::{SongInput, StemResult, generate_arrangement};

/// Apprentice-integrated pipeline that learns from each generated song
pub struct ApprenticelyIntegratedPipeline {
    pub apprentice: RustApprentice,
    pub model_manager: ModelManager,
    model_registry: InstrumentModelRegistry,
}

impl ApprenticelyIntegratedPipeline {
    pub fn new(owner_only: bool) -> Self {
        let config = ApprenticeConfig::default();
        let apprentice = RustApprentice::new(config);
        let model_manager = ModelManager::new(owner_only);
        let model_registry = InstrumentModelRegistry::new();

        Self {
            apprentice,
            model_manager,
            model_registry,
        }
    }

    /// Generate arrangement with apprentice learning integration
    pub async fn generate_with_apprentice(
        &mut self,
        input: &SongInput,
    ) -> Result<(red_dirt_studio_core::models::BandDecision, Vec<StemResult>)> {
        // BEFORE generation: get apprentice suggestions (informational)
        let style_strength = 0.7;  // default, can be parameterized
        let _suggested_plan = self
            .apprentice
            .suggest_song_plan(&input.description, Some(input.key.clone()), Some(input.bpm as f32), style_strength)
            .ok();  // Log errors but don't block

        // Generate the arrangement (standard pipeline)
        let (decision, stems) = generate_arrangement(input).await?;

        // AFTER generation: log the song for apprentice to learn
        let song_id = Uuid::new_v4().to_string();
        let song_log = SongLog {
            id: song_id.clone(),
            timestamp: Utc::now(),
            prompt: input.description.clone(),
            key: input.key.clone(),
            tempo_bpm: input.bpm as f32,
            sections: input.sections.iter().map(|(name, _)| name.clone()).collect(),
            chords: decision.chord_progression.clone(),
            drum_pattern_ids: vec![],  // Would be populated from generation details
            bass_movement_ids: vec![],
            acoustic_pattern_ids: vec![],
            arrangement_density: calculate_density(&input.instruments),
            user_rating: None,  // User can set this later
            style_alignment: 0.7,  // Default; apprentice refines over time
        };

        // Store the song in apprentice dataset
        self.apprentice.observe_song(song_log)?;

        // Check for pending model approvals and suggest next instrument
        let _next_instrument = self.model_manager.suggest_next_instrument();

        Ok((decision, stems))
    }

    /// Check for pending model approvals
    pub fn check_model_readiness(&self) -> Vec<String> {
        self.model_manager.get_pending_instruments()
    }

    /// Owner approves and deploys a model
    pub fn owner_approve_and_deploy(&mut self, instrument: &str, version_id: String) -> Result<String> {
        self.model_manager
            .owner_approve_and_deploy(instrument, version_id)
    }

    /// Get the next instrument to train
    pub fn suggest_next_instrument(&self) -> Option<String> {
        self.model_manager.suggest_next_instrument()
    }

    /// Run readiness evaluation on a model output
    pub fn evaluate_model_readiness(
        &self,
        instrument: &str,
        plan: &InstrumentPlan,
    ) -> Result<red_dirt_studio_core::readiness::ReadinessReport> {
        // This would be called after model generation to check if ready
        let dataset = self.apprentice.dataset.lock().unwrap();
        
        match instrument {
            "drums" => plan.drum_plan.as_ref()
                .map(|p| ModelReadinessEngine::evaluate_drum_model(p, &dataset))
                .unwrap_or(Err(anyhow::anyhow!("No drum plan in InstrumentPlan"))),
            "bass" => plan.bass_plan.as_ref()
                .map(|p| ModelReadinessEngine::evaluate_bass_model(p, &dataset))
                .unwrap_or(Err(anyhow::anyhow!("No bass plan in InstrumentPlan"))),
            "acoustic" => plan.acoustic_plan.as_ref()
                .map(|p| ModelReadinessEngine::evaluate_acoustic_model(p, &dataset))
                .unwrap_or(Err(anyhow::anyhow!("No acoustic plan in InstrumentPlan"))),
            "fiddle" => plan.fiddle_plan.as_ref()
                .map(|p| ModelReadinessEngine::evaluate_fiddle_model(p, &dataset))
                .unwrap_or(Err(anyhow::anyhow!("No fiddle plan in InstrumentPlan"))),
            _ => Err(anyhow::anyhow!("Unknown instrument: {}", instrument)),
        }
    }

    /// Create a new instrument builder for training
    pub fn create_instrument_builder(instrument_type: &str) -> red_dirt_studio_core::training::TrainingConfig {
        InstrumentBuilder::new(instrument_type).build()
    }
}

/// Helper to calculate arrangement density
fn calculate_density(instruments: &[String]) -> f32 {
    // Max realistic instruments is ~12, so density = count / 12
    (instruments.len() as f32 / 12.0).min(1.0)
}

impl Default for ApprenticelyIntegratedPipeline {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Helper to rate a generated song (called by user later)
pub fn rate_song(_apprentice: &mut RustApprentice, _song_id: &str, _rating: f32) -> Result<()> {
    // This would update the song rating in the dataset
    // For now, a placeholder for future enhancement
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_calculation() {
        assert_eq!(calculate_density(&[]), 0.0);
        assert_eq!(calculate_density(&["drums".to_string()]), 1.0 / 12.0);
        assert_eq!(calculate_density(&vec!["drums".to_string(); 12]), 1.0);
        assert!(calculate_density(&vec!["drums".to_string(); 20]) <= 1.0);
    }
}
