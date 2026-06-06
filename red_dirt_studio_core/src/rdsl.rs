use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct RdslScript {
    pub session: Option<String>,
    pub tracks: Vec<Track>,
    pub mixdown: Option<Mixdown>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub name: String,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mixdown {
    pub filename: String,
    pub loudness: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Compressor { threshold_db: f32, ratio: f32 },
    Loudness { target_db: f32 },
    Reverb { room_size: f32 },
    Delay { time_ms: u32, feedback: f32 },
    Eq { low_db: f32, mid_db: f32, high_db: f32 },
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Compressor { threshold_db, ratio } => write!(f, "compress threshold={} ratio={}", threshold_db, ratio),
            Effect::Loudness { target_db } => write!(f, "loudness {}", target_db),
            Effect::Reverb { room_size } => write!(f, "reverb room_size={}", room_size),
            Effect::Delay { time_ms, feedback } => write!(f, "delay time_ms={} feedback={}", time_ms, feedback),
            Effect::Eq { low_db, mid_db, high_db } => write!(f, "eq low={} mid={} high={}", low_db, mid_db, high_db),
        }
    }
}

impl RdslScript {
    pub fn new() -> Self {
        Self {
            session: None,
            tracks: Vec::new(),
            mixdown: None,
        }
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        if let Some(session) = &self.session {
            out.push_str(&format!("session \"{}\"\n", session));
        }
        for track in &self.tracks {
            out.push_str(&format!("track {} {{\n", track.name));
            for effect in &track.effects {
                out.push_str(&format!("  {}\n", effect));
            }
            out.push_str("}\n");
        }
        if let Some(mixdown) = &self.mixdown {
            out.push_str(&format!("mixdown \"{}\"", mixdown.filename));
            if let Some(loudness) = mixdown.loudness {
                out.push_str(&format!(" loudness {}", loudness));
            }
            out.push('\n');
        }
        out
    }
}

pub fn parse_rdsl(script: &str) -> Result<RdslScript> {
    let mut rdsl = RdslScript::new();
    let mut current_track: Option<Track> = None;

    for (line_index, line) in script.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("session ") {
            if let Some(name) = trimmed.split_once('"').and_then(|(_, rest)| rest.split_once('"')).map(|(inner, _)| inner.to_string()) {
                rdsl.session = Some(name);
                continue;
            }
            anyhow::bail!("Invalid session declaration at line {}: {}", line_index + 1, trimmed);
        }

        if trimmed.starts_with("track ") && trimmed.ends_with('{') {
            if current_track.is_some() {
                anyhow::bail!("Nested track declarations are not supported at line {}", line_index + 1);
            }
            let name = trimmed[6..trimmed.len() - 1].trim().to_string();
            current_track = Some(Track { name, effects: Vec::new() });
            continue;
        }

        if trimmed == "}" {
            if let Some(track) = current_track.take() {
                rdsl.tracks.push(track);
                continue;
            }
            anyhow::bail!("Unexpected closing brace at line {}", line_index + 1);
        }

        if trimmed.starts_with("mixdown ") {
            let mut parts = trimmed[8..].trim().split_whitespace();
            let filename_part = parts.next().context("Missing mixdown filename")?;
            let filename = filename_part.trim_matches('"').to_string();
            let mut loudness = None;
            while let Some(token) = parts.next() {
                if token == "loudness" {
                    if let Some(value) = parts.next() {
                        loudness = Some(value.parse::<f32>().context("Invalid loudness value")?);
                    }
                }
            }
            rdsl.mixdown = Some(Mixdown { filename, loudness });
            continue;
        }

        if let Some(track) = current_track.as_mut() {
            let effect = parse_effect(trimmed).context(format!("Parsing effect at line {}", line_index + 1))?;
            track.effects.push(effect);
            continue;
        }

        anyhow::bail!("Unexpected token at line {}: {}", line_index + 1, trimmed);
    }

    if current_track.is_some() {
        anyhow::bail!("Unclosed track block at end of script");
    }

    Ok(rdsl)
}

fn parse_effect(line: &str) -> Result<Effect> {
    let mut parts = line.split_whitespace();
    let keyword = parts.next().context("Empty effect line")?;
    let args = parts.collect::<Vec<_>>().join(" ");
    let parsed = match keyword {
        "compress" => {
            let values = parse_named_args(&args, &["threshold", "ratio"])?;
            Effect::Compressor { threshold_db: values[0].parse::<f32>()?, ratio: values[1].parse::<f32>()? }
        }
        "loudness" => {
            let value = args.parse::<f32>().context("Invalid loudness value")?;
            Effect::Loudness { target_db: value }
        }
        "reverb" => {
            let values = parse_named_args(&args, &["room_size"])?;
            Effect::Reverb { room_size: values[0].parse::<f32>()? }
        }
        "delay" => {
            let values = parse_named_args(&args, &["time_ms", "feedback"])?;
            Effect::Delay { time_ms: values[0].parse::<u32>()?, feedback: values[1].parse::<f32>()? }
        }
        "eq" => {
            let values = parse_named_args(&args, &["low", "mid", "high"])?;
            Effect::Eq { low_db: values[0].parse::<f32>()?, mid_db: values[1].parse::<f32>()?, high_db: values[2].parse::<f32>()? }
        }
        _ => anyhow::bail!("Unknown effect '{}'", keyword),
    };
    Ok(parsed)
}

