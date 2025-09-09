use anyhow::Result;

/// Enhanced diff utility for generating file differences using Myers algorithm
pub struct DiffGenerator;

impl DiffGenerator {
    pub fn generate_diff(original: &str, modified: &str) -> String {
        let original_lines: Vec<&str> = original.lines().collect();
        let modified_lines: Vec<&str> = modified.lines().collect();

        let mut diff = String::new();
        diff.push_str("--- original\n");
        diff.push_str("+++ modified\n");

        let operations = Self::compute_diff(&original_lines, &modified_lines);
        let hunks = Self::group_into_hunks(&operations, &original_lines, &modified_lines);

        for hunk in hunks {
            diff.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.original_start, hunk.original_len, hunk.modified_start, hunk.modified_len
            ));

            for line in &hunk.lines {
                match line.line_type {
                    DiffLineType::Added => diff.push_str(&format!("+{}\n", line.content)),
                    DiffLineType::Removed => diff.push_str(&format!("-{}\n", line.content)),
                    DiffLineType::Context => diff.push_str(&format!(" {}\n", line.content)),
                }
            }
        }

        diff
    }

    /// Compute diff using simplified Myers algorithm
    fn compute_diff(original: &[&str], modified: &[&str]) -> Vec<DiffOperation> {
        let mut operations = Vec::new();

        let n = original.len();
        let m = modified.len();

        // Simple LCS-based diff implementation
        let lcs = Self::lcs(original, modified);
        let mut i = 0; // original index
        let mut j = 0; // modified index
        let mut lcs_idx = 0;

        while i < n || j < m {
            if lcs_idx < lcs.len()
                && i < n
                && j < m
                && original[i] == modified[j]
                && original[i] == lcs[lcs_idx]
            {
                // Common line
                operations.push(DiffOperation::Equal(i, j));
                i += 1;
                j += 1;
                lcs_idx += 1;
            } else if i < n && (lcs_idx >= lcs.len() || original[i] != lcs[lcs_idx]) {
                // Deletion
                operations.push(DiffOperation::Delete(i));
                i += 1;
            } else if j < m {
                // Insertion
                operations.push(DiffOperation::Insert(j));
                j += 1;
            } else {
                break;
            }
        }

        operations
    }

    /// Compute Longest Common Subsequence
    fn lcs<'a>(original: &'a [&'a str], modified: &'a [&'a str]) -> Vec<&'a str> {
        let n = original.len();
        let m = modified.len();
        let mut dp = vec![vec![0; m + 1]; n + 1];

        // Fill DP table
        for i in 1..=n {
            for j in 1..=m {
                if original[i - 1] == modified[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        // Reconstruct LCS
        let mut lcs = Vec::new();
        let mut i = n;
        let mut j = m;

        while i > 0 && j > 0 {
            if original[i - 1] == modified[j - 1] {
                lcs.push(original[i - 1]);
                i -= 1;
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }

        lcs.reverse();
        lcs
    }

    /// Group diff operations into hunks with context
    fn group_into_hunks(
        operations: &[DiffOperation],
        original: &[&str],
        modified: &[&str],
    ) -> Vec<DiffHunk> {
        const CONTEXT_SIZE: usize = 3;
        let mut hunks = Vec::new();
        let mut current_lines = Vec::new();
        let mut last_change = None;

        for (idx, op) in operations.iter().enumerate() {
            match op {
                DiffOperation::Equal(orig_idx, mod_idx) => {
                    // Add context around changes
                    if let Some(last_change_idx) = last_change {
                        if idx - last_change_idx <= CONTEXT_SIZE * 2 {
                            // Close enough to last change, include as context
                            current_lines.push(DiffLine {
                                content: original[*orig_idx].to_string(),
                                line_type: DiffLineType::Context,
                            });
                        } else {
                            // Too far from last change, finalize current hunk
                            if !current_lines.is_empty() {
                                hunks.push(Self::create_hunk(
                                    current_lines,
                                    operations,
                                    last_change_idx,
                                    idx,
                                ));
                                current_lines = Vec::new();
                            }
                            last_change = None;
                        }
                    }
                }
                DiffOperation::Delete(orig_idx) => {
                    current_lines.push(DiffLine {
                        content: original[*orig_idx].to_string(),
                        line_type: DiffLineType::Removed,
                    });
                    last_change = Some(idx);
                }
                DiffOperation::Insert(mod_idx) => {
                    current_lines.push(DiffLine {
                        content: modified[*mod_idx].to_string(),
                        line_type: DiffLineType::Added,
                    });
                    last_change = Some(idx);
                }
            }
        }

        // Finalize last hunk
        if !current_lines.is_empty() {
            hunks.push(DiffHunk {
                header: "@@".to_string(),
                lines: current_lines,
                original_start: 1,
                original_len: original.len(),
                modified_start: 1,
                modified_len: modified.len(),
            });
        }

        if hunks.is_empty() && !operations.is_empty() {
            // Create a single hunk with all operations
            let mut lines = Vec::new();
            for op in operations {
                match op {
                    DiffOperation::Equal(orig_idx, _) => {
                        lines.push(DiffLine {
                            content: original[*orig_idx].to_string(),
                            line_type: DiffLineType::Context,
                        });
                    }
                    DiffOperation::Delete(orig_idx) => {
                        lines.push(DiffLine {
                            content: original[*orig_idx].to_string(),
                            line_type: DiffLineType::Removed,
                        });
                    }
                    DiffOperation::Insert(mod_idx) => {
                        lines.push(DiffLine {
                            content: modified[*mod_idx].to_string(),
                            line_type: DiffLineType::Added,
                        });
                    }
                }
            }
            hunks.push(DiffHunk {
                header: format!("@@ -1,{} +1,{} @@", original.len(), modified.len()),
                lines,
                original_start: 1,
                original_len: original.len(),
                modified_start: 1,
                modified_len: modified.len(),
            });
        }

        hunks
    }

    fn create_hunk(
        lines: Vec<DiffLine>,
        _operations: &[DiffOperation],
        _start: usize,
        _end: usize,
    ) -> DiffHunk {
        DiffHunk {
            header: "@@".to_string(),
            lines,
            original_start: 1,
            original_len: 0,
            modified_start: 1,
            modified_len: 0,
        }
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
                    original_start: 1,
                    original_len: 0,
                    modified_start: 1,
                    modified_len: 0,
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
enum DiffOperation {
    Equal(usize, usize), // (original_idx, modified_idx)
    Delete(usize),       // original_idx
    Insert(usize),       // modified_idx
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
    pub original_start: usize,
    pub original_len: usize,
    pub modified_start: usize,
    pub modified_len: usize,
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
