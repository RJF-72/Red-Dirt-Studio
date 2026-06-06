use super::SongLog;

pub struct SongLogger;

impl SongLogger {
    pub fn new() -> Self {
        Self
    }

    /// Extract structured data from a generated song for the apprentice to learn
    pub fn log_song(
        &self,
        id: String,
        prompt: String,
        key: String,
        tempo_bpm: f32,
        sections: Vec<String>,
        chords: Vec<String>,
        drum_pattern_ids: Vec<String>,
        bass_movement_ids: Vec<String>,
        acoustic_pattern_ids: Vec<String>,
        arrangement_density: f32,
    ) -> SongLog {
        SongLog {
            id,
            timestamp: chrono::Utc::now(),
            prompt,
            key,
            tempo_bpm,
            sections,
            chords,
            drum_pattern_ids,
            bass_movement_ids,
            acoustic_pattern_ids,
            arrangement_density,
            user_rating: None,
            style_alignment: 0.0,
        }
    }
}
