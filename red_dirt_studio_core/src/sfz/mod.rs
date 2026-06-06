pub mod parser;
pub mod region;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFZRegion {
    pub sample: PathBuf,
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub volume: f32,
    pub pan: f32,
    pub loop_mode: Option<String>,
    pub loop_start: Option<usize>,
    pub loop_end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFZGroup {
    pub regions: Vec<SFZRegion>,
    pub volume: f32,
    pub pan: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFZInstrument {
    pub path: PathBuf,
    pub groups: Vec<SFZGroup>,
    pub default_volume: f32,
    pub default_pan: f32,
}