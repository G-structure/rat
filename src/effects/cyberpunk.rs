use ratatui::style::{Color, Style};
use tachyonfx::{fx, Effect, EffectTimer, Interpolation, CellFilter, IntoEffect};
use tachyonfx::fx::Glitch;

// A bold, neon-infused color palette suitable for cyberpunk vibes
#[derive(Clone, Copy, Debug)]
pub struct CyberPalette {
    pub crust: Color,      // deepest background
    pub surface: Color,    // panels
    pub text: Color,       // primary text
    pub accent_a: Color,   // neon pink
    pub accent_b: Color,   // neon cyan
    pub accent_c: Color,   // electric purple
    pub warning: Color,
    pub success: Color,
}

impl Default for CyberPalette {
    fn default() -> Self {
        Self {
            crust: Color::from_u32(0x0b0f14),
            surface: Color::from_u32(0x121821),
            text: Color::from_u32(0xd6dde7),
            accent_a: Color::from_u32(0xff2e88), // magenta/pink
            accent_b: Color::from_u32(0x18e5ff), // cyan
            accent_c: Color::from_u32(0x8b5cf6), // purple
            warning: Color::from_u32(0xffb86c),
            success: Color::from_u32(0x50fa7b),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CyberTheme {
    pub palette: CyberPalette,
}

impl Default for CyberTheme {
    fn default() -> Self {
        Self { palette: CyberPalette::default() }
    }
}

impl CyberTheme {
    pub fn surface_style(&self) -> Style {
        Style::default().bg(self.palette.surface).fg(self.palette.text)
    }
    pub fn background_style(&self) -> Style {
        Style::default().bg(self.palette.crust).fg(self.palette.text)
    }
    pub fn border_active(&self) -> Style {
        Style::default().fg(self.palette.accent_b)
    }
    pub fn border_inactive(&self) -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn title_active(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_a)
            .add_modifier(ratatui::style::Modifier::BOLD)
    }
    pub fn title_inactive(&self) -> Style {
        Style::default().fg(Color::Gray)
    }
}

// Persistent outer-border neon pulse
pub fn neon_pulse_border(theme: &CyberTheme) -> Effect {
    let timer = EffectTimer::from_ms(1400, Interpolation::SineInOut);
    fx::repeating(
        fx::fade_from_fg(theme.palette.accent_b, timer)
            .with_filter(CellFilter::Outer(ratatui::layout::Margin::new(1, 1))),
    )
}

// Subtle foreground hue drift on text to make UI feel alive
pub fn subtle_hsl_drift() -> Effect {
    // Softly breathe the foreground between two nearby tones
    let breathe = fx::fade_to_fg(Color::from_u32(0xb3ecff), (1600, Interpolation::SineInOut))
        .with_filter(CellFilter::Text);
    fx::repeating(fx::ping_pong(breathe))
}

// Quick attention ping for newly added content areas
pub fn sweep_in_attention(accent: Color) -> Effect {
    fx::sweep_in(
        tachyonfx::Motion::LeftToRight,
        10,
        2,
        accent,
        EffectTimer::from_ms(420, Interpolation::QuadOut),
    )
    .with_filter(CellFilter::Inner(ratatui::layout::Margin::new(1, 1)))
}

// Short, intense glitch burst for text areas
pub fn glitch_burst() -> Effect {
    Glitch::builder()
        .cell_glitch_ratio(0.025)
        .action_start_delay_ms(0..100)
        .action_ms(60..140)
        .build()
        .into_effect()
        .with_filter(CellFilter::Text)
}
