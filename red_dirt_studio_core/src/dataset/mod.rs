pub mod builder;
pub mod instrument_library;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentMetadata {
    pub name: String,
    pub category: String,
    pub sfz_path: PathBuf,
    pub flac_root: PathBuf,
    pub sample_count: usize,
    pub avg_duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    pub instruments: Vec<InstrumentMetadata>,
    pub sample_rate: u32,
    pub max_duration: f64,
    pub fft_size: usize,
    pub hop_length: usize,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            instruments: Vec::new(),
            sample_rate: 48000,
            max_duration: 10.0,
            fft_size: 2048,
            hop_length: 512,
        }
    }
}