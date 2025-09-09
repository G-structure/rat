// Effects system for visual enhancements using tachyonfx
// Phase 4 foundation: themes + common effects + manager wiring

pub mod code;
pub mod text;
pub mod themes;
pub mod transitions;
pub mod cyberpunk;

// Lightweight toggle for enabling effects globally (future use)
pub struct EffectsConfig {
    pub enabled: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
