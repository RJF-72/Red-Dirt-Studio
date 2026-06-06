use anyhow::Result;
use crate::instruments::tokens::*;

// ============================================================================
// DRIVER TRAITS
// ============================================================================

pub trait InstrumentDriver: Send + Sync {
    fn expand_pattern(&self, pattern_id: &str, variation: u8, section: &SectionSpec) -> Result<Vec<u8>>;
    fn humanize_events(&self, events: &[u8]) -> Result<Vec<u8>>;
    fn validate_range(&self, note: u8) -> bool;
}

pub trait DrumDriver: Send + Sync {
    fn expand_drum_bar(&self, bar: &DrumBar, section: &SectionSpec) -> Result<Vec<u8>>;
    fn apply_humanization(&self, events: &[u8], swing: f32) -> Result<Vec<u8>>;
}

pub trait BassDriver: Send + Sync {
    fn expand_bass_note(&self, note: &BassNote, chord: &ChordToken, section: &SectionSpec) -> Result<Vec<u8>>;
    fn validate_voice_leading(&self, prev_note: u8, next_note: u8) -> bool;
}

pub trait AcousticDriver: Send + Sync {
    fn expand_strum_pattern(&self, strum: &StrumEvent, voicing: &VoicingTag, chord: &ChordToken) -> Result<Vec<u8>>;
    fn apply_pick_noise(&self, events: &[u8], intensity: StrumIntensity) -> Result<Vec<u8>>;
}

pub trait FiddleDriver: Send + Sync {
    fn expand_phrase(&self, phrase: &FiddlePhrase, chord: &ChordToken, section: &SectionSpec) -> Result<Vec<u8>>;
    fn apply_bowing(&self, events: &[u8], bowing_template: &str) -> Result<Vec<u8>>;
}

// ============================================================================
// BASIC DRIVER IMPLEMENTATIONS
// ============================================================================

pub struct BasicDrumDriver {
    patterns: std::collections::HashMap<String, Vec<u8>>,
}

impl BasicDrumDriver {
    pub fn new() -> Self {
        Self {
            patterns: std::collections::HashMap::new(),
        }
    }

    pub fn register_pattern(&mut self, id: String, pattern: Vec<u8>) {
        self.patterns.insert(id, pattern);
    }
}

impl DrumDriver for BasicDrumDriver {
    fn expand_drum_bar(&self, _bar: &DrumBar, _section: &SectionSpec) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn apply_humanization(&self, events: &[u8], _swing: f32) -> Result<Vec<u8>> {
        Ok(events.to_vec())
    }
}

pub struct BasicBassDriver;

impl BasicBassDriver {
    pub fn new() -> Self {
        Self
    }
}

impl BassDriver for BasicBassDriver {
    fn expand_bass_note(&self, _note: &BassNote, _chord: &ChordToken, _section: &SectionSpec) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn validate_voice_leading(&self, _prev_note: u8, _next_note: u8) -> bool {
        true
    }
}

pub struct BasicAcousticDriver;

impl AcousticDriver for BasicAcousticDriver {
    fn expand_strum_pattern(&self, _strum: &StrumEvent, _voicing: &VoicingTag, _chord: &ChordToken) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn apply_pick_noise(&self, events: &[u8], _intensity: StrumIntensity) -> Result<Vec<u8>> {
        Ok(events.to_vec())
    }
}

pub struct BasicFiddleDriver;

impl FiddleDriver for BasicFiddleDriver {
    fn expand_phrase(&self, _phrase: &FiddlePhrase, _chord: &ChordToken, _section: &SectionSpec) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn apply_bowing(&self, events: &[u8], _bowing_template: &str) -> Result<Vec<u8>> {
        Ok(events.to_vec())
    }
}
