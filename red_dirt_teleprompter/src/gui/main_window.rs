use eframe::egui;
use egui::epaint::{Color32, FontFamily};
use egui::{RichText, ProgressBar};

use crate::RenderState;

/// Main teleprompter window
pub struct MainWindow {
    /// Current render state
    render_state: RenderState,
    
    /// Theme configuration
    theme: ThemeConfig,
    
    /// Display settings
    display_settings: DisplaySettings,
    
    /// Animation state
    animation_state: AnimationState,
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub background_color: Color32,
    pub text_color: Color32,
    pub highlight_color: Color32,
    pub secondary_color: Color32,
    pub font_family: FontFamily,
    pub base_font_size: f32,
}

#[derive(Debug, Clone)]
pub struct DisplaySettings {
    pub line_spacing: f32,
    pub word_spacing: f32,
    pub highlight_intensity: f32,
    pub animation_speed: f32,
    pub blur_background: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub scroll_position: f32,
    pub highlight_animation: f32,
    pub fade_animation: f32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: Color32::from_rgb(10, 10, 20),
            text_color: Color32::from_rgb(220, 220, 220),
            highlight_color: Color32::from_rgb(255, 215, 0), // Gold
            secondary_color: Color32::from_rgb(100, 100, 150),
            font_family: FontFamily::Proportional,
            base_font_size: 48.0,
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            line_spacing: 1.2,
            word_spacing: 0.1,
            highlight_intensity: 1.0,
            animation_speed: 1.0,
            blur_background: true,
        }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            scroll_position: 0.0,
            highlight_animation: 0.0,
            fade_animation: 1.0,
        }
    }
}

impl MainWindow {
    pub fn new(render_state: RenderState) -> Self {
        Self {
            render_state,
            theme: ThemeConfig::default(),
            display_settings: DisplaySettings::default(),
            animation_state: AnimationState::default(),
        }
    }

    /// Update render state
    pub fn update_render_state(&mut self, render_state: RenderState) {
        self.render_state = render_state;
    }

    /// Render the main teleprompter window
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Apply theme
            ui.style_mut().visuals.extreme_bg_color = self.theme.background_color;
            ui.style_mut().visuals.override_text_color = Some(self.theme.text_color);

            // Main content area
            ui.vertical_centered(|ui| {
                // Title display
                if !self.render_state.song_title.is_empty() {
                    ui.heading(RichText::new(&self.render_state.song_title)
                        .color(self.theme.secondary_color)
                        .size(self.theme.base_font_size * 0.8));
                }

                // Lyrics display with highlighting
                self.render_lyrics(ui);

                // Progress bar
                self.render_progress_bar(ui);

                // Performance metrics
                self.render_performance_metrics(ui);
            });
        });

        // Update animations
        self.update_animations(ctx);
    }

    fn render_lyrics(&mut self, ui: &mut egui::Ui) {
        let current_line = self.render_state.current_line;
        let current_word = self.render_state.current_word;

        // Show 3 lines: previous, current, next
        let start_line = current_line.saturating_sub(1);
        let end_line = (current_line + 2).min(self.render_state.lyrics.len());

        for line_idx in start_line..end_line {
            if line_idx >= self.render_state.lyrics.len() {
                continue;
            }

            let line = &self.render_state.lyrics[line_idx];
            let is_current = line_idx == current_line;

            ui.horizontal_wrapped(|ui| {
                for (word_idx, word) in line.words.iter().enumerate() {
                    let is_current_word = is_current && word_idx == current_word;
                    
                    // Check if this word should be highlighted
                    let highlight_intensity = self.render_state.highlighted_words
                        .iter()
                        .find(|hw| hw.line == line_idx && hw.position == word_idx)
                        .map(|hw| hw.intensity)
                        .unwrap_or(0.0);

                    // Create rich text with appropriate styling
                    let mut text_style = RichText::new(&word.text)
                        .family(self.theme.font_family.clone());

                    if is_current_word || highlight_intensity > 0.5 {
                        // Highlight current word
                        let highlight_color = self.interpolate_color(
                            self.theme.text_color,
                            self.theme.highlight_color,
                            highlight_intensity.min(self.display_settings.highlight_intensity),
                        );
                        text_style = text_style.color(highlight_color).strong();
                    } else {
                        text_style = text_style.color(self.theme.text_color);
                    }

                    // Apply animation effects
                    let alpha = if is_current_word {
                        1.0 + (self.animation_state.highlight_animation * 0.2)
                    } else {
                        1.0
                    };

                    text_style = text_style.size(self.theme.base_font_size * alpha);

                    ui.label(text_style);
                    ui.add_space(self.display_settings.word_spacing);
                }
            });

            if line_idx < end_line - 1 {
                ui.add_space(self.display_settings.line_spacing * self.theme.base_font_size);
            }
        }
    }

    fn render_progress_bar(&self, ui: &mut egui::Ui) {
        ui.separator();
        
        let progress = self.render_state.progress;
        let progress_text = format!("{:.1}%", progress * 100.0);
        
        ui.horizontal(|ui| {
            ui.label(progress_text);
            ui.add(ProgressBar::new(progress)
                .desired_width(ui.available_width() - 60.0));
        });
    }

    fn render_performance_metrics(&self, ui: &mut egui::Ui) {
        ui.separator();
        
        ui.horizontal(|ui| {
            let metrics = &self.render_state.performance_metrics;
            
            ui.label(format!("Timing: {:.1}%", metrics.timing_accuracy * 100.0));
            ui.separator();
            ui.label(format!("Pitch: {:.1}%", metrics.pitch_accuracy * 100.0));
            ui.separator();
            ui.label(format!("Words: {:.1}%", metrics.word_completion * 100.0));
            ui.separator();
            ui.label(format!("Score: {:.1}", metrics.overall_score * 100.0));
        });
    }

    fn interpolate_color(&self, start: Color32, end: Color32, factor: f32) -> Color32 {
        let factor = factor.clamp(0.0, 1.0);
        
        let r = (start.r() as f32 * (1.0 - factor) + end.r() as f32 * factor) as u8;
        let g = (start.g() as f32 * (1.0 - factor) + end.g() as f32 * factor) as u8;
        let b = (start.b() as f32 * (1.0 - factor) + end.b() as f32 * factor) as u8;
        let a = (start.a() as f32 * (1.0 - factor) + end.a() as f32 * factor) as u8;
        
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn update_animations(&mut self, ctx: &egui::Context) {
        // Update highlight animation
        self.animation_state.highlight_animation = (self.animation_state.highlight_animation + 
            0.02 * self.display_settings.animation_speed) % 1.0;
        
        // Update scroll position based on current position
        self.animation_state.scroll_position = self.render_state.progress;
        
        // Request repaint for smooth animations
        ctx.request_repaint();
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}
