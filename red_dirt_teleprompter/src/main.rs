use red_dirt_karaoke_teleprompter::*;
use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let lyrics_json = args.get(1).cloned().unwrap_or_else(|| {
        r#"[{"time":0,"text":"Demo Song"}]"#.to_string()
    });
    let duration_s: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30.0);

    let raw = if fs::metadata(&lyrics_json).is_ok() {
        fs::read_to_string(&lyrics_json)?
    } else {
        lyrics_json.clone()
    };
    let lyrics_data: Vec<serde_json::Value> = serde_json::from_str(&raw)?;

    let lyric_lines: Vec<LyricLine> = lyrics_data.iter().enumerate().map(|(i, l)| {
        let time = l["time"].as_f64().unwrap_or(i as f64 * 4.0) as f32;
        let text = l["text"].as_str().unwrap_or("").to_string();
        let words: Vec<LyricWord> = text.split_whitespace().enumerate().map(|(wi, w)| {
            let wt = time + wi as f32 * 0.15;
            LyricWord { text: w.to_string(), start_time: wt, end_time: wt + 0.3, position: wi }
        }).collect();
        LyricLine { text, start_time: time, end_time: time + 2.0, words, line_number: i }
    }).collect();

    let config = TeleprompterConfig {
        sample_rate: 44100.0,
        buffer_size: 512,
        neural_resolution: 512,
        timing_precision: 1000,
        highlight_prediction_ms: 100.0,
        network_streaming: true,
        producer_dashboard: false,
        performance_analytics: false,
        device_support: DeviceSupport { web_client: true, android_app: false, ios_app: false, smart_tv: false, projector: false },
    };

    let mut tp = RedDirtKaraokeTeleprompter::new(config)?;
    tp.session.lyrics = lyric_lines;
    tp.session.song_title = "Generated Song".to_string();
    tp.start_performance()?;

    println!("TELEPROMPTER_READY");

    let start = std::time::Instant::now();
    let total_ms = (duration_s * 1000.0) as f32;
    loop {
        thread::sleep(Duration::from_millis(50));
        let _ = tp.update_performance();
        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
        if elapsed >= total_ms { break; }
    }

    println!("TELEPROMPTER_DONE");
    Ok(())
}
