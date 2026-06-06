use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Serialize, Deserialize};

use super::sfz_parser::{SFZParser, SFZRegion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledSegment {
    pub audio_path: PathBuf,
    pub start_sec: f64,
    pub end_sec: f64,
    pub pitch: u8,
    pub velocity: u8,
    pub articulation: String,
    pub group: String,
    pub instrument: String,
    pub sfz_region: SFZRegion,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FLACAligner {
    flac_root: PathBuf,
}

impl FLACAligner {
    pub fn new(flac_root: &Path) -> Self {
        Self {
            flac_root: flac_root.to_path_buf(),
        }
    }
    
    pub fn align_region(&self, region: &SFZRegion) -> Vec<LabeledSegment> {
        let base = region.sample_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let flac_path = self.flac_root.join(format!("{}.flac", base));
        
        vec![LabeledSegment {
            audio_path: flac_path,
            start_sec: 0.0,
            end_sec: 1.0,
            pitch: region.pitch_keycenter,
            velocity: (region.lovel + region.hivel) / 2,
            articulation: region.articulation.clone(),
            group: region.group.clone(),
            instrument: region.instrument.clone(),
            sfz_region: region.clone(),
            meta: serde_json::json!({"source": "auto_align_stub"}),
        }]
    }
}

#[derive(Debug, Clone)]
pub struct DatasetBuilder {
    sfz_parser: SFZParser,
    flac_aligner: FLACAligner,
}

impl DatasetBuilder {
    pub fn new(sfz_parser: SFZParser, flac_aligner: FLACAligner) -> Self {
        Self {
            sfz_parser,
            flac_aligner,
        }
    }
    
    pub fn build_dataset(&self) -> Result<Vec<LabeledSegment>> {
        let mut parser = self.sfz_parser.clone();
        let regions = parser.parse()?;
        
        let mut dataset = Vec::new();
        for region in regions {
            let segments = self.flac_aligner.align_region(&region);
            dataset.extend(segments);
        }
        
        Ok(dataset)
    }
}