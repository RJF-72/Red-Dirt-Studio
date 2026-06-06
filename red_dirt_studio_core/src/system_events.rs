use serde::{Serialize, Deserialize};

/// Event emitted when a model is upgraded or approved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Model has been upgraded and is ready for use
    ModelUpgraded {
        instrument: String,
        version: String,
        performance_score: f32,
        style_confidence: f32,
    },
    /// Model is ready for owner approval
    ModelReadyForApproval {
        instrument: String,
        version: String,
        report: ReadinessReport,
    },
    /// Apprentice has learned from songs
    ApprenticeUpdated {
        version: String,
        songs_learned: usize,
    },
    /// New instrument suggested for training
    NextInstrumentSuggested {
        instrument: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub instrument: String,
    pub version: String,
    pub performance_score: f32,
    pub style_confidence: f32,
    pub is_ready: bool,
    pub issues: Vec<String>,
}

/// Event bus for system-wide notifications
pub struct EventBus {
    listeners: Vec<fn(&SystemEvent)>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { listeners: Vec::new() }
    }

    pub fn subscribe(&mut self, callback: fn(&SystemEvent)) {
        self.listeners.push(callback);
    }

    pub fn emit(&self, event: &SystemEvent) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}