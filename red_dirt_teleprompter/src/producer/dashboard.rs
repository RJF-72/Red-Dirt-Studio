use std::sync::Arc;
use parking_lot::RwLock;

use crate::{TeleprompterConfig, TeleprompterError, ProducerControls, PerformanceMetrics};

/// Producer dashboard for controlling teleprompter without interrupting performance
pub struct ProducerDashboard {
    /// Current controls state
    controls: Arc<RwLock<ProducerControls>>,
    
    /// DAW integration
    daw_integration: Option<DAWIntegration>,
    
    /// Performance monitoring
    performance_monitor: PerformanceMonitor,
    
    /// Configuration
    config: ProducerConfig,
}

#[derive(Debug, Clone)]
pub struct ProducerConfig {
    pub enable_remote_control: bool,
    pub daw_integration: bool,
    pub performance_analytics: bool,
    pub hotkey_support: bool,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            enable_remote_control: true,
            daw_integration: true,
            performance_analytics: true,
            hotkey_support: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Real-time performance metrics
    pub metrics: PerformanceMetrics,
    
    /// Historical data
    pub historical_data: Vec<HistoricalMetric>,
    
    /// Peak detection
    pub peaks: Vec<PeakEvent>,
}

#[derive(Debug, Clone)]
pub struct HistoricalMetric {
    pub timestamp: f32, // Time in song (ms)
    pub timing_accuracy: f32,
    pub pitch_accuracy: f32,
    pub word_completion: f32,
}

#[derive(Debug, Clone)]
pub struct PeakEvent {
    pub timestamp: f32,
    pub event_type: PeakEventType,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub enum PeakEventType {
    ExcellentTiming,
    GoodPitch,
    PerfectWord,
    Mistake,
}

impl ProducerDashboard {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        let daw_integration = if config.producer_dashboard {
            Some(DAWIntegration::new()?)
        } else {
            None
        };

        Ok(Self {
            controls: Arc::new(RwLock::new(ProducerControls {
                playback_speed: 1.0,
                volume: 0.8,
                tempo_adjustment: 0.0,
                skip_to_ms: None,
                paused: false,
                highlight_advance_ms: 50.0,
            })),
            daw_integration,
            performance_monitor: PerformanceMonitor {
                metrics: PerformanceMetrics {
                    timing_accuracy: 0.0,
                    pitch_accuracy: 0.0,
                    word_completion: 0.0,
                    overall_score: 0.0,
                    mistakes: Vec::new(),
                },
                historical_data: Vec::new(),
                peaks: Vec::new(),
            },
            config: ProducerConfig::default(),
        })
    }

    /// Update performance metrics with current song time
    pub fn update_performance(&mut self, time_ms: f32, metrics: &PerformanceMetrics) {
        self.performance_monitor.metrics = metrics.clone();
        
        // Store historical data
        self.performance_monitor.historical_data.push(HistoricalMetric {
            timestamp: time_ms,
            timing_accuracy: metrics.timing_accuracy,
            pitch_accuracy: metrics.pitch_accuracy,
            word_completion: metrics.word_completion,
        });
    }

    /// Get current producer controls
    pub fn get_controls(&self) -> ProducerControls {
        self.controls.read().clone()
    }

    /// Set producer controls
    pub fn set_controls(&self, controls: ProducerControls) {
        *self.controls.write() = controls;
    }

    /// Send command to DAW
    pub fn send_daw_command(&self, command: DAWCommand) -> Result<(), TeleprompterError> {
        if let Some(ref daw) = self.daw_integration {
            daw.send_command(command)?;
        }
        Ok(())
    }

    /// Get DAW status
    pub fn get_daw_status(&self) -> Option<DAWStatus> {
        self.daw_integration.as_ref().and_then(|daw| daw.get_status().ok())
    }

    /// Handle hotkey
    pub fn handle_hotkey(&self, key: Hotkey) -> Result<(), TeleprompterError> {
        match key {
            Hotkey::TogglePause => {
                let mut controls = self.controls.write();
                controls.paused = !controls.paused;
            },
            Hotkey::SpeedUp => {
                let mut controls = self.controls.write();
                controls.playback_speed = (controls.playback_speed + 0.1).min(2.0);
            },
            Hotkey::SpeedDown => {
                let mut controls = self.controls.write();
                controls.playback_speed = (controls.playback_speed - 0.1).max(0.5);
            },
            Hotkey::VolumeUp => {
                let mut controls = self.controls.write();
                controls.volume = (controls.volume + 0.1).min(1.0);
            },
            Hotkey::VolumeDown => {
                let mut controls = self.controls.write();
                controls.volume = (controls.volume - 0.1).max(0.0);
            },
            Hotkey::SkipForward => {
                let mut controls = self.controls.write();
                if let Some(skip_to) = controls.skip_to_ms {
                    controls.skip_to_ms = Some(skip_to + 5000.0); // Skip 5 seconds forward
                } else {
                    controls.skip_to_ms = Some(5000.0);
                }
            },
            Hotkey::SkipBackward => {
                let mut controls = self.controls.write();
                if let Some(skip_to) = controls.skip_to_ms {
                    controls.skip_to_ms = Some((skip_to - 5000.0).max(0.0));
                } else {
                    controls.skip_to_ms = Some(0.0);
                }
            },
        }
        
        Ok(())
    }