fn parse_named_args(args: &str, names: &[&str]) -> Result<Vec<String>> {
    let tokens = args.split_whitespace().collect::<Vec<_>>();
    let mut values = Vec::with_capacity(names.len());
    for name in names {
        let token = tokens.iter().find(|token| token.starts_with(&format!("{}=", name)))
            .context(format!("Missing argument '{}'", name))?;
        let (_, value) = token.split_once('=').context("Invalid named argument")?;
        values.push(value.to_string());
    }
    Ok(values)
}

pub fn translate_natural_language(prompt: &str) -> RdslScript {
    let prompt = prompt.to_lowercase();
    let mut script = RdslScript::new();

    script.session = Some("AI Session".to_string());
    script.tracks.push(Track { name: "vocal".to_string(), effects: Vec::new() });

    let track = script.tracks.last_mut().unwrap();

    if prompt.contains("compress") || prompt.contains("smooth") || prompt.contains("control the vocal") {
        track.effects.push(Effect::Compressor { threshold_db: -18.0, ratio: 2.5 });
    }
    if prompt.contains("loud") || prompt.contains("boost") || prompt.contains("present") {
        track.effects.push(Effect::Loudness { target_db: -8.0 });
    }
    if prompt.contains("bright") || prompt.contains("air") {
        track.effects.push(Effect::Eq { low_db: 0.0, mid_db: 0.0, high_db: 3.0 });
    }
    if prompt.contains("warm") {
        track.effects.push(Effect::Eq { low_db: 2.0, mid_db: 0.0, high_db: -1.0 });
    }
    if prompt.contains("echo") || prompt.contains("delay") {
        track.effects.push(Effect::Delay { time_ms: 250, feedback: 0.35 });
    }
    if prompt.contains("space") || prompt.contains("reverb") || prompt.contains("ambient") {
        track.effects.push(Effect::Reverb { room_size: 0.45 });
    }
    if track.effects.is_empty() {
        track.effects.push(Effect::Compressor { threshold_db: -14.0, ratio: 2.0 });
        track.effects.push(Effect::Loudness { target_db: -10.0 });
    }

    script.mixdown = Some(Mixdown { filename: "output.wav".to_string(), loudness: Some(-14.0) });
    script
}

pub fn compile_to_rust(script: &RdslScript) -> String {
    let mut code = String::new();
    code.push_str("// Generated RDSL plugin in Rust\n");
    code.push_str("use red_dirt_studio_core::rdsl::Effect;\n\n");
    code.push_str("fn build_rdsl_plugin() -> Vec<Effect> {\n");
    code.push_str("    vec![\n");
    for track in &script.tracks {
        code.push_str(&format!("        // Track: {}\n", track.name));
        for effect in &track.effects {
            code.push_str(&format!("        Effect::{},\n", rust_effect_variant(effect)));
        }
    }
    code.push_str("    ]\n}");
    code
}

fn rust_effect_variant(effect: &Effect) -> String {
    match effect {
        Effect::Compressor { threshold_db, ratio } => format!("Compressor {{ threshold_db: {}, ratio: {} }}", threshold_db, ratio),
        Effect::Loudness { target_db } => format!("Loudness {{ target_db: {} }}", target_db),
        Effect::Reverb { room_size } => format!("Reverb {{ room_size: {} }}", room_size),
        Effect::Delay { time_ms, feedback } => format!("Delay {{ time_ms: {}, feedback: {} }}", time_ms, feedback),
        Effect::Eq { low_db, mid_db, high_db } => format!("Eq {{ low_db: {}, mid_db: {}, high_db: {} }}", low_db, mid_db, high_db),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rdsl() {
        let script = r#"
            session "Test"
            track vocal {
                compress threshold=-12 ratio=3
                loudness -8
            }
            mixdown "out.wav" loudness -14
        "#;
        let parsed = parse_rdsl(script).expect("Failed to parse");
        assert_eq!(parsed.session, Some("Test".to_string()));
        assert_eq!(parsed.tracks.len(), 1);
        assert!(matches!(parsed.tracks[0].effects[0], Effect::Compressor { .. }));
    }

    #[test]
    fn translate_from_nl() {
        let prompt = "Make the vocal sound brighter and more present with a little delay.";
        let script = translate_natural_language(prompt);
        assert!(script.tracks[0].effects.iter().any(|e| matches!(e, Effect::Eq { .. })));
        assert!(script.tracks[0].effects.iter().any(|e| matches!(e, Effect::Delay { .. })));
    }
}
