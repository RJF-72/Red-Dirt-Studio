use rand::random;
use serde_json::json;
use crate::models::DriverEvent;
use crate::instruments::tokens::SectionType as SharedSectionType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Solo,
    Outro,
}

impl SectionType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "intro" => Self::Intro,
            "verse" | "verse1" => Self::Verse,
            "chorus" | "chorus1" => Self::Chorus,
            "bridge" => Self::Bridge,
            "solo" => Self::Solo,
            "outro" => Self::Outro,
            _ => Self::Verse,
        }
    }
}

impl From<SectionType> for SharedSectionType {
    fn from(val: SectionType) -> Self {
        match val {
            SectionType::Intro => SharedSectionType::Intro,
            SectionType::Verse => SharedSectionType::Verse1,
            SectionType::Chorus => SharedSectionType::Chorus1,
            SectionType::Bridge => SharedSectionType::Bridge,
            SectionType::Solo => SharedSectionType::Solo,
            SectionType::Outro => SharedSectionType::Outro,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Energy {
    Low,
    Medium,
    High,
}

impl Energy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" | "sparse" => Self::Low,
            "high" | "driving" | "full" => Self::High,
            _ => Self::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feel {
    Straight,
    Shuffle,
    Train,
    HalfTime,
}

impl Feel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "shuffle" => Self::Shuffle,
            "train" | "train beat" => Self::Train,
            "half-time" | "half_time" => Self::HalfTime,
            _ => Self::Straight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseType {
    Fill,
    Hook,
    SoloFrag,
    Pad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Articulation {
    Sustain,
    Staccato,
    Slide,
    Bend,
    DoubleStop,
    Chop,
    Trill,
    Drone,
    Pizz,
}

impl Articulation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sustain => "sustain",
            Self::Staccato => "staccato",
            Self::Slide => "slide",
            Self::Bend => "bend",
            Self::DoubleStop => "double_stop",
            Self::Chop => "chop",
            Self::Trill => "trill",
            Self::Drone => "drone",
            Self::Pizz => "pizz",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FiddlePhrase {
    pub phrase_type: PhraseType,
    pub articulation: Articulation,
    pub contour: Vec<i8>,
    pub start_beat: f64,
    pub duration_beats: f64,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub struct FiddleSectionPlan {
    pub section_type: SectionType,
    pub energy: Energy,
    pub feel: Feel,
    pub bpm: f64,
    pub start_bar: u32,
    pub num_bars: u32,
    pub chords: Vec<String>,
    pub vocal_active_ranges: Vec<(f64, f64)>,
    pub phrases: Vec<FiddlePhrase>,
}

#[derive(Debug, Clone)]
pub struct FiddleDriver {
    pub bpm: f64,
}

impl FiddleDriver {
    pub fn new(bpm: f64) -> Self {
        Self { bpm }
    }

    pub fn plan_section(
        &self,
        section_type: SectionType,
        energy: Energy,
        feel: Feel,
        start_bar: u32,
        num_bars: u32,
        chords: Vec<String>,
        vocal_active_ranges: Vec<(f64, f64)>,
    ) -> FiddleSectionPlan {
        let phrases = self.generate_phrases(
            section_type,
            energy,
            feel,
            num_bars,
            &chords,
            &vocal_active_ranges,
        );
        FiddleSectionPlan {
            section_type,
            energy,
            feel,
            bpm: self.bpm,
            start_bar,
            num_bars,
            chords,
            vocal_active_ranges,
            phrases,
        }
    }

    fn generate_phrases(
        &self,
        section_type: SectionType,
        energy: Energy,
        feel: Feel,
        num_bars: u32,
        _chords: &[String],
        vocal_active_ranges: &[(f64, f64)],
    ) -> Vec<FiddlePhrase> {
        let beats_per_bar = match feel {
            Feel::Straight | Feel::Shuffle | Feel::Train => 4.0,
            Feel::HalfTime => 2.0,
        };
        let total_beats = num_bars as f64 * beats_per_bar;

        let mut phrases = Vec::new();
        let beats_per_section = total_beats;

        match section_type {
            SectionType::Intro => {
                phrases.push(FiddlePhrase {
                    phrase_type: PhraseType::Pad,
                    articulation: Articulation::Sustain,
                    contour: vec![0, 2, 4, 5, 7],
                    start_beat: 0.0,
                    duration_beats: beats_per_section.min(4.0),
                    intensity: 0.5,
                });
                if energy == Energy::Medium || energy == Energy::High {
                    phrases.push(FiddlePhrase {
                        phrase_type: PhraseType::Fill,
                        articulation: Articulation::Slide,
                        contour: vec![7, 5, 4, 2, 0],
                        start_beat: beats_per_section - 2.0,
                        duration_beats: 1.5,
                        intensity: 0.6,
                    });
                }
            }

            SectionType::Verse => {
                let (fill_density, intensity) = match energy {
                    Energy::Low => (0.25, 0.35),
                    Energy::Medium => (0.4, 0.5),
                    Energy::High => (0.55, 0.65),
                };

                for bar in 0..num_bars {
                    let bar_start_beat = bar as f64 * beats_per_bar;

                    let has_vocal = vocal_active_ranges.iter().any(|(s, e)| {
                        let beat_s = bar_start_beat;
                        let beat_e = bar_start_beat + beats_per_bar;
                        *s < beat_e && *e > beat_s
                    });

                    if has_vocal && random::<f32>() < fill_density {
                        let fill_beat = bar_start_beat + beats_per_bar * 0.5
                            + random::<f32>() as f64 * beats_per_bar * 0.3;

                        let articulation = match feel {
                            Feel::Shuffle => Articulation::Slide,
                            Feel::Train => Articulation::Chop,
                            _ => {
                                if random::<f32>() < 0.3 {
                                    Articulation::DoubleStop
                                } else if random::<f32>() < 0.2 {
                                    Articulation::Bend
                                } else {
                                    Articulation::Staccato
                                }
                            }
                        };

                        let contour = match articulation {
                            Articulation::Slide => vec![5, 7, 5],
                            Articulation::Chop => vec![-2, 0, 2],
                            Articulation::DoubleStop => vec![0, 4, 7],
                            Articulation::Bend => vec![0, 1, 0],
                            _ => vec![0, 2, 4, 2],
                        };

                        phrases.push(FiddlePhrase {
                            phrase_type: PhraseType::Fill,
                            articulation,
                            contour,
                            start_beat: fill_beat,
                            duration_beats: 1.0,
                            intensity,
                        });
                    }

                    if !has_vocal && random::<f32>() < fill_density * 1.5 {
                        let fill_beat = bar_start_beat + 1.0 + random::<f32>() as f64 * (beats_per_bar - 2.0);
                        phrases.push(FiddlePhrase {
                            phrase_type: PhraseType::Fill,
                            articulation: Articulation::Sustain,
                            contour: vec![0, 3, 5, 7],
                            start_beat: fill_beat,
                            duration_beats: 1.5,
                            intensity: intensity * 1.2,
                        });
                    }
                }
            }

            SectionType::Chorus => {
                let (hook_freq, intensity) = match energy {
                    Energy::Low => (0.5, 0.5),
                    Energy::Medium => (0.75, 0.65),
                    Energy::High => (1.0, 0.8),
                };

                for bar in 0..num_bars {
                    let bar_start_beat = bar as f64 * beats_per_bar;

                    if random::<f32>() < hook_freq {
                        let articulation = match feel {
                            Feel::Shuffle => Articulation::Slide,
                            _ => {
                                if intensity > 0.7 {
                                    Articulation::DoubleStop
                                } else if random::<f32>() < 0.3 {
                                    Articulation::Trill
                                } else {
                                    Articulation::Sustain
                                }
                            }
                        };

                        let contour = match energy {
                            Energy::High => vec![0, 5, 7, 10, 7, 5],
                            Energy::Medium => vec![0, 4, 7, 4],
                            Energy::Low => vec![0, 2, 4],
                        };

                        phrases.push(FiddlePhrase {
                            phrase_type: PhraseType::Hook,
                            articulation,
                            contour,
                            start_beat: bar_start_beat,
                            duration_beats: 2.0,
                            intensity,
                        });
                    }
                }

                if (energy == Energy::Medium || energy == Energy::High) && num_bars >= 4 {
                    let pre_chorus_bar = if num_bars >= 8 { 4 } else { 2 };
                    phrases.push(FiddlePhrase {
                        phrase_type: PhraseType::Fill,
                        articulation: Articulation::Slide,
                        contour: vec![7, 10, 12, 10, 7],
                        start_beat: pre_chorus_bar as f64 * beats_per_bar,
                        duration_beats: 2.0,
                        intensity: intensity * 1.1,
                    });
                }
            }

            SectionType::Bridge => {
                let intensity = match energy {
                    Energy::Low => 0.4,
                    Energy::Medium => 0.6,
                    Energy::High => 0.85,
                };

                let use_solo = energy == Energy::High;
                if use_solo && num_bars >= 4 {
                    for bar in 0..num_bars {
                        let bar_start_beat = bar as f64 * beats_per_bar;
                        let articulation = if random::<f32>() < 0.4 {
                            Articulation::Bend
                        } else if random::<f32>() < 0.3 {
                            Articulation::Slide
                        } else {
                            Articulation::Sustain
                        };
                        let contour = match articulation {
                            Articulation::Bend => vec![0, 1, 2, 3, 5, 7, 10],
                            Articulation::Slide => vec![12, 10, 7, 5],
                            _ => vec![0, 3, 7, 10, 7, 3],
                        };
                        phrases.push(FiddlePhrase {
                            phrase_type: PhraseType::SoloFrag,
                            articulation,
                            contour,
                            start_beat: bar_start_beat,
                            duration_beats: 2.5,
                            intensity,
                        });
                    }
                } else {
                    phrases.push(FiddlePhrase {
                        phrase_type: PhraseType::Pad,
                        articulation: Articulation::Sustain,
                        contour: vec![0, 7, 4, 0],
                        start_beat: 0.0,
                        duration_beats: beats_per_section.min(6.0),
                        intensity: 0.5,
                    });
                    phrases.push(FiddlePhrase {
                        phrase_type: PhraseType::Fill,
                        articulation: Articulation::Slide,
                        contour: vec![7, 10, 12, 14, 12],
                        start_beat: beats_per_section - 3.0,
                        duration_beats: 2.0,
                        intensity: intensity * 0.9,
                    });
                }
            }

            SectionType::Solo => {
                let intensity = match energy {
                    Energy::Low => 0.5,
                    Energy::Medium => 0.7,
                    Energy::High => 0.9,
                };
                for bar in 0..num_bars {
                    let bar_start_beat = bar as f64 * beats_per_bar;
                    let articulation = match (bar % 4, random::<f32>()) {
                        (0, _) => Articulation::Slide,
                        (_, p) if p < 0.25 => Articulation::Bend,
                        (_, p) if p < 0.45 => Articulation::DoubleStop,
                        (_, p) if p < 0.6 => Articulation::Trill,
                        _ => Articulation::Sustain,
                    };
                    let contour = match articulation {
                        Articulation::Slide => vec![0, 5, 10, 12, 10],
                        Articulation::Bend => vec![7, 8, 7, 5, 3],
                        Articulation::DoubleStop => vec![0, 4, 7, 4],
                        Articulation::Trill => vec![0, 1, 0, 1, 0],
                        _ => vec![0, 7, 10, 7, 5, 7],
                    };
                    phrases.push(FiddlePhrase {
                        phrase_type: PhraseType::SoloFrag,
                        articulation,
                        contour,
                        start_beat: bar_start_beat,
                        duration_beats: 3.0,
                        intensity,
                    });
                }
            }

            SectionType::Outro => {
                phrases.push(FiddlePhrase {
                    phrase_type: PhraseType::Pad,
                    articulation: Articulation::Sustain,
                    contour: vec![0, 4, 7, 4, 0],
                    start_beat: 0.0,
                    duration_beats: beats_per_section.min(6.0),
                    intensity: 0.4,
                });
            }
        }

        phrases
    }

    pub fn generate_events(
        &self,
        plan: &FiddleSectionPlan,
        chord_root: u8,
    ) -> Vec<DriverEvent> {
        let mut events = Vec::new();
        let sec_per_beat = 60.0 / self.bpm;
        let bar_offset_sec = plan.start_bar as f64 * 4.0 * sec_per_beat;

        for phrase in &plan.phrases {
            let start_sec = bar_offset_sec + phrase.start_beat * sec_per_beat;
            let duration_sec = phrase.duration_beats * sec_per_beat;

            let notes = self.phrase_to_midi(phrase, chord_root);

            for (i, note) in notes.iter().enumerate() {
                let offset = i as f64 * (duration_sec / notes.len() as f64);
                let time_sec = start_sec + offset;
                let velocity = (phrase.intensity * 127.0) as u8;

                let articulation_str = phrase.articulation.as_str().to_string();
                let phrase_type_str = match phrase.phrase_type {
                    PhraseType::Fill => "fill",
                    PhraseType::Hook => "hook",
                    PhraseType::SoloFrag => "solo_frag",
                    PhraseType::Pad => "pad",
                };

                events.push(DriverEvent {
                    time_sec,
                    pitch: *note,
                    velocity,
                    articulation: articulation_str,
                    group: "fiddle".to_string(),
                    instrument: "fiddle".to_string(),
                    chord: plan.chords.first().cloned(),
                    meta: json!({
                        "phrase_type": phrase_type_str,
                        "intensity": phrase.intensity,
                        "contour": phrase.contour,
                        "duration_sec": duration_sec,
                        "section": format!("{:?}", plan.section_type),
                    }),
                });
            }
        }

        events
    }

    fn phrase_to_midi(&self, phrase: &FiddlePhrase, root: u8) -> Vec<u8> {
        let mut notes = Vec::new();
        let mut current = root as i16;
        for &step in &phrase.contour {
            current = (current + step as i16).clamp(24, 96);
            notes.push(current as u8);
        }
        if notes.is_empty() {
            notes.push(root);
        }
        notes
    }

    pub fn generate_fiddle_track(
        &self,
        chord_progression: &[String],
        phrase_structure: &[(String, u32)],
        feel: Feel,
        sections: &[SectionType],
        vocal_ranges: &[Vec<(f64, f64)>],
    ) -> Vec<DriverEvent> {
        let mut all_events = Vec::new();
        let mut bar_cursor = 0u32;

        for (i, section) in sections.iter().enumerate() {
            let section_name = &phrase_structure[i].0;
            let num_bars = phrase_structure[i].1;

            let energy = Energy::from_str(section_name);
            let chords: Vec<String> = chord_progression
                .iter()
                .cycle()
                .take(num_bars as usize)
                .cloned()
                .collect();

            let v_ranges = vocal_ranges.get(i).cloned().unwrap_or_default();

            let plan = self.plan_section(
                *section,
                energy,
                feel,
                bar_cursor,
                num_bars,
                chords.clone(),
                v_ranges,
            );

            let first_chord_root = crate::engine::driver::BandDriver::new(self.bpm)
                .chord_root_to_midi(chords.first().map(|s| s.as_str()).unwrap_or("C"));

            let events = self.generate_events(&plan, first_chord_root);
            all_events.extend(events);

            bar_cursor += num_bars;
        }

        all_events
    }
}
