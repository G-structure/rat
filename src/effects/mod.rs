// Effects system for visual enhancements using tachyonfx
// This module will be expanded in Phase 4

pub mod code;
pub mod text;
pub mod themes;
pub mod transitions;

// Placeholder types for now
pub struct EffectsManager {
    enabled: bool,
}

impl EffectsManager {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
