//! Centralized expressive glyph palette for the TUI.
//! Uses broadly supported Unicode box drawing and symbols (no Nerd Font).

#[derive(Clone, Debug)]
pub struct Glyphs {
    pub sep: char,           // primary separator
    pub thin_sep: char,      // thin separator
    pub bullet: char,        // list bullet
    pub diamond: char,       // accent marker
    pub chev_left: char,
    pub chev_right: char,
    pub corner_left: char,
    pub corner_right: char,
    pub clock: &'static str,
    pub agent: &'static str,
    pub link: &'static str,
}

impl Default for Glyphs {
    fn default() -> Self {
        Self {
            sep: '┃',
            thin_sep: '│',
            bullet: '•',
            diamond: '◆',
            chev_left: '‹',
            chev_right: '›',
            corner_left: '⟦',
            corner_right: '⟧',
            clock: "⌚",
            agent: "🤖",
            link: "🔗",
        }
    }
}

pub fn glyphs() -> Glyphs {
    Glyphs::default()
}

