use std::path::Path;
use anyhow::Result;
use red_dirt_studio_core::audio::AudioBuffer;
use red_dirt_studio_core::audio::wav::write_wav;
use red_dirt_studio_core::engine::BandEngine;
use red_dirt_studio_core::engine::driver::BandDriver;
use red_dirt_studio_core::fiddle::driver::{FiddleDriver, SectionType, Feel};
use red_dirt_studio_core::fiddle::player::FiddlePlayer;
pub use red_dirt_studio_core::models::BandDecision;
pub use red_dirt_studio_core::models::band_brain::OllamaBandBrain;
use red_dirt_studio_core::titan_engine::titan_mix_stems;
use red_dirt_studio_core::voice_inference::{VoiceInferenceEngine, VoiceInferenceConfig};

#[derive(Clone)]
pub struct SongInput {
    pub title: String,
    pub artist: String,
    pub key: String,
    pub bpm: u32,
    pub sections: Vec<(String, u32)>,
    pub chords: String,
    pub lyrics: String,
    pub description: String,
    pub instruments: Vec<String>,
    pub ollama_endpoint: String,
    pub model_name: String,
    pub temperature: f64,
}

#[derive(Clone, PartialEq)]
pub struct StemResult {
    pub name: String,
    pub buffer: AudioBuffer,
}

pub async fn generate_arrangement(
    input: &SongInput,
) -> Result<(BandDecision, Vec<StemResult>)> {
    let brain = OllamaBandBrain::new()
        .with_model(&input.model_name)
        .with_url(&input.ollama_endpoint)
        .with_temperature(input.temperature);

    let desc_lower = input.description.to_lowercase();
    let style = if desc_lower.contains("bluegrass") || desc_lower.contains("folk") {
        "bluegrass"
    } else if desc_lower.contains("ballad") || desc_lower.contains("slow") {
        "country ballad"
    } else if desc_lower.contains("rock") || desc_lower.contains("upbeat") {
        "country rock"
    } else if input.lyrics.contains("heart") || input.lyrics.contains("home")
        || input.lyrics.contains("road")
    {
        "country rock"
    } else if input.lyrics.contains("love") || input.lyrics.contains("baby") {
        "country ballad"
    } else if input.lyrics.contains("train") || input.lyrics.contains("track") {
        "bluegrass train beat"
    } else {
        "country rock"
    };
    let mood = if desc_lower.contains("sad") || desc_lower.contains("melancholic") {
        "melancholic"
    } else if desc_lower.contains("happy") || desc_lower.contains("joyful") {
        "upbeat"
    } else if input.title.to_lowercase().contains("sad")
        || input.title.to_lowercase().contains("lonely")
    {
        "melancholic"
    } else if input.title.to_lowercase().contains("happy")
        || input.title.to_lowercase().contains("dance")
    {
        "upbeat"
    } else {
        "driving"
    };
    let total_bars: u32 = input.sections.iter().map(|(_, b)| b).sum();
    let total_bars = if total_bars > 0 { total_bars as usize } else { 32 };

    let instrument_names: Vec<String> = input.instruments.iter()
        .map(|i| match i.as_str() {
            "drums" => "drums",
            "bass" => "bass",
            "guitar" => "acoustic_gtr",
            "keys" => "piano",
            "strings" => "strings",
            _ => i,
        })
        .map(String::from)
        .collect();

    let decision = brain.design_song(
        style, &input.key, mood, total_bars, instrument_names.clone(),
    ).await.unwrap_or_else(|_| {
        let fallback_chords: Vec<String> = if !input.chords.is_empty() {
            input.chords.split_whitespace().map(String::from).collect()
        } else {
            vec!["G".into(), "C".into(), "D".into(), "G".into()]
        };
        BandDecision {
            chord_progression: fallback_chords,
            bpm: input.bpm as f64,
            bars: total_bars,
            instruments: instrument_names.clone(),
            phrase_structure: input.sections.iter().map(|(name, bars)| {
                (name.clone(), red_dirt_studio_core::models::PhraseInfo {
                    bars: *bars as usize,
                    feel: mood.to_string(),
                })
            }).collect(),
            style_notes: format!("Fallback arrangement for '{}' by {} (Ollama unavailable)", input.title, input.artist),
        }
    });

    let bpm = decision.bpm;
    let core_driver = BandDriver::new(bpm);
    let engine = BandEngine::new(44100);
    let total_bars = decision.bars;
    let sec_per_bar = 60.0 / bpm * 4.0;
    let duration = total_bars as f64 * sec_per_bar;

    let feel = if input.lyrics.contains("train") || input.lyrics.contains("track") {
        Feel::Train
    } else if input.lyrics.contains("shuffle") {
        Feel::Shuffle
    } else if input.title.to_lowercase().contains("ballad") {
        Feel::HalfTime
    } else {
        Feel::Straight
    };

    let mut stems = Vec::new();
    for inst in &input.instruments {
        if inst == "fiddle" {
            let fiddle_driver = FiddleDriver::new(bpm);
            let mut fiddle_player = FiddlePlayer::new(44100);

            let sections: Vec<SectionType> = if !decision.phrase_structure.is_empty() {
                let section_order: Vec<&str> = input.sections.iter().map(|(n, _)| n.as_str()).collect();
                section_order.iter().map(|n| SectionType::from_str(n)).collect()
            } else {
                vec![SectionType::Verse, SectionType::Chorus,
                     SectionType::Verse, SectionType::Chorus,
                     SectionType::Bridge, SectionType::Chorus, SectionType::Outro]
            };

            let vocal_ranges = build_vocal_ranges(&input.sections, bpm);

            let events = fiddle_driver.generate_fiddle_track(
                &decision.chord_progression,
                &input.sections,
                feel,
                &sections,
                &vocal_ranges,
            );

            let buffer = fiddle_player.render_events(&events, duration)?;
            stems.push(StemResult {
                name: format!("{}_{}", input.title.replace(' ', "_"), inst),
                buffer,
            });
        } else {
            let driver_inst = match inst.as_str() {
                "drums" => "drums",
                "bass" => "bass",
                "guitar" => "acoustic_gtr",
                "keys" => "piano",
                "strings" => "strings",
                _ => inst,
            };
            let events = core_driver.generate_phrase_plan(&decision.chord_progression, total_bars, driver_inst);
            let mut buffer = engine.render_events(&events, duration)?;
            buffer.normalize(0.95);
            stems.push(StemResult {
                name: format!("{}_{}", input.title.replace(' ', "_"), inst),
                buffer,
            });
        }
    }

    let master = if stems.is_empty() {
        AudioBuffer::new(ndarray::Array1::zeros(0), 44100, 2)
    } else {
        // Use the Titan engine's SuperAnalogue + DFC summing for the master mix
        let sample_rate = stems[0].buffer.sample_rate;
        let channels = stems[0].buffer.channels as usize;
        let total_samples = stems[0].buffer.data.len();
        let total_frames = total_samples / channels;
        let mono_stems: Vec<(&str, &[f32])> = stems.iter()
            .filter(|s| !s.name.ends_with("_master"))
            .map(|s| (s.name.as_str(), s.buffer.data.as_slice().unwrap_or(&[])))
            .collect();
        let mixed: Vec<f32> = titan_mix_stems(&mono_stems, sample_rate, total_frames, -1.0);
        AudioBuffer::new(ndarray::Array1::from_vec(mixed), sample_rate, channels as u16)
    };
    stems.push(StemResult {
        name: format!("{}_master", input.title.replace(' ', "_")),
        buffer: master,
    });

    Ok((decision, stems))
}

