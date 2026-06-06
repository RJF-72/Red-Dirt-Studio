use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use super::{SFZInstrument, SFZGroup, SFZRegion};

pub struct SFZParser;

impl SFZParser {
    pub fn parse(path: &Path) -> Result<SFZInstrument> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read SFZ file: {:?}", path))?;
        
        let parent_dir = path.parent().unwrap_or(Path::new(""));
        let mut groups = Vec::new();
        let mut current_group = SFZGroup {
            regions: Vec::new(),
            volume: 0.0,
            pan: 0.0,
        };
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            
            if line.starts_with("<group>") {
                if !current_group.regions.is_empty() {
                    groups.push(current_group.clone());
                }
                current_group = SFZGroup {
                    regions: Vec::new(),
                    volume: 0.0,
                    pan: 0.0,
                };
                continue;
            }
            
            if line.starts_with('<') {
                continue;
            }
            
            if line.starts_with("sample=") {
                let sample_path = line.strip_prefix("sample=").unwrap().trim();
                let region = SFZRegion {
                    sample: parent_dir.join(sample_path),
                    lokey: 0,
                    hikey: 127,
                    pitch_keycenter: 60,
                    volume: current_group.volume,
                    pan: current_group.pan,
                    loop_mode: None,
                    loop_start: None,
                    loop_end: None,
                };
                
                current_group.regions.push(region);
            } else {
                for part in line.split_whitespace() {
                    if let Some(eq_pos) = part.find('=') {
                        let key = &part[..eq_pos];
                        let value = &part[eq_pos + 1..];
                        
                        match key {
                            "lokey" => if let Some(region) = current_group.regions.last_mut() {
                                region.lokey = value.parse().unwrap_or(0);
                            },
                            "hikey" => if let Some(region) = current_group.regions.last_mut() {
                                region.hikey = value.parse().unwrap_or(127);
                            },
                            "pitch_keycenter" => if let Some(region) = current_group.regions.last_mut() {
                                region.pitch_keycenter = value.parse().unwrap_or(60);
                            },
                            "volume" => {
                                let vol_db: f32 = value.parse().unwrap_or(0.0);
                                let vol_linear = 10.0_f32.powf(vol_db / 20.0);
                                if let Some(region) = current_group.regions.last_mut() {
                                    region.volume = vol_linear;
                                }
                                current_group.volume = vol_linear;
                            }
                            "pan" => {
                                let pan_val: f32 = value.parse().unwrap_or(0.0);
                                if let Some(region) = current_group.regions.last_mut() {
                                    region.pan = pan_val / 100.0;
                                }
                                current_group.pan = pan_val / 100.0;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        if !current_group.regions.is_empty() {
            groups.push(current_group);
        }
        
        Ok(SFZInstrument {
            path: path.to_path_buf(),
            groups,
            default_volume: 0.8,
            default_pan: 0.0,
        })
    }
}