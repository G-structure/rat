use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

/// Terminal utilities for cross-platform terminal operations
pub struct TerminalUtils;

impl TerminalUtils {
    /// Initialize terminal for TUI mode
    pub fn init() -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(())
    }

    /// Restore terminal to normal mode
    pub fn restore() -> Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Get terminal size
    pub fn size() -> Result<(u16, u16)> {
        Ok(terminal::size()?)
    }

    /// Clear the screen
    pub fn clear() -> Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    /// Move cursor to position
    pub fn move_cursor(x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }

    /// Print colored text
    pub fn print_colored(text: &str, color: Color) -> Result<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(color),
            Print(text),
            ResetColor
        )?;
        Ok(())
    }

    /// Check if a key event is available
    pub fn poll_key(timeout: std::time::Duration) -> Result<bool> {
        Ok(event::poll(timeout)?)
    }

    /// Read a key event
    pub fn read_key() -> Result<Option<KeyEvent>> {
        match event::read()? {
            Event::Key(key) => Ok(Some(key)),
            _ => Ok(None),
        }
    }

    /// Wait for any key press
    pub fn wait_for_key() -> Result<KeyEvent> {
        loop {
            if let Event::Key(key) = event::read()? {
                return Ok(key);
            }
        }
    }

    /// Check if terminal supports colors
    pub fn supports_colors() -> bool {
        // Simple heuristic - could be more sophisticated
        std::env::var("TERM")
            .map(|term| !term.is_empty() && term != "dumb")
            .unwrap_or(false)
    }

    /// Get available color count
    pub fn color_count() -> u16 {
        if let Ok(colors) = std::env::var("COLORS") {
            colors.parse().unwrap_or(8)
        } else if Self::supports_colors() {
            256 // Assume 256-color support
        } else {
            8 // Basic color support
        }
    }

    /// Check if terminal supports unicode
    pub fn supports_unicode() -> bool {
        std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_CTYPE"))
            .map(|locale| locale.contains("UTF-8") || locale.contains("utf8"))
            .unwrap_or(false)
    }
}

/// Terminal capabilities detection
pub struct TerminalCapabilities {
    pub colors: u16,
    pub supports_unicode: bool,
    pub supports_mouse: bool,
    pub width: u16,
    pub height: u16,
}

impl TerminalCapabilities {
    pub fn detect() -> Result<Self> {
        let (width, height) = TerminalUtils::size()?;

        Ok(Self {
            colors: TerminalUtils::color_count(),
            supports_unicode: TerminalUtils::supports_unicode(),
            supports_mouse: true, // Assume mouse support for now
            width,
            height,
        })
    }

    pub fn is_suitable_for_tui(&self) -> bool {
        self.width >= 80 && self.height >= 24 && self.colors >= 8
    }
}

/// ANSI escape sequence utilities
pub struct AnsiUtils;

impl AnsiUtils {
    /// Strip ANSI escape sequences from text
    pub fn strip_ansi(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    while let Some(ch) = chars.next() {
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Calculate display width of text (accounting for ANSI sequences)
    pub fn display_width(text: &str) -> usize {
        Self::strip_ansi(text).chars().count()
    }

    /// Truncate text to fit in specified width
    pub fn truncate_to_width(text: &str, width: usize) -> String {
        let stripped = Self::strip_ansi(text);
        if stripped.chars().count() <= width {
            text.to_string()
        } else {
            stripped
                .chars()
                .take(width.saturating_sub(1))
                .collect::<String>()
                + "…"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let text_with_ansi = "\x1b[31mRed text\x1b[0m normal";
        let stripped = AnsiUtils::strip_ansi(text_with_ansi);
        assert_eq!(stripped, "Red text normal");
    }

    #[test]
    fn test_display_width() {
        let text_with_ansi = "\x1b[31mHello\x1b[0m";
        assert_eq!(AnsiUtils::display_width(text_with_ansi), 5);
    }

    #[test]
    fn test_truncate_to_width() {
        let text = "Hello, world!";
        let truncated = AnsiUtils::truncate_to_width(text, 8);
        assert_eq!(truncated, "Hello, …");
    }
}
