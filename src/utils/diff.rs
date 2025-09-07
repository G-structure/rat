use anyhow::Result;

/// Simple diff utility for generating file differences
pub struct DiffGenerator;

impl DiffGenerator {
    pub fn generate_diff(original: &str, modified: &str) -> String {
        let original_lines: Vec<&str> = original.lines().collect();
        let modified_lines: Vec<&str> = modified.lines().collect();

        let mut diff = String::new();
        diff.push_str("--- original\n");
        diff.push_str("+++ modified\n");

        // Simple line-by-line diff (would use proper diff algorithm in production)
        let max_lines = original_lines.len().max(modified_lines.len());

        for i in 0..max_lines {
            let original_line = original_lines.get(i).unwrap_or(&"");
            let modified_line = modified_lines.get(i).unwrap_or(&"");

            if original_line != modified_line {
                if !original_line.is_empty() {
                    diff.push_str(&format!("-{}\n", original_line));
                }
                if !modified_line.is_empty() {
                    diff.push_str(&format!("+{}\n", modified_line));
                }
            } else {
                diff.push_str(&format!(" {}\n", original_line));
            }
        }

        diff
    }

    pub fn parse_diff(diff_text: &str) -> Result<Vec<DiffHunk>> {
        let mut hunks = Vec::new();
        let mut current_hunk = None;

        for line in diff_text.lines() {
            if line.starts_with("@@") {
                // Start of new hunk
                if let Some(hunk) = current_hunk.take() {
                    hunks.push(hunk);
                }
                current_hunk = Some(DiffHunk {
                    header: line.to_string(),
                    lines: Vec::new(),
                });
            } else if let Some(ref mut hunk) = current_hunk {
                hunk.lines.push(DiffLine::from_str(line));
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        Ok(hunks)
    }
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}

impl DiffLine {
    pub fn from_str(line: &str) -> Self {
        if line.starts_with('+') {
            Self {
                content: line[1..].to_string(),
                line_type: DiffLineType::Added,
            }
        } else if line.starts_with('-') {
            Self {
                content: line[1..].to_string(),
                line_type: DiffLineType::Removed,
            }
        } else {
            Self {
                content: line.to_string(),
                line_type: DiffLineType::Context,
            }
        }
    }
}
