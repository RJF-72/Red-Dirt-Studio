use std::sync::Arc;
use parking_lot::RwLock;

use crate::{TeleprompterConfig, TeleprompterError, RenderState, TeleprompterUpdate};

/// Public API for external teleprompter control.
pub struct TeleprompterApi {
    state: Arc<RwLock<RenderState>>,
}

impl TeleprompterApi {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self {
            state: Arc::new(RwLock::new(RenderState {
                current_line: 0,
                current_word: 0,
                highlighted_words: vec![],
                lyrics: vec![],
                performance_metrics: crate::PerformanceMetrics {
                    timing_accuracy: 0.0,
                    pitch_accuracy: 0.0,
                    word_completion: 0.0,
                    overall_score: 0.0,
                    mistakes: vec![],
                },
                time_ms: 0.0,
                progress: 0.0,
                song_title: String::new(),
                artist: String::new(),
            })),
        })
    }

    pub fn get_state(&self) -> RenderState {
        self.state.read().clone()
    }

    pub fn update_state(&self, state: RenderState) {
        *self.state.write() = state;
    }

    pub fn get_update(&self) -> TeleprompterUpdate {
        let st = self.state.read().clone();
        TeleprompterUpdate {
            position: crate::SongPosition {
                time_ms: st.time_ms,
                current_line: st.current_line,
                current_word: st.current_word,
                progress: st.progress,
            },
            highlighted_words: st.highlighted_words,
            performance_metrics: st.performance_metrics,
            song_title: st.song_title,
            artist: st.artist,
        }
    }
}
