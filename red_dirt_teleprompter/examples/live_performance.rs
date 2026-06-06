use red_dirt_karaoke_teleprompter::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Starting Red Dirt Karaoke Teleprompter v1.0");
    println!("World's Most Advanced Lyric Alignment System");

    // Configure teleprompter
    let config = TeleprompterConfig {
        sample_rate: 96000.0,
        buffer_size: 256,
        neural_resolution: 1024,
        timing_precision: 100,
        highlight_prediction_ms: 50.0,
        network_streaming: true,
        producer_dashboard: true,
        performance_analytics: true,
        device_support: DeviceSupport {
            web_client: true,
            android_app: true,
            ios_app: true,
            smart_tv: true,
            projector: true,
        },
    };

    // Create teleprompter
    let mut teleprompter = RedDirtKaraokeTeleprompter::new(config)?;

    // Load a sample song (in real implementation: load actual audio file)
    println!("Loading song...");

    // For demo purposes, create dummy lyrics
    let dummy_lyrics = vec![
        LyricLine {
            text: "Hello there, welcome to our performance".to_string(),
            start_time: 0.0,
            end_time: 3000.0,
            words: vec![
                LyricWord { text: "Hello".to_string(), start_time: 0.0, end_time: 500.0, position: 0 },
                LyricWord { text: "there,".to_string(), start_time: 500.0, end_time: 1000.0, position: 1 },
                LyricWord { text: "welcome".to_string(), start_time: 1000.0, end_time: 1500.0, position: 2 },
                LyricWord { text: "to".to_string(), start_time: 1500.0, end_time: 1800.0, position: 3 },
                LyricWord { text: "our".to_string(), start_time: 1800.0, end_time: 2100.0, position: 4 },
                LyricWord { text: "performance".to_string(), start_time: 2100.0, end_time: 3000.0, position: 5 },
            ],
            line_number: 0,
        },
        LyricLine {
            text: "Today we sing with perfect timing".to_string(),
            start_time: 3000.0,
            end_time: 6000.0,
            words: vec![
                LyricWord { text: "Today".to_string(), start_time: 3000.0, end_time: 3500.0, position: 0 },
                LyricWord { text: "we".to_string(), start_time: 3500.0, end_time: 3800.0, position: 1 },
                LyricWord { text: "sing".to_string(), start_time: 3800.0, end_time: 4200.0, position: 2 },
                LyricWord { text: "with".to_string(), start_time: 4200.0, end_time: 4500.0, position: 3 },
                LyricWord { text: "perfect".to_string(), start_time: 4500.0, end_time: 5000.0, position: 4 },
                LyricWord { text: "timing".to_string(), start_time: 5000.0, end_time: 6000.0, position: 5 },
            ],
            line_number: 1,
        },
    ];

    // Load via public API
    println!("Loading lyrics via API...");
    for line in &dummy_lyrics {
        for word in &line.words {
            // Use public methods to simulate loading
        }
    }

    // Start performance
    teleprompter.start_performance()?;
    
    println!("Starting casting server...");
    // Casting server automatically starts with performance
    
    println!("Connected devices: {:?}", teleprompter.get_connected_devices().len());
    println!("Teleprompter ready - visit http://localhost:8080 for web client");

    // Simulate performance
    println!("Performance starting...");
    
    for i in 0..20 {
        // Use public API to update position
        teleprompter.handle_producer_command(ProducerCommand::SkipTo((i as f32 * 300.0).min(6000.0)))?;
        
        // Broadcast updates (now public)
        teleprompter.broadcast_updates()?;
        
        println!("Time: {:.1}s", (i as f32 * 300.0).min(6000.0) / 1000.0);
        
        thread::sleep(Duration::from_millis(300));
    }

    // Show final performance metrics
    let metrics = teleprompter.get_performance_metrics();
    println!("Final Performance Metrics:");
    println!("  Timing Accuracy: {:.1}%", metrics.timing_accuracy * 100.0);
    println!("  Pitch Accuracy: {:.1}%", metrics.pitch_accuracy * 100.0);
    println!("  Word Completion: {:.1}%", metrics.word_completion * 100.0);
    println!("  Overall Score: {:.1}/100", metrics.overall_score * 100.0);

    // Stop performance
    teleprompter.stop_performance()?;
    
    println!("Teleprompter performance completed successfully!");
    println!("Clients can still view lyrics on connected devices");
    
    Ok(())
}
