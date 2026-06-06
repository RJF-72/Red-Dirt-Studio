use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

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

#[derive(Debug, Clone)]
pub struct SFZParser {
    sfz_path: PathBuf,
    root_dir: PathBuf,
}

impl SFZParser {
    pub fn new(sfz_path: &Path, root_dir: &Path) -> Self {
        Self {
            sfz_path: sfz_path.to_path_buf(),
            root_dir: root_dir.to_path_buf(),
        }
    }
    
    pub fn parse(&mut self) -> Result<Vec<SFZRegion>> {
        let content = fs::read_to_string(&self.sfz_path)
            .with_context(|| format!("Failed to read SFZ file: {:?}", self.sfz_path))?;
        
        let mut regions = Vec::new();
        let mut current_region = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            
            if line.starts_with("<region>") {
                if !current_region.is_empty() {
                    if let Some(region) = self.commit_region(&current_region) {
                        regions.push(region);
                    }
                    current_region.clear();
                }
                continue;
            }
            
            if line.starts_with('<') {
                continue;
            }
            
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                current_region.insert(key.to_string(), value.to_string());
            }
        }
        
        if !current_region.is_empty() {
            if let Some(region) = self.commit_region(&current_region) {
                regions.push(region);
            }
        }
        
        Ok(regions)
    }
    
    fn commit_region(&self, meta: &HashMap<String, String>) -> Option<SFZRegion> {
        let sample_rel = meta.get("sample")?;
        let sample_path = self.root_dir.join(sample_rel);
        
        let lokey = meta.get("lokey")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let hikey = meta.get("hikey")
            .and_then(|v| v.parse().ok())
            .unwrap_or(127);
        let pitch_keycenter = meta.get("pitch_keycenter")
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let lovel = meta.get("lovel")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let hivel = meta.get("hivel")
            .and_then(|v| v.parse().ok())
            .unwrap_or(127);
        
        let group = meta.get("group")
            .or_else(|| meta.get("group"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".to_string());
        
        let articulation = meta.get("articulation")
            .or_else(|| meta.get("label"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "sustain".to_string());
        
        let instrument = meta.get("instrument")
            .or_else(|| meta.get("instr"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "band".to_string());
        
        let mut extra = HashMap::new();
        for (k, v) in meta {
            if !["sample", "lokey", "hikey", "pitch_keycenter", "lovel", "hivel", "group", "articulation", "label", "instrument", "instr"]
                .contains(&k.as_str()) {
                extra.insert(k.clone(), v.clone());
            }
        }
        
        Some(SFZRegion {
            sample_path,
            lokey,
            hikey,
            pitch_keycenter,
            lovel,
            hivel,
            group,
            articulation,
            instrument,
            extra,
        })
    }
}