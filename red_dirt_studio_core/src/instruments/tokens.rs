use serde::{Serialize, Deserialize};

// ============================================================================
// SHARED TOKEN TYPES (used by all instruments and apprentice)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SectionType {
    Intro,
    Verse1,
    Verse2,
    Chorus1,
    Chorus2,
    Bridge,
    Solo,
    Outro,
}

impl SectionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Intro => "INTRO",
            Self::Verse1 => "VERSE1",
            Self::Verse2 => "VERSE2",
            Self::Chorus1 => "CHORUS1",
            Self::Chorus2 => "CHORUS2",
            Self::Bridge => "BRIDGE",
            Self::Solo => "SOLO",
            Self::Outro => "OUTRO",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Feel {
    Straight,
    Shuffle,
    Train,
    HalfTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Energy {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Dom7,
    Sus2,
    Sus4,
    Add9,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordToken {
    pub degree: i8,              // -7..7 relative to key
    pub quality: ChordQuality,
    pub duration_beats: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSpec {
    pub section_type: SectionType,
    pub start_bar: u32,
    pub end_bar: u32,
    pub tempo_bpm: f32,
    pub feel: Feel,
    pub energy: Energy,
}

// ============================================================================
// DRUM TOKENS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrumStep {
    pub kick: bool,
    pub snare: bool,
    pub hat_closed: bool,
    pub hat_open: bool,
    pub ride: bool,
    pub tom_low: bool,
    pub tom_mid: bool,
    pub tom_high: bool,
    pub crash: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumBar {
    pub steps: [DrumStep; 16],  // 16 steps per bar (4/4)
    pub fill_flag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPlan {
    pub section: SectionSpec,
    pub bars: Vec<DrumBar>,
    pub pattern_id: String,       // e.g., "TRAIN_BEAT_01"
    pub variation: u8,
}

// ============================================================================
// BASS TOKENS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BassRole {
    Root,
    Third,
    Fifth,
    Octave,
    Passing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassNote {
    pub step_index: u8,          // 0..15 in 16-step grid
    pub degree_role: BassRole,
    pub length_steps: u8,         // 1..16
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassPlan {
    pub section: SectionSpec,
    pub notes: Vec<BassNote>,
    pub movement_template: String,  // e.g., "ROOT_3_5", "WALK_UP"
    pub density: f32,                // 0.0..1.0
}

// ============================================================================
// ACOUSTIC GUITAR TOKENS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrumDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrumIntensity {
    Soft,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrumEvent {
    pub step_index: u8,           // 0..15 in 16-step grid
    pub direction: StrumDirection,
    pub intensity: StrumIntensity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoicingShape {
    Open,
    Capo,
    Barre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoicingColor {
    Plain,
    Add2,
    Sus,
    Add9,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicingTag {
    pub shape: VoicingShape,
    pub color: VoicingColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticPlan {
    pub section: SectionSpec,
    pub strums: Vec<StrumEvent>,
    pub voicings: Vec<VoicingTag>,
    pub pattern_id: String,       // e.g., "COUNTRY_6_8_PATTERN_01"
}

// ============================================================================
// FIDDLE TOKENS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhraseType {
    Fill,
    Hook,
    SoloFrag,
    Pad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation {
    Sustain,
    Staccato,
    Slide,
    Bend,
    DoubleStop,
    Chop,
    Trill,
    Drone,
    Pizz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiddlePhrase {
    pub start_beat: f32,
    pub end_beat: f32,
    pub phrase_type: PhraseType,
    pub contour: Vec<i8>,         // relative pitch steps, e.g., [+2, -1, +2, 0]
    pub intensity: Energy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiddlePlan {
    pub section: SectionSpec,
    pub phrases: Vec<FiddlePhrase>,
    pub bowing_template: String,  // e.g., "SMOOTH", "ATTACK"
}

// ============================================================================
// SONG PLAN (what the apprentice oracle returns)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongPlan {
    pub key: String,               // e.g., "G", "D", "A"
    pub tempo_bpm: f32,
    pub sections: Vec<SectionSpec>,
    pub chords_per_section: Vec<Vec<ChordToken>>,
    pub drum_pattern_hints: Vec<String>,        // pattern IDs
    pub bass_movement_hints: Vec<String>,       // movement template IDs
    pub acoustic_pattern_hints: Vec<String>,    // pattern IDs
    pub style_strength: f32,                     // 0.0..1.0 (apprentice influence)
}

#[derive(Debug, Clone)]
pub struct InstrumentPlan {
    pub drum_plan: Option<DrumPlan>,
    pub bass_plan: Option<BassPlan>,
    pub acoustic_plan: Option<AcousticPlan>,
    pub fiddle_plan: Option<FiddlePlan>,
}
