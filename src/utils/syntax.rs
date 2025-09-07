use ratatui::prelude::*;
use std::collections::HashMap;

/// Simple syntax highlighting utility
pub struct SyntaxHighlighter {
    language_configs: HashMap<String, LanguageConfig>,
}

#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub keywords: Vec<String>,
    pub string_delimiters: Vec<char>,
    pub comment_prefixes: Vec<String>,
    pub keyword_style: Style,
    pub string_style: Style,
    pub comment_style: Style,
    pub number_style: Style,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        let mut highlighter = Self {
            language_configs: HashMap::new(),
        };

        // Add common language configurations
        highlighter.add_rust_config();
        highlighter.add_python_config();
        highlighter.add_javascript_config();

        highlighter
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn highlight_text(&self, text: &str, language: &str) -> Vec<Span> {
        if let Some(config) = self.language_configs.get(language) {
            self.highlight_with_config(text, config)
        } else {
            // Return plain text
            vec![Span::styled(text, Style::default())]
        }
    }

    fn highlight_with_config(&self, text: &str, config: &LanguageConfig) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut current_pos = 0;

        for token in self.tokenize(text) {
            let style = match token.token_type {
                TokenType::Keyword => {
                    if config.keywords.contains(&token.text.to_lowercase()) {
                        config.keyword_style
                    } else {
                        Style::default()
                    }
                }
                TokenType::String => config.string_style,
                TokenType::Comment => config.comment_style,
                TokenType::Number => config.number_style,
                TokenType::Text => Style::default(),
            };

            spans.push(Span::styled(token.text, style));
        }

        if spans.is_empty() {
            spans.push(Span::styled(text, Style::default()));
        }

        spans
    }

    fn tokenize(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let mut string_delimiter = '"';

        for ch in text.chars() {
            if in_comment {
                current_token.push(ch);
                if ch == '\n' {
                    tokens.push(Token {
                        text: current_token.clone(),
                        token_type: TokenType::Comment,
                    });
                    current_token.clear();
                    in_comment = false;
                }
            } else if in_string {
                current_token.push(ch);
                if ch == string_delimiter {
                    tokens.push(Token {
                        text: current_token.clone(),
                        token_type: TokenType::String,
                    });
                    current_token.clear();
                    in_string = false;
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        if !current_token.is_empty() {
                            tokens.push(Token {
                                text: current_token.clone(),
                                token_type: self.classify_token(&current_token),
                            });
                            current_token.clear();
                        }
                        current_token.push(ch);
                        string_delimiter = ch;
                        in_string = true;
                    }
                    '/' if text.chars().skip_while(|&c| c != ch).nth(1) == Some('/') => {
                        // Start of comment
                        if !current_token.is_empty() {
                            tokens.push(Token {
                                text: current_token.clone(),
                                token_type: self.classify_token(&current_token),
                            });
                            current_token.clear();
                        }
                        current_token.push(ch);
                        in_comment = true;
                    }
                    ' ' | '\t' | '\n' | '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' => {
                        if !current_token.is_empty() {
                            tokens.push(Token {
                                text: current_token.clone(),
                                token_type: self.classify_token(&current_token),
                            });
                            current_token.clear();
                        }
                        if !ch.is_whitespace() {
                            tokens.push(Token {
                                text: ch.to_string(),
                                token_type: TokenType::Text,
                            });
                        } else {
                            tokens.push(Token {
                                text: ch.to_string(),
                                token_type: TokenType::Text,
                            });
                        }
                    }
                    _ => {
                        current_token.push(ch);
                    }
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(Token {
                text: current_token,
                token_type: if in_string {
                    TokenType::String
                } else if in_comment {
                    TokenType::Comment
                } else {
                    self.classify_token(&current_token)
                },
            });
        }

        tokens
    }

    fn classify_token(&self, token: &str) -> TokenType {
        if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
            TokenType::Number
        } else {
            TokenType::Keyword // Will be filtered by language-specific keywords later
        }
    }

    fn add_rust_config(&mut self) {
        let config = LanguageConfig {
            keywords: vec![
                "fn".to_string(),
                "let".to_string(),
                "mut".to_string(),
                "const".to_string(),
                "struct".to_string(),
                "enum".to_string(),
                "impl".to_string(),
                "trait".to_string(),
                "pub".to_string(),
                "use".to_string(),
                "mod".to_string(),
                "crate".to_string(),
                "if".to_string(),
                "else".to_string(),
                "match".to_string(),
                "for".to_string(),
                "while".to_string(),
                "loop".to_string(),
                "return".to_string(),
                "break".to_string(),
                "continue".to_string(),
                "async".to_string(),
                "await".to_string(),
            ],
            string_delimiters: vec!['"', '\''],
            comment_prefixes: vec!["//".to_string(), "/*".to_string()],
            keyword_style: Style::default().fg(Color::Blue).bold(),
            string_style: Style::default().fg(Color::Green),
            comment_style: Style::default().fg(Color::Gray),
            number_style: Style::default().fg(Color::Magenta),
        };

        self.language_configs.insert("rust".to_string(), config);
        self.language_configs
            .insert("rs".to_string(), config.clone());
    }

    fn add_python_config(&mut self) {
        let config = LanguageConfig {
            keywords: vec![
                "def".to_string(),
                "class".to_string(),
                "import".to_string(),
                "from".to_string(),
                "if".to_string(),
                "elif".to_string(),
                "else".to_string(),
                "for".to_string(),
                "while".to_string(),
                "try".to_string(),
                "except".to_string(),
                "finally".to_string(),
                "with".to_string(),
                "as".to_string(),
                "return".to_string(),
                "yield".to_string(),
                "lambda".to_string(),
                "async".to_string(),
                "await".to_string(),
            ],
            string_delimiters: vec!['"', '\''],
            comment_prefixes: vec!["#".to_string()],
            keyword_style: Style::default().fg(Color::Blue).bold(),
            string_style: Style::default().fg(Color::Green),
            comment_style: Style::default().fg(Color::Gray),
            number_style: Style::default().fg(Color::Magenta),
        };

        self.language_configs
            .insert("python".to_string(), config.clone());
        self.language_configs.insert("py".to_string(), config);
    }

    fn add_javascript_config(&mut self) {
        let config = LanguageConfig {
            keywords: vec![
                "function".to_string(),
                "var".to_string(),
                "let".to_string(),
                "const".to_string(),
                "if".to_string(),
                "else".to_string(),
                "for".to_string(),
                "while".to_string(),
                "do".to_string(),
                "switch".to_string(),
                "case".to_string(),
                "default".to_string(),
                "try".to_string(),
                "catch".to_string(),
                "finally".to_string(),
                "throw".to_string(),
                "return".to_string(),
                "break".to_string(),
                "continue".to_string(),
                "class".to_string(),
                "extends".to_string(),
                "import".to_string(),
                "export".to_string(),
                "async".to_string(),
                "await".to_string(),
            ],
            string_delimiters: vec!['"', '\'', '`'],
            comment_prefixes: vec!["//".to_string(), "/*".to_string()],
            keyword_style: Style::default().fg(Color::Blue).bold(),
            string_style: Style::default().fg(Color::Green),
            comment_style: Style::default().fg(Color::Gray),
            number_style: Style::default().fg(Color::Magenta),
        };

        self.language_configs
            .insert("javascript".to_string(), config.clone());
        self.language_configs
            .insert("js".to_string(), config.clone());
        self.language_configs
            .insert("typescript".to_string(), config.clone());
        self.language_configs.insert("ts".to_string(), config);
    }
}

#[derive(Debug, Clone)]
struct Token {
    text: String,
    token_type: TokenType,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,
    String,
    Comment,
    Number,
    Text,
}

/// Detect language from file extension
pub fn detect_language_from_extension(filename: &str) -> Option<String> {
    let extension = std::path::Path::new(filename).extension()?.to_str()?;

    match extension {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" => Some("javascript".to_string()),
        "ts" => Some("typescript".to_string()),
        "json" => Some("json".to_string()),
        "toml" => Some("toml".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "md" => Some("markdown".to_string()),
        "html" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        _ => None,
    }
}
