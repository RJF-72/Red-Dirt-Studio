use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFZRegion {
    pub sample_path: PathBuf,
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub group: String,
    pub articulation: String,
    pub instrument: String,
    pub extra: HashMap<String, String>,
}