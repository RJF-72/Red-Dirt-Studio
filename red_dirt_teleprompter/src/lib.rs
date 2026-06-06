//! # Red Dirt Karaoke Teleprompter
//! The world's most advanced karaoke-style teleprompter — combining neural lyric alignment,
//! real-time synchronization, and multi-device casting for professional performance.

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

mod neural;
mod audio;
mod teleprompter;
mod casting;
mod producer;
mod gui;
mod network;
mod api;

pub use neural::*;
pub use audio::*;
pub use teleprompter::*;
pub use casting::*;
pub use producer::*;
pub use gui::*;
pub use network::*;
pub use api::*;

/// Teleprompter system configuration
#[derive(Debug, Clone)]
pub struct TeleprompterConfig {
    /// Sample rate for audio analysis
    pub sample_rate: f32,
    
    /// Buffer size for real-time processing
    pub buffer_size: usize,
    
    /// Neural model resolution
    pub neural_resolution: usize,
    
    /// Timing precision (microseconds)
    pub timing_precision: u32,
    
    /// Highlight prediction window (milliseconds)
    pub highlight_prediction_ms: f32,
    
    /// Network streaming enabled
    pub network_streaming: bool,
    
    /// Producer dashboard enabled
    pub producer_dashboard: bool,
    
    /// Performance analytics enabled
    pub performance_analytics: bool,
    
    /// Cross-platform device support
    pub device_support: DeviceSupport,
}

#[derive(Debug, Clone)]
pub struct DeviceSupport {
    pub web_client: bool,
    pub android_app: bool,
    pub ios_app: bool,
    pub smart_tv: bool,
    pub projector: bool,
}

impl Default for DeviceSupport {
    fn default() -> Self {
        Self {
            web_client: true,
            android_app: true,
            ios_app: true,
            smart_tv: true,
            projector: true,
        }
    }
}

impl Default for TeleprompterConfig {
    fn default() -> Self {
        Self {
            sample_rate: 96000.0,
            buffer_size: 256,
            neural_resolution: 1024,
            timing_precision: 100, // 100 microseconds
            highlight_prediction_ms: 50.0, // 50ms ahead prediction
            network_streaming: true,
            producer_dashboard: true,
            performance_analytics: true,
            device_support: DeviceSupport::default(),
        }
    }
}

/// Main teleprompter system
pub struct RedDirtKaraokeTeleprompter {
    /// Configuration
    config: TeleprompterConfig,
    
    /// Neural alignment engine
    neural_aligner: Arc<RwLock<NeuralAligner>>,
    
    /// Audio synchronization
    audio_sync: Arc<RwLock<AudioSynchronizer>>,
    
    /// Teleprompter engine
    teleprompter_engine: Arc<RwLock<TeleprompterEngine>>,
    
    /// AI highlighter
    ai_highlighter: Arc<RwLock<AIHighlighter>>,
    
    /// Casting server
    casting_server: Arc<RwLock<CastingServer>>,
    
    /// Producer dashboard
    producer_dashboard: Arc<RwLock<ProducerDashboard>>,
    
    /// Performance analyzer
    performance_analyzer: Arc<RwLock<PerformanceAnalyzer>>,
    
    /// Network interface
    network_interface: Arc<RwLock<NetworkInterface>>,
    
    /// Current session state
    pub session: SessionState,

    /// System state
    state: SystemState,

