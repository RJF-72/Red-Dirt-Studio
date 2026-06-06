use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use walkdir::WalkDir;
use anyhow::Result;
use tracing::{info, warn};
use super::{DatasetConfig, InstrumentMetadata};

// Curated instrument mapping for Red Dirt Rhythm Studio
pub const STARTER_INSTRUMENTS: &[(&str, &[&str])] = &[
    // FIDDLE
    ("fiddle", &[
        "ethnic_(gm_)_fiddle_americana",
        "ethnic_(gm_)_fiddle_country",
        "ethnic_(gm_)_fiddle_country_folk",
        "playable_realtracks_ethnic_111_fiddle_americana_alt",
        "playable_realtracks_ethnic_111_fiddle_country_alt",
    ]),
    
    // PEDAL STEEL
    ("pedal_steel", &[
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_country",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_vintage_country",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_country_alt",
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_steel_distortion_alt",
    ]),
    
    // BARITONE ACOUSTIC
    ("baritone_acoustic_guitar", &[
        "acoustic_guitars_etc_(gm_)_guitar_baritone_acoustic_americana",
        "playable_realtracks_acoustic_guitars_etc_026_guitar_baritone_acoustic_americana_alt",
    ]),
    
    // ACOUSTIC GUITAR
    ("acoustic_guitar", &[
        "guitar_finger_picked_acoustic",
        "guitar_warm_picked_acoustic",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_campfire",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_americana",
        "playable_realtracks_acoustic_guitars_etc_0",
    ]),
    
    // RESONATOR/DOBRO
    ("resonator", &[
        "acoustic_guitars_etc_(gm_)_guitar_resonator_dobro",
        "acoustic_guitars_etc_(gm_)_guitar_resonator_slide",
        "playable_realtracks_acoustic_guitars_etc_resonator_alt",
    ]),
    
    // DRUMS (Country & Worship)
    ("country_drums", &[
        "drums_(gm_)_drums_country",
        "drums_(gm_)_drums_country_train_beat",
        "drums_(gm_)_drums_country_shuffle",
        "playable_realtracks_drums_country_alt",
    ]),
    
    ("worship_drums", &[
        "drums_(gm_)_drums_worship",
        "drums_(gm_)_drums_worship_ballad",
        "drums_(gm_)_drums_worship_anthem",
        "playable_realtracks_drums_worship_alt",
    ]),
    
    // MANDOLIN
    ("mandolin", &[
        "acoustic_guitars_etc_(gm_)_mandolin_bluegrass",
        "acoustic_guitars_etc_(gm_)_mandolin_tremolo",
        "playable_realtracks_acoustic_guitars_etc_mandolin_alt",
    ]),
    
    // BANJO
    ("banjo", &[
        "acoustic_guitars_etc_(gm_)_banjo_bluegrass",
        "acoustic_guitars_etc_(gm_)_banjo_old_time",
        "playable_realtracks_acoustic_guitars_etc_banjo_alt",
    ]),
    
    // HARMONICA
    ("harmonica", &[
        "wind_(gm_)_harmonica_country",
        "wind_(gm_)_harmonica_blues",
        "playable_realtracks_wind_harmonica_alt",
    ]),
    
    // KEYBOARD & PIANO
    ("keyboard", &[
        "keyboard_(gm_)_keyboard_country",
        "keyboard_(gm_)_keyboard_worship",
        "playable_realtracks_keyboard_alt",
    ]),
    
    ("piano", &[
        "acoustic_grand_piano",
        "piano_(gm_)_piano_country",
        "piano_(gm_)_piano_ballad",
        "playable_realtracks_piano_country_alt",
    ]),
    
    // BASS
    ("acoustic_bass", &[
        "bass_(gm_)_bass_acoustic_upright",
        "bass_(gm_)_bass_acoustic_country",
        "playable_realtracks_bass_acoustic_alt",
    ]),
    
    ("electric_bass", &[
        "bass_(gm_)_bass_electric",
        "bass_(gm_)_bass_electric_country",
        "bass_(gm_)_bass_electric_finger",
        "playable_realtracks_bass_electric_alt",
    ]),
    
    // SAXOPHONE
    ("alto_sax", &[
        "wind_(gm_)_sax_alto",
        "wind_(gm_)_sax_alto_country",
        "playable_realtracks_wind_sax_alto_alt",
    ]),
    
    ("tenor_sax", &[
        "wind_(gm_)_sax_tenor",
        "wind_(gm_)_sax_tenor_ballad",
        "playable_realtracks_wind_sax_tenor_alt",
    ]),
];

pub struct DatasetBuilder {
    rdrs_root: PathBuf,
    output_dir: PathBuf,
    instruments: HashMap<String, Vec<PathBuf>>,
}

impl DatasetBuilder {
    pub fn new(rdrs_root: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            rdrs_root,
            output_dir,
            instruments: HashMap::new(),
        }
    }
    
    pub fn scan_rdrs(&mut self) -> Result<()> {
        info!("Scanning RDRS directory: {:?}", self.rdrs_root);
        
        for (instrument, patterns) in STARTER_INSTRUMENTS {
            let mut matches = Vec::new();
            
            for entry in WalkDir::new(&self.rdrs_root)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("sfz") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    for pattern in *patterns {
                        if name.contains(pattern) {
                            matches.push(path.to_path_buf());
                            break;
                        }
                    }
                }
            }
            
            if !matches.is_empty() {
                info!("Found {} SFZ files for {}", matches.len(), instrument);
                self.instruments.insert(instrument.to_string(), matches);
            } else {
                warn!("No SFZ files found for {}", instrument);
            }
        }
        
        Ok(())
    }
    
    pub fn build_aligned_dataset(&self) -> Result<DatasetConfig> {
        fs::create_dir_all(&self.output_dir)?;
        
        let mut config = DatasetConfig::default();
        
        for (instrument, sfz_paths) in &self.instruments {
            for sfz_path in sfz_paths {
                let metadata = InstrumentMetadata {
                    name: instrument.clone(),
                    category: instrument.clone(),
                    sfz_path: sfz_path.clone(),
                    flac_root: self.rdrs_root.join("FLAC"),
                    sample_count: 0,
                    avg_duration: 0.0,
                };
                config.instruments.push(metadata);
            }
        }
        
        // Save config
        let config_path = self.output_dir.join("dataset_config.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;
        
        info!("Built dataset with {} instruments", config.instruments.len());
        Ok(config)
    }
    
    pub fn generate_manifest(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut manifest = HashMap::new();
        
        for (instrument, sfz_paths) in &self.instruments {
            let paths: Vec<String> = sfz_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            manifest.insert(instrument.clone(), paths);
        }
        
        let manifest_path = self.output_dir.join("instrument_manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, manifest_json)?;
        
        Ok(manifest)
    }
}