use rat::utils::{
    diff::{DiffGenerator, DiffLineType},
    syntax::{detect_language_from_extension, SyntaxHighlighter},
    terminal::AnsiUtils,
};

#[test]
fn test_diff_generation() {
    let original = "Hello\nWorld\n";
    let modified = "Hello\nRust\nWorld\n";

    let diff = DiffGenerator::generate_diff(original, modified);
    assert!(diff.contains("--- original"));
    assert!(diff.contains("+++ modified"));
}

#[test]
fn test_language_detection() {
    assert_eq!(
        detect_language_from_extension("main.rs"),
        Some("rust".to_string())
    );
    assert_eq!(
        detect_language_from_extension("script.py"),
        Some("python".to_string())
    );
    assert_eq!(
        detect_language_from_extension("app.js"),
        Some("javascript".to_string())
    );
    assert_eq!(
        detect_language_from_extension("config.toml"),
        Some("toml".to_string())
    );
    assert_eq!(detect_language_from_extension("unknown.xyz"), None);
}

#[test]
fn test_syntax_highlighter() {
    let highlighter = SyntaxHighlighter::new();
    let spans = highlighter.highlight_text("fn main() {}", "rust");
    assert!(!spans.is_empty());
}

#[test]
fn test_ansi_utils() {
    let text_with_ansi = "\x1b[31mRed text\x1b[0m normal";
    let stripped = AnsiUtils::strip_ansi(text_with_ansi);
    assert_eq!(stripped, "Red text normal");

    let width = AnsiUtils::display_width(text_with_ansi);
    assert_eq!(width, 13); // "Red text normal".len()

    let truncated = AnsiUtils::truncate_to_width("Hello, world!", 8);
    assert_eq!(truncated, "Hello, …");
}
