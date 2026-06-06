use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use anyhow::Result;
use super::{BandDecision, FillIdea, DriverEvent};

const OLLAMA_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_MODEL: &str = "qwen2:1.5b";

pub struct OllamaBandBrain {
    client: Client,
    model: String,
    url: String,
    temperature: f64,
}

impl OllamaBandBrain {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            model: DEFAULT_MODEL.to_string(),
            url: OLLAMA_URL.to_string(),
            temperature: 0.3,
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = format!("{}/api/chat", url.trim_end_matches('/'));
        self
    }
    
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
    
    pub async fn check_connection(&self) -> bool {
        let payload = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": "Reply with just: OK"}],
            "stream": false
        });
        
        match self.client.post(&self.url).json(&payload).timeout(std::time::Duration::from_secs(10)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
    
    async fn chat(&self, system: &str, user: &str, json_mode: bool) -> Result<Value> {
        let mut payload = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "stream": false,
            "options": {"temperature": self.temperature}
        });
        
        if json_mode {
            payload["format"] = json!("json");
        }
        
        let response = self.client
            .post(&self.url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;
        
        let data: Value = response.json().await?;
        let content = data["message"]["content"].as_str().unwrap_or("");
        
        if json_mode {
            Ok(serde_json::from_str(content)?)
        } else {
            Ok(json!({"text": content}))
        }
    }
    
    pub async fn design_song(
        &self,
        style: &str,
        key: &str,
        mood: &str,
        bars: usize,
        instruments: Vec<String>,
    ) -> Result<BandDecision> {
        let system = r#"You are a Nashville session arranger. Output ONLY valid JSON with:
{
  "chord_progression": ["G", "C", "D", "G"],
  "bpm": 96,
  "bars": 16,
  "instruments": ["acoustic_gtr", "bass", "drums"],
  "phrase_structure": {
    "intro": {"bars": 4, "feel": "sparse"},
    "verse": {"bars": 8, "feel": "driving"},
    "chorus": {"bars": 8, "feel": "full"},
    "turnaround": {"bars": 2, "feel": "walkup"},
    "outro": {"bars": 4, "feel": "fade"}
  },
  "style_notes": "Train beat, telecaster chicken pickin', pedal steel swells"
}"#;
        
        let user = format!(
            "Design a {} arrangement in {} major, {} feel, {} bars.\nAvailable instruments: {}\nInclude: chord progression, BPM, phrase map, and 1-sentence style notes.",
            style, key, mood, bars, instruments.join(", ")
        );
        
        let result = self.chat(system, &user, true).await?;
        
        let empty = vec![];
        Ok(BandDecision {
            chord_progression: result["chord_progression"]
                .as_array()
                .unwrap_or(&empty)
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            bpm: result["bpm"].as_f64().unwrap_or(96.0),
            bars: result["bars"].as_u64().unwrap_or(bars as u64) as usize,
            instruments: result["instruments"]
                .as_array()
                .unwrap_or(&empty)
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            phrase_structure: self.parse_phrase_structure(&result["phrase_structure"]),
            style_notes: result["style_notes"].as_str().unwrap_or("").to_string(),
        })
    }
    
    pub async fn generate_fills(
        &self,
        chord_progression: &[String],
        current_bar: usize,
        style: &str,
    ) -> Result<Vec<FillIdea>> {
        let system = r#"You are a session player. Output JSON array of fill ideas:
[{"bar": 4, "instrument": "fiddle", "notes": "G A B C#", "rhythm": "triplet"}]"#;
        
        let user = format!(
            "Bar {} in progression {:?}. Style: {}. Suggest 2-3 tasteful fills (fiddle, steel, guitar, etc).",
            current_bar, chord_progression, style
        );
        
        let result = self.chat(system, &user, true).await?;
        let empty = vec![];
        let fills = result.as_array().unwrap_or(&empty);
        
        Ok(fills.iter().map(|v| FillIdea {
            bar: v["bar"].as_u64().unwrap_or(0) as usize,
            instrument: v["instrument"].as_str().unwrap_or("").to_string(),
            notes: v["notes"].as_str().unwrap_or("").to_string(),
            rhythm: v["rhythm"].as_str().unwrap_or("quarter").to_string(),
        }).collect())
    }
    
    pub async fn vocal_follow_plan(
        &self,
        vocal_description: &str,
        chord_progression: &[String],
    ) -> Result<Vec<DriverEvent>> {
        let system = r#"Output JSON array of band hits aligned to vocal phrases:
[{"time_sec": 0.0, "chord": "G", "action": "downbeat kick + bass", "instrument": "drums"}]"#;
        
        let user = format!(
            "Vocal guide: \"{}\"\nChords: {:?}\nMap vocal phrases to band entrance/hits. Be specific with timing.",
            vocal_description, chord_progression
        );
        
        let result = self.chat(system, &user, true).await?;
        let empty = vec![];
        let events = result.as_array().unwrap_or(&empty);
        
        Ok(events.iter().map(|v| DriverEvent {
            time_sec: v["time_sec"].as_f64().unwrap_or(0.0),
            pitch: 60,
            velocity: 100,
            articulation: "sustain".to_string(),
            group: "band".to_string(),
            instrument: v["instrument"].as_str().unwrap_or("acoustic_gtr").to_string(),
            chord: v["chord"].as_str().map(String::from),
            meta: v.clone(),
        }).collect())
    }
    
    fn parse_phrase_structure(&self, value: &Value) -> HashMap<String, super::PhraseInfo> {
        let mut map = HashMap::new();
        
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                if let (Some(bars), Some(feel)) = (val["bars"].as_u64(), val["feel"].as_str()) {
                    map.insert(key.clone(), super::PhraseInfo {
                        bars: bars as usize,
                        feel: feel.to_string(),
                    });
                }
            }
        }
        
        map
    }
}

impl Default for OllamaBandBrain {
    fn default() -> Self {
        Self::new()
    }
}