use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentLibrary {
    pub instruments: HashMap<String, InstrumentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub name: String,
    pub category: String,
    pub sfz_files: Vec<PathBuf>,
    pub flac_root: PathBuf,
    pub articulations: Vec<String>,
    pub pitch_range: (u8, u8),
    pub velocity_layers: usize,
}

impl InstrumentLibrary {
    pub fn new() -> Self {
        Self {
            instruments: HashMap::new(),
        }
    }
    
    pub fn add_instrument(&mut self, info: InstrumentInfo) {
        self.instruments.insert(info.name.clone(), info);
    }
    
    pub fn get(&self, name: &str) -> Option<&InstrumentInfo> {
        self.instruments.get(name)
    }
    
    pub fn list_instruments(&self) -> Vec<&str> {
        self.instruments.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for InstrumentLibrary {
    fn default() -> Self {
        Self::new()
    }
}