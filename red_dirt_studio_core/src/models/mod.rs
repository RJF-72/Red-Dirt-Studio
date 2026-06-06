pub mod band_brain;
pub mod qwen_audio;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandDecision {
    pub chord_progression: Vec<String>,
    pub bpm: f64,
    pub bars: usize,
    pub instruments: Vec<String>,
    pub phrase_structure: HashMap<String, PhraseInfo>,
    pub style_notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhraseInfo {
    pub bars: usize,
    pub feel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillIdea {
    pub bar: usize,
    pub instrument: String,
    pub notes: String,
    pub rhythm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverEvent {
    pub time_sec: f64,
    pub pitch: u8,
    pub velocity: u8,
    pub articulation: String,
    pub group: String,
    pub instrument: String,
    pub chord: Option<String>,
    pub meta: serde_json::Value,
}