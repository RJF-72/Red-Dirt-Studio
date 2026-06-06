use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprenticeConfig {
    pub buffer_size: usize,              // max songs to keep in replay buffer
    pub learning_rate: f32,
    pub batch_size: usize,               // songs per training step
    pub update_frequency: usize,         // train every N new songs
    pub auto_train_enabled: bool,
    pub style_strength_default: f32,     // 0.0..1.0
    pub checkpoint_dir: String,
}

impl Default for ApprenticeConfig {
    fn default() -> Self {
        Self {
            buffer_size: 500,
            learning_rate: 0.001,
            batch_size: 8,
            update_frequency: 5,
            auto_train_enabled: true,
            style_strength_default: 0.7,
            checkpoint_dir: "./apprentice_checkpoints".to_string(),
        }
    }
}
