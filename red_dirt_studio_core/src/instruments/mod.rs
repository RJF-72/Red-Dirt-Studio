pub mod tokens;
pub mod drivers;
pub mod models;
pub mod auto_swap;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub use tokens::*;
pub use drivers::*;
pub use models::*;
pub use auto_swap::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub category: String,
    pub sfz_path: PathBuf,
    pub flac_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentRack {
    pub instruments: Vec<Instrument>,
}

impl InstrumentRack {
    pub fn new() -> Self {
        Self {
            instruments: Vec::new(),
        }
    }
    
    pub fn add(&mut self, instrument: Instrument) {
        self.instruments.push(instrument);
    }
    
    pub fn get(&self, name: &str) -> Option<&Instrument> {
        self.instruments.iter().find(|i| i.name == name)
    }
}

impl Default for InstrumentRack {
    fn default() -> Self {
        Self::new()
    }
}