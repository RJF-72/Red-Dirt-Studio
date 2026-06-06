use crate::instruments::{DrumPlan, BassPlan, AcousticPlan, FiddlePlan};
use crate::apprentice::SongDataset;
use crate::model_manager::ModelStatus;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub instrument: String,
    pub version: String,
    pub performance_score: f32,
    pub style_confidence: f32,
    pub overall_readiness: f32,
    pub is_ready: bool,
    pub issues: Vec<String>,
    pub status: ModelStatus,
}

pub struct ModelReadinessEngine;

impl ModelReadinessEngine {
    /// Evaluate if a drum model is ready for production
    pub fn evaluate_drum_model(
        plan: &DrumPlan,
        apprentice_dataset: &SongDataset,
    ) -> Result<ReadinessReport> {
        let mut issues = Vec::new();

        // Test 1: Pattern structure (no empty bars)
        if plan.bars.is_empty() {
            issues.push("No drum bars generated".to_string());
        }

        // Test 2: Timing correctness (all bars have 16 steps)
        for (i, bar) in plan.bars.iter().enumerate() {
            if bar.steps.len() != 16 {
                issues.push(format!("Bar {} has {} steps, expected 16", i, bar.steps.len()));
            }
        }

        let performance_score = if issues.is_empty() { 0.95 } else { 0.5 };
        let style_confidence = Self::evaluate_style_fit("drums", apprentice_dataset);
        let overall_readiness = (performance_score + style_confidence) / 2.0;
        let is_ready = overall_readiness >= 0.85;

        Ok(ReadinessReport {
            instrument: "drums".to_string(),
            version: plan.pattern_id.clone(),
            performance_score,
            style_confidence,
            overall_readiness,
            is_ready,
            issues,
            status: if is_ready { ModelStatus::StyleReady } else { ModelStatus::Training },
        })
    }

    /// Evaluate if a bass model is ready for production
    pub fn evaluate_bass_model(
        plan: &BassPlan,
        apprentice_dataset: &SongDataset,
    ) -> Result<ReadinessReport> {
        let mut issues = Vec::new();

        // Test 1: Notes in valid range
        for note in &plan.notes {
            if note.step_index > 15 {
                issues.push(format!("Bass note at invalid step: {}", note.step_index));
            }
        }

        // Test 2: Density reasonable
        if plan.density < 0.1 || plan.density > 1.0 {
            issues.push(format!("Bass density out of range: {}", plan.density));
        }

        let performance_score = if issues.is_empty() { 0.95 } else { 0.5 };
        let style_confidence = Self::evaluate_style_fit("bass", apprentice_dataset);
        let overall_readiness = (performance_score + style_confidence) / 2.0;
        let is_ready = overall_readiness >= 0.85;

        Ok(ReadinessReport {
            instrument: "bass".to_string(),
            version: plan.movement_template.clone(),
            performance_score,
            style_confidence,
            overall_readiness,
            is_ready,
            issues,
            status: if is_ready { ModelStatus::StyleReady } else { ModelStatus::Training },
        })
    }

    /// Evaluate if an acoustic guitar model is ready
    pub fn evaluate_acoustic_model(
        plan: &AcousticPlan,
        apprentice_dataset: &SongDataset,
    ) -> Result<ReadinessReport> {
        let mut issues = Vec::new();

        // Test 1: Strum events in valid range
        for strum in &plan.strums {
            if strum.step_index > 15 {
                issues.push(format!("Strum at invalid step: {}", strum.step_index));
            }
        }

        // Test 2: Voicings count matches strums (roughly)
        if !plan.strums.is_empty() && plan.voicings.len() < plan.strums.len() / 2 {
            issues.push("Too few voicings for strum events".to_string());
        }

        let performance_score = if issues.is_empty() { 0.95 } else { 0.5 };
        let style_confidence = Self::evaluate_style_fit("acoustic", apprentice_dataset);
        let overall_readiness = (performance_score + style_confidence) / 2.0;
        let is_ready = overall_readiness >= 0.85;

        Ok(ReadinessReport {
            instrument: "acoustic".to_string(),
            version: plan.pattern_id.clone(),
            performance_score,
            style_confidence,
            overall_readiness,
            is_ready,
            issues,
            status: if is_ready { ModelStatus::StyleReady } else { ModelStatus::Training },
        })
    }

    /// Evaluate if a fiddle model is ready
    pub fn evaluate_fiddle_model(
        plan: &FiddlePlan,
        apprentice_dataset: &SongDataset,
    ) -> Result<ReadinessReport> {
        let mut issues = Vec::new();

        // Test 1: Phrases have valid boundaries
        for phrase in &plan.phrases {
            if phrase.end_beat <= phrase.start_beat {
                issues.push(format!(
                    "Invalid phrase timing: start={}, end={}",
                    phrase.start_beat, phrase.end_beat
                ));
            }
        }

        // Test 2: Phrase contours not empty
        let mut empty_contours = 0;
        for phrase in &plan.phrases {
            if phrase.contour.is_empty() {
                empty_contours += 1;
            }
        }
        if empty_contours > plan.phrases.len() / 2 {
            issues.push(format!("{} phrases have empty contours", empty_contours));
        }

        let performance_score = if issues.is_empty() { 0.95 } else { 0.5 };
        let style_confidence = Self::evaluate_style_fit("fiddle", apprentice_dataset);
        let overall_readiness = (performance_score + style_confidence) / 2.0;
        let is_ready = overall_readiness >= 0.85;

        Ok(ReadinessReport {
            instrument: "fiddle".to_string(),
            version: plan.bowing_template.clone(),
            performance_score,
            style_confidence,
            overall_readiness,
            is_ready,
            issues,
            status: if is_ready { ModelStatus::StyleReady } else { ModelStatus::Training },
        })
    }

    fn evaluate_style_fit(_instrument: &str, dataset: &SongDataset) -> f32 {
        // Placeholder: check if dataset has enough examples of this instrument
        let songs = dataset.get_all_songs();
        if songs.is_empty() {
            return 0.5;  // insufficient data
        }

        // Simplistic: assume 60% confidence baseline, increase with dataset size
        let confidence_boost = (songs.len() as f32 / 100.0).min(0.35);
        (0.60 + confidence_boost).min(0.95)
    }
}