pub async fn export_stems(stems: &[StemResult], output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    for stem in stems {
        let path = output_dir.join(format!("{}.wav", stem.name));
        write_wav(&path, &stem.buffer)?;
    }
    Ok(())
}

fn build_vocal_ranges(sections: &[(String, u32)], _bpm: f64) -> Vec<Vec<(f64, f64)>> {
    let mut bar_cursor = 0u32;
    let mut all_ranges = Vec::new();
    for (name, bars) in sections {
        let start_beat = bar_cursor as f64 * 4.0;
        let end_beat = (bar_cursor + bars) as f64 * 4.0;
        let section_name = name.to_lowercase();
        let ranges = match section_name.as_str() {
            "intro" | "solo" => vec![],
            "outro" => vec![(start_beat + 2.0, end_beat - 2.0)],
            "bridge" => {
                let first_half = start_beat + (end_beat - start_beat) * 0.3;
                let second_half = start_beat + (end_beat - start_beat) * 0.6;
                vec![(start_beat, first_half), (second_half, end_beat)]
            }
            _ => {
                let rest_start = start_beat + (end_beat - start_beat) * 0.6;
                vec![(start_beat, rest_start)]
            }
        };
        all_ranges.push(ranges);
        bar_cursor += bars;
    }
    all_ranges
}

pub async fn run_voice_inference(
    model_path: &str,
    input_path: &str,
    output_path: &str,
    pitch_shift: f64,
    _speed: f64,
) -> Result<String> {
    let config = VoiceInferenceConfig {
        model_path: model_path.to_string(),
        config_path: String::new(),
        speaker: "titan".to_string(),
        transpose: pitch_shift as i32,
        cluster_ratio: 0.0,
        noise_scale: 0.4,
        f0_method: "dio".to_string(),
        device: "cpu".to_string(),
    };

    let engine = VoiceInferenceEngine::new(config);
    engine.infer(Path::new(input_path), Path::new(output_path)).await?;

    Ok(format!("Voice conversion complete: {} -> {}", input_path, output_path))
}
