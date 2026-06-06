use anyhow::Result;
use crate::instruments::tokens::*;

// ============================================================================
// MODEL TRAITS
// ============================================================================

pub trait InstrumentModel: Send + Sync {
    fn generate(&self, section: &SectionSpec, chords: &[ChordToken]) -> Result<Vec<u8>>;
    fn get_model_version(&self) -> String;
    fn get_style_confidence(&self) -> f32;
}

pub trait DrumModel: Send + Sync {
    fn generate_drums(&self, section: &SectionSpec) -> Result<DrumPlan>;
    fn get_version(&self) -> String;
    fn get_style_confidence(&self) -> f32;
}

pub trait BassModel: Send + Sync {
    fn generate_bass(&self, section: &SectionSpec, chords: &[ChordToken], drums: &DrumPlan) -> Result<BassPlan>;
    fn get_version(&self) -> String;
    fn get_style_confidence(&self) -> f32;
}

pub trait AcousticModel: Send + Sync {
    fn generate_acoustic(&self, section: &SectionSpec, chords: &[ChordToken]) -> Result<AcousticPlan>;
    fn get_version(&self) -> String;
    fn get_style_confidence(&self) -> f32;
}

pub trait FiddleModel: Send + Sync {
    fn generate_fiddle(&self, section: &SectionSpec, chords: &[ChordToken], vocal_map: Option<&[(f64, f64)]>) -> Result<FiddlePlan>;
    fn get_version(&self) -> String;
    fn get_style_confidence(&self) -> f32;
}

// ============================================================================
// MODEL READINESS TRACKING
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    Training,
    PerformanceReady,
    StyleReady,
    OwnerApproved,
    Deployed,
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub instrument: String,
    pub version: String,
    pub status: ModelStatus,
    pub performance_score: f32,          // 0.0..1.0
    pub style_confidence: f32,           // 0.0..1.0
    pub owner_approval_pending: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ModelMetadata {
    pub fn new(instrument: String, version: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            instrument,
            version,
            status: ModelStatus::Training,
            performance_score: 0.0,
            style_confidence: 0.0,
            owner_approval_pending: false,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn is_ready_for_deployment(&self) -> bool {
        self.status == ModelStatus::OwnerApproved
    }
}

// ============================================================================
// STUB MODELS FOR TESTING / DEMO
// ============================================================================

pub struct StubDrumModel {
    version: String,
}

impl StubDrumModel {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl DrumModel for StubDrumModel {
    fn generate_drums(&self, section: &SectionSpec) -> Result<DrumPlan> {
        let bars = vec![DrumBar {
            steps: [DrumStep {
                kick: true,
                snare: false,
                hat_closed: false,
                hat_open: false,
                ride: false,
                tom_low: false,
                tom_mid: false,
                tom_high: false,
                crash: false,
            }; 16],
            fill_flag: false,
        }];

        Ok(DrumPlan {
            section: section.clone(),
            bars,
            pattern_id: "STUB_PATTERN".to_string(),
            variation: 0,
        })
    }

    fn get_version(&self) -> String {
        self.version.clone()
    }

    fn get_style_confidence(&self) -> f32 {
        0.5
    }
}

pub struct StubBassModel {
    version: String,
}

impl StubBassModel {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl BassModel for StubBassModel {
    fn generate_bass(&self, section: &SectionSpec, _chords: &[ChordToken], _drums: &DrumPlan) -> Result<BassPlan> {
        Ok(BassPlan {
            section: section.clone(),
            notes: vec![],
            movement_template: "ROOT_5".to_string(),
            density: 0.5,
        })
    }

    fn get_version(&self) -> String {
        self.version.clone()
    }

    fn get_style_confidence(&self) -> f32 {
        0.5
    }
}

pub struct StubAcousticModel {
    version: String,
}

impl StubAcousticModel {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl AcousticModel for StubAcousticModel {
    fn generate_acoustic(&self, section: &SectionSpec, _chords: &[ChordToken]) -> Result<AcousticPlan> {
        Ok(AcousticPlan {
            section: section.clone(),
            strums: vec![],
            voicings: vec![],
            pattern_id: "STUB_PATTERN".to_string(),
        })
    }

    fn get_version(&self) -> String {
        self.version.clone()
    }

    fn get_style_confidence(&self) -> f32 {
        0.5
    }
}

pub struct StubFiddleModel {
    version: String,
}

impl StubFiddleModel {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl FiddleModel for StubFiddleModel {
    fn generate_fiddle(&self, section: &SectionSpec, _chords: &[ChordToken], _vocal_map: Option<&[(f64, f64)]>) -> Result<FiddlePlan> {
        Ok(FiddlePlan {
            section: section.clone(),
            phrases: vec![],
            bowing_template: "SMOOTH".to_string(),
        })
    }

    fn get_version(&self) -> String {
        self.version.clone()
    }

    fn get_style_confidence(&self) -> f32 {
        0.5
    }
}
