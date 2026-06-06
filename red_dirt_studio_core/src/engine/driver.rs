
use crate::models::DriverEvent;

pub struct BandDriver {
    pub bpm: f64,
}

impl BandDriver {
    pub fn new(bpm: f64) -> Self {
        Self { bpm }
    }
    
    pub fn chord_root_to_midi(&self, chord: &str) -> u8 {
        let root = chord.chars().next().unwrap_or('C').to_ascii_uppercase();
        match root {
            'C' => 60,
            'D' => 62,
            'E' => 64,
            'F' => 65,
            'G' => 67,
            'A' => 69,
            'B' => 71,
            _ => 60,
        }
    }
    
    pub fn generate_phrase_plan(
        &self,
        chord_progression: &[String],
        bars: usize,
        instrument: &str,
    ) -> Vec<DriverEvent> {
        let mut events = Vec::new();
        let sec_per_beat = 60.0 / self.bpm;
        let total_beats = bars * 4;
        
        for i in 0..total_beats {
            let chord_idx = i % chord_progression.len();
            let chord = &chord_progression[chord_idx];
            let time_sec = i as f64 * sec_per_beat;
            let root_pitch = self.chord_root_to_midi(chord);
            
            let velocity = if i % 4 == 0 { 100 } else { 80 };
            let articulation = if i % 2 == 0 { "sustain" } else { "staccato" };
            
            events.push(DriverEvent {
                time_sec,
                pitch: root_pitch,
                velocity,
                articulation: articulation.to_string(),
                group: instrument.to_string(),
                instrument: instrument.to_string(),
                chord: Some(chord.clone()),
                meta: serde_json::json!({"beat_index": i}),
            });
        }
        
        events
    }
    
    pub fn generate_groove_pattern(
        &self,
        chord_progression: &[String],
        pattern: &[usize],
        instrument: &str,
    ) -> Vec<DriverEvent> {
        let mut events = Vec::new();
        let sec_per_beat = 60.0 / self.bpm;
        
        for (i, &beat_offset) in pattern.iter().enumerate() {
            let chord = &chord_progression[i % chord_progression.len()];
            let time_sec = beat_offset as f64 * sec_per_beat / 4.0;
            let root_pitch = self.chord_root_to_midi(chord);
            
            events.push(DriverEvent {
                time_sec,
                pitch: root_pitch,
                velocity: 90,
                articulation: "sustain".to_string(),
                group: instrument.to_string(),
                instrument: instrument.to_string(),
                chord: Some(chord.clone()),
                meta: serde_json::json!({"pattern_index": i}),
            });
        }
        
        events
    }
}