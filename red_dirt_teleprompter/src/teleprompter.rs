use crate::{TeleprompterConfig, TeleprompterError, LyricLine, SongPosition, HighlightedWord, PerformanceMetrics, Mistake, MistakeType};

pub struct TeleprompterEngine {
    config: TeleprompterConfig,
    is_running: bool,
    current_position: SongPosition,
    highlight_window_ms: f32,
}

impl TeleprompterEngine {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self {
            config: config.clone(),
            is_running: false,
            current_position: SongPosition {
                time_ms: 0.0,
                current_line: 0,
                current_word: 0,
                progress: 0.0,
            },
            highlight_window_ms: config.highlight_prediction_ms,
        })
    }

    pub fn start(&mut self, _lyrics: &[LyricLine], highlight_prediction_ms: f32) -> Result<(), TeleprompterError> {
        self.is_running = true;
        self.current_position = SongPosition {
            time_ms: 0.0,
            current_line: 0,
            current_word: 0,
            progress: 0.0,
        };
        self.highlight_window_ms = highlight_prediction_ms;
        Ok(())
    }

    pub fn update_position(&mut self, time_ms: f32, lyrics: &[LyricLine]) -> Result<SongPosition, TeleprompterError> {
        self.current_position.time_ms = time_ms;

        // Compute current line and word based on actual lyric timings
        let mut current_line = 0usize;
        let mut current_word = 0usize;
        let total_duration_ms = lyrics.last().map(|l| l.end_time).unwrap_or(1.0).max(1.0);

        for (li, line) in lyrics.iter().enumerate() {
            if time_ms >= line.start_time && time_ms <= line.end_time {
                current_line = li;
                for (wi, word) in line.words.iter().enumerate() {
                    if time_ms >= word.start_time {
                        current_word = wi;
                    } else {
                        break;
                    }
                }
                break;
            } else if time_ms > line.end_time {
                current_line = li;
                current_word = line.words.len().saturating_sub(1);
            }
        }

        self.current_position.current_line = current_line;
        self.current_position.current_word = current_word;
        self.current_position.progress = (time_ms / total_duration_ms).min(1.0);

        Ok(self.current_position.clone())
    }

    pub fn stop(&mut self) -> Result<(), TeleprompterError> {
        self.is_running = false;
        Ok(())
    }
}

pub struct PerformanceAnalyzer {
    config: TeleprompterConfig,
}

impl PerformanceAnalyzer {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self { config: config.clone() })
    }

    pub fn analyze_performance(
        &self,
        lyrics: &[LyricLine],
        position: &SongPosition,
    ) -> Result<PerformanceMetrics, TeleprompterError> {
        let word_completion = if lyrics.is_empty() {
            1.0
        } else {
            let total_words: usize = lyrics.iter().map(|l| l.words.len()).sum();
            let completed_words: usize = lyrics.iter()
                .filter(|l| position.time_ms >= l.end_time)
                .map(|l| l.words.len())
                .sum();
            if total_words > 0 {
                completed_words as f32 / total_words as f32
            } else {
                1.0
            }
        };

        let mut mistakes = Vec::new();
        if position.time_ms > 0.0 && word_completion < 0.5 && position.current_line > 0 {
            mistakes.push(Mistake {
                mistake_type: MistakeType::Timing,
                time_ms: position.time_ms,
                description: "Potential timing deviation detected".to_string(),
            });
        }

        Ok(PerformanceMetrics {
            timing_accuracy: word_completion.max(0.5),
            pitch_accuracy: 0.8,
            word_completion,
            overall_score: word_completion * 0.9 + 0.1,
            mistakes,
        })
    }
}

pub struct AIHighlighter {
    config: TeleprompterConfig,
}

impl AIHighlighter {
    pub fn new(config: &TeleprompterConfig) -> Result<Self, TeleprompterError> {
        Ok(Self { config: config.clone() })
    }

    pub fn get_highlighted_words(
        &self,
        lyrics: &[LyricLine],
        position: &SongPosition,
    ) -> Result<Vec<HighlightedWord>, TeleprompterError> {
        let mut highlighted = Vec::new();
        let prediction_window = self.config.highlight_prediction_ms.max(0.0);

        for line in lyrics {
            if position.time_ms >= line.start_time && position.time_ms <= line.end_time {
                for word in &line.words {
                    let window_start = (word.start_time - prediction_window).max(0.0);
                    if position.time_ms >= window_start && position.time_ms <= word.end_time {
                        let time_until = (word.start_time - position.time_ms).max(0.0);
                        let intensity = if position.time_ms >= word.start_time {
                            1.0
                        } else {
                            (1.0 - time_until / prediction_window).max(0.0)
                        };

                        highlighted.push(HighlightedWord {
                            text: word.text.clone(),
                            line: line.line_number,
                            position: word.position,
                            intensity,
                            time_until_highlight: time_until,
                        });
                    }
                }
            }
        }

        Ok(highlighted)
    }
}
