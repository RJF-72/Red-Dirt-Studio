use std::collections::HashMap;
use std::sync::LazyLock;

use super::PresetConfig;

pub static NASHVILLE_PRESETS: LazyLock<Vec<(&str, PresetConfig)>> = LazyLock::new(|| {
    vec![
        (
            "basic_band",
            PresetConfig {
                name: "basic_band".to_string(),
                bpm: 96.0,
                chords: vec!["C".to_string(), "G".to_string(), "Am".to_string(), "F".to_string()],
                instruments: vec!["acoustic_gtr".to_string(), "bass".to_string(), "drums".to_string()],
            },
        ),
        (
            "ballad",
            PresetConfig {
                name: "ballad".to_string(),
                bpm: 72.0,
                chords: vec!["G".to_string(), "Em".to_string(), "C".to_string(), "D".to_string()],
                instruments: vec!["piano".to_string(), "pad".to_string(), "steel".to_string()],
            },
        ),
        (
            "train_beat",
            PresetConfig {
                name: "train_beat".to_string(),
                bpm: 120.0,
                chords: vec!["D".to_string(), "G".to_string(), "A".to_string(), "D".to_string()],
                instruments: vec!["acoustic_gtr".to_string(), "snare".to_string(), "bass".to_string()],
            },
        ),
        (
            "country_ballad",
            PresetConfig {
                name: "country_ballad".to_string(),
                bpm: 80.0,
                chords: vec!["G".to_string(), "D".to_string(), "Em".to_string(), "C".to_string()],
                instruments: vec!["fiddle".to_string(), "pedal_steel".to_string(), "acoustic_gtr".to_string(), "bass".to_string()],
            },
        ),
        (
            "bluegrass",
            PresetConfig {
                name: "bluegrass".to_string(),
                bpm: 140.0,
                chords: vec!["G".to_string(), "C".to_string(), "D".to_string(), "G".to_string()],
                instruments: vec!["banjo".to_string(), "mandolin".to_string(), "fiddle".to_string(), "bass".to_string()],
            },
        ),
    ]
});

pub fn get_preset(name: &str) -> Option<PresetConfig> {
    NASHVILLE_PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, preset)| preset.clone())
}

pub fn get_all_presets() -> HashMap<String, PresetConfig> {
    NASHVILLE_PRESETS
        .iter()
        .map(|(name, preset)| (name.to_string(), preset.clone()))
        .collect()
}