use super::{ApprenticeConfig, SongDataset};
use crate::instruments::*;
use anyhow::Result;

pub struct ApprenticeOracle {
    config: ApprenticeConfig,
}

impl ApprenticeOracle {
    pub fn new(config: ApprenticeConfig) -> Self {
        Self { config }
    }

    pub fn suggest_song_plan(
        &self,
        _prompt: &str,
        user_key: Option<String>,
        user_tempo: Option<f32>,
        style_strength: f32,
        dataset: &SongDataset,
    ) -> Result<SongPlan> {
        // Extract style patterns from dataset
        let avg_tempo = self.extract_avg_tempo(dataset);
        let common_key = self.extract_common_key(dataset);
        let common_sections = self.extract_common_sections(dataset);
        let common_chords = self.extract_common_chords(dataset);

        // Blend user input with learned style
        let final_key = user_key.unwrap_or(common_key);
        let final_tempo = user_tempo.unwrap_or(avg_tempo);

        // Apply style strength: 0.0 = ignore apprentice, 1.0 = follow strictly
        let blended_strength = if style_strength > 0.0 {
            style_strength.min(1.0)
        } else {
            self.config.style_strength_default
        };

        // Build section specs
        let mut sections = Vec::new();
        let mut chords_per_section = Vec::new();

        for (idx, section_name) in common_sections.iter().enumerate() {
            let section_type = self.parse_section_type(section_name);
            let energy = if idx % 2 == 0 {
                Energy::Medium
            } else {
                Energy::High
            };

            sections.push(SectionSpec {
                section_type,
                start_bar: (idx as u32) * 8,
                end_bar: ((idx as u32) + 1) * 8,
                tempo_bpm: final_tempo,
                feel: Feel::Straight,
                energy,
            });

            chords_per_section.push(common_chords.clone());
        }

        Ok(SongPlan {
            key: final_key,
            tempo_bpm: final_tempo,
            sections,
            chords_per_section,
            drum_pattern_hints: vec!["TRAIN_BEAT_01".to_string()],
            bass_movement_hints: vec!["ROOT_3_5".to_string()],
            acoustic_pattern_hints: vec!["COUNTRY_6_8_PATTERN_01".to_string()],
            style_strength: blended_strength,
        })
    }

    fn extract_avg_tempo(&self, dataset: &SongDataset) -> f32 {
        let songs = dataset.get_all_songs();
        if songs.is_empty() {
            120.0  // default fallback
        } else {
            let sum: f32 = songs.iter().map(|s| s.tempo_bpm).sum();
            sum / songs.len() as f32
        }
    }

    fn extract_common_key(&self, dataset: &SongDataset) -> String {
        let songs = dataset.get_all_songs();
        if songs.is_empty() {
            "G".to_string()  // default fallback
        } else {
            songs[0].key.clone()  // simplistic: just take the first key
        }
    }

    fn extract_common_sections(&self, dataset: &SongDataset) -> Vec<String> {
        let songs = dataset.get_all_songs();
        if songs.is_empty() {
            return vec![
                "INTRO".to_string(),
                "VERSE1".to_string(),
                "CHORUS1".to_string(),
                "VERSE2".to_string(),
                "CHORUS2".to_string(),
                "BRIDGE".to_string(),
                "CHORUS3".to_string(),
                "OUTRO".to_string(),
            ];
        }

        songs[0].sections.clone()
    }

    fn extract_common_chords(&self, _dataset: &SongDataset) -> Vec<ChordToken> {
        // Simplistic default: common country progression
        vec![
            ChordToken {
                degree: 0,  // I
                quality: ChordQuality::Major,
                duration_beats: 4.0,
            },
            ChordToken {
                degree: 4,  // V
                quality: ChordQuality::Dom7,
                duration_beats: 4.0,
            },
            ChordToken {
                degree: 5,  // vi
                quality: ChordQuality::Minor,
                duration_beats: 4.0,
            },
            ChordToken {
                degree: 3,  // IV
                quality: ChordQuality::Major,
                duration_beats: 4.0,
            },
        ]
    }

    fn parse_section_type(&self, section_name: &str) -> SectionType {
        match section_name.to_uppercase().as_str() {
            "INTRO" => SectionType::Intro,
            "VERSE1" => SectionType::Verse1,
            "VERSE2" => SectionType::Verse2,
            "CHORUS1" => SectionType::Chorus1,
            "CHORUS2" => SectionType::Chorus2,
            "BRIDGE" => SectionType::Bridge,
            "SOLO" => SectionType::Solo,
            "OUTRO" => SectionType::Outro,
            _ => SectionType::Verse1,
        }
    }
}
