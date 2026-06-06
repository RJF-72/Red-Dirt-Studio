use crate::{TeleprompterConfig, TeleprompterError, SongPosition, ProducerControls};
use std::time::Instant;

pub struct AudioSynchronizer {
    config: AudioSyncConfig,
    current_position: SongPosition,
    pub(crate) is_syncing: bool,
    start_instant: Option<Instant>,
    producer_controls: ProducerControls,
}

#[derive(Debug, Clone)]
pub struct AudioSyncConfig {
    pub sample_rate: f32,
    pub buffer_size: usize,
}

impl AudioSynchronizer {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self {
            config: AudioSyncConfig {
                sample_rate: config.sample_rate,
                buffer_size: config.buffer_size,
            },
            current_position: SongPosition {
                time_ms: 0.0,
                current_line: 0,
                current_word: 0,
                progress: 0.0,
            },
            is_syncing: false,
            start_instant: None,
            producer_controls: ProducerControls {
                playback_speed: 1.0,
                volume: 0.8,
                tempo_adjustment: 0.0,
                skip_to_ms: None,
                paused: false,
                highlight_advance_ms: 50.0,
            },
        })
    }

    pub fn start_synchronization(&mut self, _lyrics: &[crate::LyricLine], sample_rate: f32) -> Result<(), TeleprompterError> {
        self.is_syncing = true;
        self.config.sample_rate = sample_rate;
        self.start_instant = Some(Instant::now());
        self.current_position = SongPosition {
            time_ms: 0.0,
            current_line: 0,
            current_word: 0,
            progress: 0.0,
        };
        Ok(())
    }

    pub fn get_current_analysis(&self) -> Result<AudioAnalysis, TeleprompterError> {
        let elapsed_ms = if let Some(start) = self.start_instant {
            let raw_elapsed = start.elapsed().as_secs_f32() * 1000.0;
            if self.producer_controls.paused {
                self.current_position.time_ms
            } else {
                // Apply playback speed and any skip offset
                let skip_offset = self.producer_controls.skip_to_ms.unwrap_or(0.0);
                raw_elapsed * self.producer_controls.playback_speed + skip_offset
            }
        } else {
            0.0
        };

        Ok(AudioAnalysis {
            time_ms: elapsed_ms,
            peak_level: 0.0,
            rms_level: 0.0,
        })
    }

    pub fn apply_producer_controls(&mut self, controls: &ProducerControls) -> Result<(), TeleprompterError> {
        self.producer_controls = controls.clone();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), TeleprompterError> {
        self.is_syncing = false;
        self.start_instant = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub time_ms: f32,
    pub peak_level: f32,
    pub rms_level: f32,
}