    /// Get performance analytics
    pub fn get_performance_analytics(&self) -> PerformanceAnalytics {
        PerformanceAnalytics {
            current_metrics: self.performance_monitor.metrics.clone(),
            historical_average: self.calculate_historical_average(),
            trend: self.calculate_trend(),
            recommendations: self.generate_recommendations(),
        }
    }

    fn calculate_historical_average(&self) -> PerformanceMetrics {
        if self.performance_monitor.historical_data.is_empty() {
            return self.performance_monitor.metrics.clone();
        }

        let total_timing: f32 = self.performance_monitor.historical_data.iter()
            .map(|m| m.timing_accuracy).sum();
        let total_pitch: f32 = self.performance_monitor.historical_data.iter()
            .map(|m| m.pitch_accuracy).sum();
        let total_completion: f32 = self.performance_monitor.historical_data.iter()
            .map(|m| m.word_completion).sum();

        let count = self.performance_monitor.historical_data.len() as f32;

        PerformanceMetrics {
            timing_accuracy: total_timing / count,
            pitch_accuracy: total_pitch / count,
            word_completion: total_completion / count,
            overall_score: (total_timing + total_pitch + total_completion) / (count * 3.0),
            mistakes: Vec::new(), // Don't average mistakes
        }
    }

    fn calculate_trend(&self) -> PerformanceTrend {
        let len = self.performance_monitor.historical_data.len();
        if len < 20 {
            return PerformanceTrend::Neutral;
        }

        // Compare the most recent 10 entries with the 10 entries before them
        let recent_data = &self.performance_monitor.historical_data[len - 10..];
        let older_data = &self.performance_monitor.historical_data[len - 20..len - 10];
        
        let recent_avg: f32 = recent_data.iter().map(|m| m.timing_accuracy).sum::<f32>() / 10.0;
        let older_avg: f32 = older_data.iter().map(|m| m.timing_accuracy).sum::<f32>() / 10.0;

        let trend = recent_avg - older_avg;

        if trend > 0.1 {
            PerformanceTrend::Improving
        } else if trend < -0.1 {
            PerformanceTrend::Declining
        } else {
            PerformanceTrend::Neutral
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let metrics = &self.performance_monitor.metrics;
        
        if metrics.timing_accuracy < 0.7 {
            recommendations.push("Focus on timing - try slowing down the playback speed".to_string());
        }
        
        if metrics.pitch_accuracy < 0.7 {
            recommendations.push("Work on pitch accuracy - practice with the original recording".to_string());
        }
        
        if metrics.word_completion < 0.8 {
            recommendations.push("Improve word completion rate - spend more time on difficult passages".to_string());
        }
        
        recommendations
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceAnalytics {
    pub current_metrics: PerformanceMetrics,
    pub historical_average: PerformanceMetrics,
    pub trend: PerformanceTrend,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PerformanceTrend {
    Improving,
    Declining,
    Neutral,
}

#[derive(Debug, Clone)]
pub enum Hotkey {
    TogglePause,
    SpeedUp,
    SpeedDown,
    VolumeUp,
    VolumeDown,
    SkipForward,
    SkipBackward,
}

// DAW integration (placeholder implementation)
struct DAWIntegration {
    // In real implementation: connection to actual DAW
    connected: bool,
}

#[derive(Debug, Clone)]
pub enum DAWCommand {
    Play,
    Pause,
    Stop,
    Record,
    SetTempo(f32),
    SetVolume(f32),
    Seek(f32),
    SetPlaybackSpeed(f32),
}

#[derive(Debug, Clone)]
pub struct DAWStatus {
    pub playing: bool,
    pub recording: bool,
    pub current_time: f32,
    pub tempo: f32,
    pub volume: f32,
}

impl DAWIntegration {
    fn new() -> Result<Self, TeleprompterError> {
        // In real implementation: establish connection to DAW
        Ok(Self {
            connected: true,
        })
    }

    fn send_command(&self, command: DAWCommand) -> Result<(), TeleprompterError> {
        // In real implementation: send command to DAW
        println!("Sending DAW command: {:?}", command);
        Ok(())
    }

    fn get_status(&self) -> Result<DAWStatus, TeleprompterError> {
        // In real implementation: get current DAW status
        Ok(DAWStatus {
            playing: true,
            recording: false,
            current_time: 0.0,
            tempo: 120.0,
            volume: 0.8,
        })
    }
}
