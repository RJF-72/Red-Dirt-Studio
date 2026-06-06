use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    Training,
    PerformanceReady,
    StyleReady,
    OwnerApproved,
    Deployed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub instrument: String,
    pub version_id: String,
    pub created_at: DateTime<Utc>,
    pub performance_score: f32,
    pub style_confidence: f32,
    pub owner_approved: bool,
    pub deployed: bool,
    pub ready_for_deployment: bool,
    pub model_path: PathBuf,
    pub metadata_path: PathBuf,
}

impl ModelVersion {
    pub fn status(&self) -> ModelStatus {
        if self.deployed {
            ModelStatus::Deployed
        } else if self.owner_approved {
            ModelStatus::OwnerApproved
        } else if self.style_confidence >= 0.85 {
            ModelStatus::StyleReady
        } else if self.performance_score >= 0.9 {
            ModelStatus::PerformanceReady
        } else {
            ModelStatus::Training
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    #[serde(default)]
    pub active: HashMap<String, String>,
    #[serde(default)]
    pub archived: HashMap<String, Vec<String>>,
    pub models_dir: PathBuf,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self {
            active: HashMap::new(),
            archived: HashMap::new(),
            models_dir: PathBuf::from("models"),
        }
    }
}

impl ModelRegistry {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let registry: Self = serde_json::from_str(&content)?;
        Ok(registry)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub struct ModelManager {
    versions: HashMap<String, Vec<ModelVersion>>,
    registry: ModelRegistry,
    owner_only: bool,
}

impl ModelManager {
    pub fn new(owner_only: bool) -> Self {
        let registry = ModelRegistry::default();
        Self {
            versions: HashMap::new(),
            registry,
            owner_only,
        }
    }

    pub fn load_registry(&mut self, path: &PathBuf) -> Result<()> {
        self.registry = ModelRegistry::load(path)?;
        Ok(())
    }

    pub fn save_registry(&self, path: &PathBuf) -> Result<()> {
        self.registry.save(path)
    }

    /// Register a new model version after readiness evaluation
    pub fn register_model(&mut self, model: ModelVersion) -> Result<()> {
        let instrument = model.instrument.clone();
        self.versions
            .entry(model.instrument.clone())
            .or_insert_with(Vec::new)
            .push(model.clone());
        
        // Archive old active version
        let registry = &mut self.registry;
        if let Some(old_version) = registry.active.get(&instrument).cloned() {
            registry
                .archived
                .entry(instrument.clone())
                .or_insert_with(Vec::new)
                .push(old_version);
        }
        registry.active.insert(instrument, model.version_id.clone());
        
        Ok(())
    }

    /// Check if a model is ready for owner approval
    pub fn has_pending_approval(&self, instrument: &str) -> bool {
        if let Some(versions) = self.versions.get(instrument) {
            versions
                .iter()
                .any(|v| v.ready_for_deployment && !v.owner_approved && !v.deployed)
        } else {
            false
        }
    }

    /// Get all instruments with pending approvals
    pub fn get_pending_instruments(&self) -> Vec<String> {
        self.versions
            .iter()
            .filter(|(instrument, _)| self.has_pending_approval(instrument))
            .map(|(instrument, _)| instrument.clone())
            .collect()
    }

    /// Owner approves and auto-swaps a model (owner-only)
    pub fn owner_approve_and_deploy(&mut self, instrument: &str, version_id: String) -> Result<String> {
        if !self.owner_only {
            return Err(anyhow::anyhow!(
                "This operation is only available in Owner's Edition"
            ));
        }

        let versions = self
            .versions
            .get_mut(instrument)
            .ok_or_else(|| anyhow::anyhow!("No versions for instrument: {}", instrument))?;

        let model = versions
            .iter_mut()
            .find(|v| v.version_id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version_id))?;

        if !model.ready_for_deployment {
            return Err(anyhow::anyhow!(
                "Model not ready for deployment. Performance: {}, Style: {}",
                model.performance_score,
                model.style_confidence
            ));
        }

        model.owner_approved = true;
        model.deployed = true;
        // status is computed via status() method

        Ok(format!("{}@{} deployed successfully", instrument, version_id))
    }

    /// Mark model as deployed and update registry
    pub fn deploy_model(&mut self, instrument: &str, version_id: &str) -> Result<()> {
        if let Some(versions) = self.versions.get_mut(instrument) {
            if let Some(model) = versions.iter_mut().find(|v| &v.version_id == version_id) {
                model.deployed = true;
                model.owner_approved = true;
            }
        }
        Ok(())
    }

    /// Get currently active model version for an instrument
    pub fn get_active_version(&self, instrument: &str) -> Option<String> {
        self.registry.active.get(instrument).cloned()
    }

    /// Get all versions for an instrument
    pub fn get_versions(&self, instrument: &str) -> Option<Vec<ModelVersion>> {
        self.versions.get(instrument).cloned()
    }

    /// Rollback to a previous version (owner-only)
    pub fn rollback(&mut self, instrument: &str, version_id: &str) -> Result<()> {
        if !self.owner_only {
            return Err(anyhow::anyhow!(
                "This operation is only available in Owner's Edition"
            ));
        }

        if let Some(current) = self.registry.active.get(instrument).cloned() {
            self.registry
                .archived
                .entry(instrument.to_string())
                .or_insert_with(Vec::new)
                .push(current);
        }
        self.registry.active.insert(instrument.to_string(), version_id.to_string());

        Ok(())
    }

    /// Get the next instrument to train (owner workflow)
    pub fn suggest_next_instrument(&self) -> Option<String> {
        let planned_instruments = vec![
            "drums", "bass", "acoustic", "fiddle", "electric_gtr", "b3",
            "pedal_steel", "mandolin", "banjo", "harmonica", "electric_bass",
            "resonator", "background_vocals"
        ];

        for instrument in planned_instruments {
            let has_model = self.versions.get(instrument)
                .map(|v| v.iter().any(|m| m.deployed))
                .unwrap_or(false);
            if !has_model {
                return Some(instrument.to_string());
            }
        }

        None
    }
}