    /// Throttle broadcasts to avoid network saturation
    last_broadcast: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemState {
    Idle,
    LoadingLyrics,
    SyncingMusic,
    Performing,
    Broadcasting,
    Paused,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    /// Current song title
    pub song_title: String,
    
    /// Current artist
    pub artist: String,
    
    /// Lyrics with timing data
    pub lyrics: Vec<LyricLine>,
    
    /// Current position in song
    pub current_position: SongPosition,
    
    /// Highlighted words
    pub highlighted_words: Vec<HighlightedWord>,
    
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    
    /// Connected devices
    pub connected_devices: Vec<ConnectedDevice>,
    
    /// Producer controls
    pub producer_controls: ProducerControls,
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    /// Line text
    pub text: String,
    
    /// Start time (milliseconds)
    pub start_time: f32,
    
    /// End time (milliseconds)
    pub end_time: f32,
    
    /// Individual word timings
    pub words: Vec<LyricWord>,
    
    /// Line number
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct LyricWord {
    /// Word text
    pub text: String,
    
    /// Start time (milliseconds)
    pub start_time: f32,
    
    /// End time (milliseconds)
    pub end_time: f32,
    
    /// Word position in line
    pub position: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SongPosition {
    /// Current time in song (milliseconds)
    pub time_ms: f32,
    
    /// Current line index
    pub current_line: usize,
    
    /// Current word index in line
    pub current_word: usize,
    
    /// Progress percentage
    pub progress: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HighlightedWord {
    /// Word text
    pub text: String,
    
    /// Line number
    pub line: usize,
    
    /// Word position
    pub position: usize,
    
    /// Highlight intensity (0.0-1.0)
    pub intensity: f32,
    
    /// Time until highlight (milliseconds)
    pub time_until_highlight: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceMetrics {
    /// Timing accuracy (0.0-1.0)
    pub timing_accuracy: f32,
    
    /// Pitch accuracy (0.0-1.0)
    pub pitch_accuracy: f32,
    
    /// Word completion rate (0.0-1.0)
    pub word_completion: f32,
    
    /// Overall performance score (0.0-1.0)
    pub overall_score: f32,
    
    /// Current mistakes
    pub mistakes: Vec<Mistake>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Mistake {
    /// Type of mistake
    pub mistake_type: MistakeType,
    
    /// Time of mistake
    pub time_ms: f32,
    
    /// Description
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum MistakeType {
    Timing,
    Pitch,
    WrongWord,
    SkippedWord,
    VolumeTooLow,
}

#[derive(Debug, Clone)]
pub struct ConnectedDevice {
    /// Device ID
    pub id: String,
    
    /// Device name
    pub name: String,
    
    /// Device type
    pub device_type: DeviceType,
    
    /// Connection status
    pub connected: bool,
    
    /// Last ping time
    pub last_ping: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    WebBrowser,
    AndroidApp,
    IOSApp,
    SmartTV,
    Projector,
    Tablet,
    Phone,
}

#[derive(Debug, Clone)]
pub struct ProducerControls {
    /// Playback speed (0.5x - 2.0x)
    pub playback_speed: f32,
    
    /// Volume control
    pub volume: f32,
    
    /// Tempo adjustment
    pub tempo_adjustment: f32,
    
    /// Skip to position
    pub skip_to_ms: Option<f32>,
    
    /// Pause/resume
    pub paused: bool,
    
    /// Highlight adjustment
    pub highlight_advance_ms: f32,
}

impl RedDirtKaraokeTeleprompter {
    /// Create new teleprompter system
    pub fn new(config: TeleprompterConfig) -> Result<Self, TeleprompterError> {
        let neural_aligner = Arc::new(RwLock::new(NeuralAligner::new(&config)?));
        let audio_sync = Arc::new(RwLock::new(AudioSynchronizer::new(&config)?));
        let teleprompter_engine = Arc::new(RwLock::new(TeleprompterEngine::new(&config)?));
        let ai_highlighter = Arc::new(RwLock::new(AIHighlighter::new(&config)?));
        let casting_server = Arc::new(RwLock::new(CastingServer::new(&config)?));
        let producer_dashboard = Arc::new(RwLock::new(ProducerDashboard::new(&config)?));
        let performance_analyzer = Arc::new(RwLock::new(PerformanceAnalyzer::new(&config)?));
        let network_interface = Arc::new(RwLock::new(NetworkInterface::new(&config)?));

        Ok(Self {
            config,
            neural_aligner,
            audio_sync,
            teleprompter_engine,
            ai_highlighter,
            casting_server,
            producer_dashboard,
            performance_analyzer,
            network_interface,
            session: SessionState {
                song_title: String::new(),
                artist: String::new(),
                lyrics: Vec::new(),
                current_position: SongPosition {
                    time_ms: 0.0,
                    current_line: 0,
                    current_word: 0,
                    progress: 0.0,
                },
                highlighted_words: Vec::new(),
                performance_metrics: PerformanceMetrics {
                    timing_accuracy: 0.0,
                    pitch_accuracy: 0.0,
                    word_completion: 0.0,
                    overall_score: 0.0,
                    mistakes: Vec::new(),
                },
                connected_devices: Vec::new(),
                producer_controls: ProducerControls {
                    playback_speed: 1.0,
                    volume: 0.8,
                    tempo_adjustment: 0.0,
                    skip_to_ms: None,
                    paused: false,
                    highlight_advance_ms: 50.0,
                },
            },
            state: SystemState::Idle,
            last_broadcast: Instant::now() - Duration::from_secs(1),
        })
    }

    /// Load song and lyrics
    pub fn load_song(&mut self, song_path: &str, lyrics_path: &str) -> Result<(), TeleprompterError> {
        self.state = SystemState::LoadingLyrics;
        
        // Load audio file
        let audio_data = self.load_audio_file(song_path)?;
        
        // Load lyrics file
        let lyrics = self.load_lyrics_file(lyrics_path)?;

        if lyrics.is_empty() {
            return Err(TeleprompterError::FileLoadError(
                format!("No valid lyric lines found in: {}", lyrics_path)
            ));
        }
        
        // Perform neural alignment
        let aligned_lyrics = self.neural_aligner.write().align_lyrics_to_audio(
            &lyrics,
            &audio_data,
        )?;
        
        // Update session
        self.session.lyrics = aligned_lyrics;
        self.session.song_title = self.extract_title_from_path(song_path);
        self.session.artist = self.extract_artist_from_path(song_path);
        
        Ok(())
    }

    /// Start performance with music synchronization
    pub fn start_performance(&mut self) -> Result<(), TeleprompterError> {
        self.state = SystemState::SyncingMusic;
        
        // Initialize audio synchronization
        self.audio_sync.write().start_synchronization(
            &self.session.lyrics,
            self.config.sample_rate,
        )?;
        
        // Start teleprompter engine
        self.teleprompter_engine.write().start(
            &self.session.lyrics,
            self.config.highlight_prediction_ms,
        )?;
        
        // Start casting server
        if self.config.network_streaming {
            self.casting_server.write().start()?;
        }
        
        self.state = SystemState::Performing;
        Ok(())
    }

    /// Update performance in real-time
    pub fn update_performance(&mut self) -> Result<(), TeleprompterError> {
        if self.state != SystemState::Performing {
            return Ok(());
        }

        // Get current audio analysis
        let audio_analysis = self.audio_sync.read().get_current_analysis()?;
        
        // Update teleprompter position (now requires lyrics to compute line/word)
        let current_position = {
            let lyrics = &self.session.lyrics;
            self.teleprompter_engine.write()
                .update_position(audio_analysis.time_ms, lyrics)?
        };
        
        // Update session position
        self.session.current_position = current_position.clone();
        
        // Get highlighted words
        let highlighted_words = self.ai_highlighter.read()
            .get_highlighted_words(&self.session.lyrics, &current_position)?;
        
        self.session.highlighted_words = highlighted_words;
        
        // Analyze performance
        let performance_metrics = self.performance_analyzer.write()
            .analyze_performance(&self.session.lyrics, &current_position)?;
        
        self.session.performance_metrics = performance_metrics.clone();
        
        // Update producer dashboard with actual time
        if self.config.performance_analytics {
            self.producer_dashboard.write()
                .update_performance(current_position.time_ms, &performance_metrics);
        }
        
        // Broadcast updates to connected devices (throttled to ~30 FPS max)
        let now = Instant::now();
        if now.duration_since(self.last_broadcast) >= Duration::from_millis(33) {
            self.broadcast_updates()?;
            self.last_broadcast = now;
        }
        
        Ok(())
    }

    /// Cast current state to connected devices
    pub fn broadcast_updates(&self) -> Result<(), TeleprompterError> {
        if !self.config.network_streaming {
            return Ok(());
        }

        let update = TeleprompterUpdate {
            position: self.session.current_position.clone(),
            highlighted_words: self.session.highlighted_words.clone(),
            performance_metrics: self.session.performance_metrics.clone(),
            song_title: self.session.song_title.clone(),
            artist: self.session.artist.clone(),
        };

        self.casting_server.read().broadcast_update(update)?;
        Ok(())
    }

    /// Handle producer commands
    pub fn handle_producer_command(&mut self, command: ProducerCommand) -> Result<(), TeleprompterError> {
        match command {
            ProducerCommand::SetPlaybackSpeed(speed) => {
                self.session.producer_controls.playback_speed = speed.clamp(0.5, 2.0);
            },
            ProducerCommand::SetVolume(volume) => {
                self.session.producer_controls.volume = volume.clamp(0.0, 1.0);
            },
            ProducerCommand::SkipTo(position_ms) => {
                self.session.producer_controls.skip_to_ms = Some(position_ms);
            },
            ProducerCommand::Pause => {
                self.session.producer_controls.paused = true;
            },
            ProducerCommand::Resume => {
                self.session.producer_controls.paused = false;
            },
            ProducerCommand::AdjustHighlight(ms) => {
                self.session.producer_controls.highlight_advance_ms = ms;
            },
            ProducerCommand::ResetPerformance => {
                self.reset_performance()?;
            },
        }

        // Apply changes to audio sync only if actively syncing
        if self.audio_sync.read().is_syncing {
            self.audio_sync.write().apply_producer_controls(&self.session.producer_controls)?;
        }
        
        Ok(())
    }

    /// Get current teleprompter state for rendering
    pub fn get_render_state(&self) -> RenderState {
        RenderState {
            current_line: self.session.current_position.current_line,
            current_word: self.session.current_position.current_word,
            highlighted_words: self.session.highlighted_words.clone(),
            lyrics: self.session.lyrics.clone(),
            performance_metrics: self.session.performance_metrics.clone(),
            time_ms: self.session.current_position.time_ms,
            progress: self.session.current_position.progress,
            song_title: self.session.song_title.clone(),
            artist: self.session.artist.clone(),
        }
    }

    /// Get connected devices
    pub fn get_connected_devices(&self) -> Vec<ConnectedDevice> {
        self.session.connected_devices.clone()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.session.performance_metrics.clone()
    }

    /// Reset performance for new take
    fn reset_performance(&mut self) -> Result<(), TeleprompterError> {
        self.session.performance_metrics = PerformanceMetrics {
            timing_accuracy: 0.0,
            pitch_accuracy: 0.0,
            word_completion: 0.0,
            overall_score: 0.0,
            mistakes: Vec::new(),
        };
        
        self.session.current_position = SongPosition {
            time_ms: 0.0,
            current_line: 0,
            current_word: 0,
            progress: 0.0,
        };

        // Reset producer controls to defaults as well
        self.session.producer_controls = ProducerControls {
            playback_speed: 1.0,
            volume: 0.8,
            tempo_adjustment: 0.0,
            skip_to_ms: None,
            paused: false,
            highlight_advance_ms: 50.0,
        };
        
        // Reset audio sync so it picks up the new controls
        self.audio_sync.write().apply_producer_controls(&self.session.producer_controls)?;
        
        Ok(())
    }

    /// Stop performance
    pub fn stop_performance(&mut self) -> Result<(), TeleprompterError> {
        self.state = SystemState::Idle;
        
        self.teleprompter_engine.write().stop()?;
        self.audio_sync.write().stop()?;
        self.casting_server.write().stop()?;
        
        Ok(())
    }

    /// Load audio file
    fn load_audio_file(&self, _path: &str) -> Result<Vec<f32>, TeleprompterError> {
        // Stub audio loading — in a real app, use rodio/symphonia/hound here.
        // Returning empty vec is acceptable for alignment because align_lyrics_to_audio
        // now falls back gracefully to original timings when the segment is empty.
        Ok(Vec::new())
    }

    /// Load lyrics file
    fn load_lyrics_file(&self, path: &str) -> Result<Vec<LyricLine>, TeleprompterError> {
        // Parse LRC format or similar
        let content = std::fs::read_to_string(path)?;
        
        let mut lines = Vec::new();
        let mut line_number = 0;
        
        for line in content.lines() {
            if let Some(parsed_line) = self.parse_lyric_line(line, line_number) {
                lines.push(parsed_line);
                line_number += 1;
            }
        }
        
        Ok(lines)
    }

    fn parse_lyric_line(&self, line: &str, line_number: usize) -> Option<LyricLine> {
        // Simple LRC format parser: [mm:ss.xx] text
        if line.starts_with('[') && line.contains(']') {
            let closing_bracket = line.find(']')?;
            let time_part = &line[1..closing_bracket];
            let text_part = &line[closing_bracket + 1..].trim();
            
            if text_part.is_empty() {
                return None;
            }
            
            let (minutes, seconds) = time_part.split_once(':')?;
            let min = minutes.parse::<f32>().ok()?;
            let sec = seconds.parse::<f32>().ok()?;
            let start_time = (min * 60.0 + sec) * 1000.0; // Convert to milliseconds
            
            // Parse words within the line
            let words: Vec<LyricWord> = text_part.split_whitespace()
                .enumerate()
                .map(|(pos, word)| LyricWord {
                    text: word.to_string(),
                    start_time: start_time + (pos as f32 * 300.0), // Approximate word timing
                    end_time: start_time + ((pos + 1) as f32 * 300.0),
                    position: pos,
                })
                .collect();
            
            let end_time = if let Some(last_word) = words.last() {
                last_word.end_time
            } else {
                start_time + (text_part.split_whitespace().count() as f32 * 300.0)
            };
            
            return Some(LyricLine {
                text: text_part.to_string(),
                start_time,
                end_time,
                words,
                line_number,
            });
        }
        
        None
    }

    fn extract_title_from_path(&self, path: &str) -> String {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }

    fn extract_artist_from_path(&self, path: &str) -> String {
        // Extract artist from filename pattern like "Artist - Song.mp3"
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Use splitn(2, " - ") to handle titles that contain dashes
        let mut parts = filename.splitn(2, " - ");
        if let Some(artist) = parts.next() {
            let artist = artist.trim();
            if !artist.is_empty() {
                return artist.to_string();
            }
        }
        
        "Unknown Artist".to_string()
    }
}

#[derive(Debug)]
pub enum TeleprompterError {
    InitializationFailed(String),
    AudioDecodeError,
    FileLoadError(String),
    NetworkError(String),
    NeuralModelError(String),
    ParameterOutOfRange,
    MemoryAllocationFailed,
}

impl From<std::io::Error> for TeleprompterError {
    fn from(e: std::io::Error) -> Self {
        TeleprompterError::FileLoadError(e.to_string())
    }
}

impl From<candle_core::Error> for TeleprompterError {
    fn from(e: candle_core::Error) -> Self {
        TeleprompterError::NeuralModelError(e.to_string())
    }
}

impl std::error::Error for TeleprompterError {}

impl std::fmt::Display for TeleprompterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeleprompterError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            TeleprompterError::AudioDecodeError => write!(f, "Audio decode error"),
            TeleprompterError::FileLoadError(msg) => write!(f, "File load error: {}", msg),
            TeleprompterError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TeleprompterError::NeuralModelError(msg) => write!(f, "Neural model error: {}", msg),
            TeleprompterError::ParameterOutOfRange => write!(f, "Parameter out of range"),
            TeleprompterError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TeleprompterUpdate {
    pub position: SongPosition,
    pub highlighted_words: Vec<HighlightedWord>,
    pub performance_metrics: PerformanceMetrics,
    pub song_title: String,
    pub artist: String,
}

#[derive(Debug, Clone)]
pub enum ProducerCommand {
    SetPlaybackSpeed(f32),
    SetVolume(f32),
    SkipTo(f32),
    Pause,
    Resume,
    AdjustHighlight(f32),
    ResetPerformance,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub current_line: usize,
    pub current_word: usize,
    pub highlighted_words: Vec<HighlightedWord>,
    pub lyrics: Vec<LyricLine>,
    pub performance_metrics: PerformanceMetrics,
    pub time_ms: f32,
    pub progress: f32,
    pub song_title: String,
    pub artist: String,
}
