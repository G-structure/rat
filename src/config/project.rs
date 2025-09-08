use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub current_project: Option<ProjectSettings>,
    pub project_history: Vec<ProjectSettings>,
    pub auto_detect: bool,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub name: String,
    pub root_path: PathBuf,
    pub preferred_agent: Option<String>,
    pub context_files: Vec<String>,
    pub environment_vars: HashMap<String, String>,
    pub custom_instructions: Option<String>,
    pub excluded_paths: Vec<String>,
    pub language_settings: HashMap<String, LanguageSettings>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSettings {
    pub formatter: Option<String>,
    pub linter: Option<String>,
    pub test_command: Option<String>,
    pub build_command: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            current_project: None,
            project_history: Vec::new(),
            auto_detect: true,
            ignore_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
            ],
        }
    }
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            formatter: None,
            linter: None,
            test_command: None,
            build_command: None,
        }
    }
}

impl ProjectConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(current) = &self.current_project {
            current.validate()?;
        }

        for project in &self.project_history {
            project.validate()?;
        }

        Ok(())
    }

    pub fn merge_with(&mut self, other: ProjectConfig) {
        if other.current_project.is_some() {
            self.current_project = other.current_project;
        }
        if !other.project_history.is_empty() {
            self.project_history = other.project_history;
        }
        if other.auto_detect != ProjectConfig::default().auto_detect {
            self.auto_detect = other.auto_detect;
        }
        if !other.ignore_patterns.is_empty() {
            self.ignore_patterns = other.ignore_patterns;
        }
    }

    pub fn set_current_project(&mut self, project: ProjectSettings) -> Result<()> {
        project.validate()?;

        // Add to history if not already there
        if !self
            .project_history
            .iter()
            .any(|p| p.root_path == project.root_path)
        {
            self.project_history.push(project.clone());
        } else {
            // Update existing entry
            if let Some(existing) = self
                .project_history
                .iter_mut()
                .find(|p| p.root_path == project.root_path)
            {
                *existing = project.clone();
            }
        }

        // Keep only the most recent 20 projects
        self.project_history
            .sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        self.project_history.truncate(20);

        self.current_project = Some(project);
        Ok(())
    }

    pub fn detect_project(&self, current_dir: &PathBuf) -> Option<ProjectSettings> {
        if !self.auto_detect {
            return None;
        }

        // Look for common project indicators
        let mut dir = current_dir.clone();
        loop {
            if self.is_project_root(&dir) {
                let name = dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                return Some(ProjectSettings {
                    name,
                    root_path: dir.clone(),
                    preferred_agent: None,
                    context_files: self.find_context_files(&dir),
                    environment_vars: HashMap::new(),
                    custom_instructions: None,
                    excluded_paths: self.ignore_patterns.clone(),
                    language_settings: HashMap::new(),
                    last_accessed: chrono::Utc::now(),
                });
            }

            if !dir.pop() {
                break;
            }
        }

        None
    }

    pub fn find_project_by_path(&self, path: &PathBuf) -> Option<&ProjectSettings> {
        self.project_history
            .iter()
            .find(|p| path.starts_with(&p.root_path))
    }

    pub fn get_recent_projects(&self, limit: usize) -> Vec<&ProjectSettings> {
        self.project_history.iter().take(limit).collect()
    }

    fn is_project_root(&self, dir: &PathBuf) -> bool {
        const PROJECT_INDICATORS: &[&str] = &[
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "requirements.txt",
            "go.mod",
            ".git",
            "Makefile",
            "CMakeLists.txt",
        ];

        PROJECT_INDICATORS
            .iter()
            .any(|&indicator| dir.join(indicator).exists())
    }

    fn find_context_files(&self, dir: &PathBuf) -> Vec<String> {
        const CONTEXT_FILES: &[&str] = &[
            "README.md",
            "README.rst",
            "CHANGELOG.md",
            "CONTRIBUTING.md",
            "LICENSE",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
        ];

        CONTEXT_FILES
            .iter()
            .filter_map(|&file| {
                let path = dir.join(file);
                if path.exists() {
                    Some(file.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl ProjectSettings {
    pub fn new(name: String, root_path: PathBuf) -> Self {
        Self {
            name,
            root_path,
            preferred_agent: None,
            context_files: Vec::new(),
            environment_vars: HashMap::new(),
            custom_instructions: None,
            excluded_paths: Vec::new(),
            language_settings: HashMap::new(),
            last_accessed: chrono::Utc::now(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }

        if !self.root_path.exists() {
            return Err(anyhow::anyhow!(
                "Project root path does not exist: {:?}",
                self.root_path
            ));
        }

        if !self.root_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Project root path is not a directory: {:?}",
                self.root_path
            ));
        }

        Ok(())
    }

    pub fn add_context_file(&mut self, file: String) {
        if !self.context_files.contains(&file) {
            self.context_files.push(file);
        }
        self.touch();
    }

    pub fn remove_context_file(&mut self, file: &str) {
        self.context_files.retain(|f| f != file);
        self.touch();
    }

    pub fn set_environment_var(&mut self, key: String, value: String) {
        self.environment_vars.insert(key, value);
        self.touch();
    }

    pub fn remove_environment_var(&mut self, key: &str) {
        self.environment_vars.remove(key);
        self.touch();
    }

    pub fn set_language_setting(&mut self, language: String, settings: LanguageSettings) {
        self.language_settings.insert(language, settings);
        self.touch();
    }

    pub fn get_language_setting(&self, language: &str) -> Option<&LanguageSettings> {
        self.language_settings.get(language)
    }

    pub fn should_exclude_path(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();

        self.excluded_paths.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple glob matching
                self.glob_match(pattern, &path_str)
            } else {
                path_str.contains(pattern)
            }
        })
    }

    pub fn get_context_file_paths(&self) -> Vec<PathBuf> {
        self.context_files
            .iter()
            .map(|file| self.root_path.join(file))
            .filter(|path| path.exists())
            .collect()
    }

    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
    }

    pub fn is_stale(&self, max_age: chrono::Duration) -> bool {
        chrono::Utc::now() - self.last_accessed > max_age
    }

    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        // Simple glob matching - would use a proper glob library in production
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                text.starts_with(parts[0]) && text.ends_with(parts[1])
            } else {
                false
            }
        } else {
            pattern == text
        }
    }
}

impl LanguageSettings {
    pub fn for_rust() -> Self {
        Self {
            formatter: Some("rustfmt".to_string()),
            linter: Some("clippy".to_string()),
            test_command: Some("cargo test".to_string()),
            build_command: Some("cargo build".to_string()),
        }
    }

    pub fn for_python() -> Self {
        Self {
            formatter: Some("black".to_string()),
            linter: Some("ruff".to_string()),
            test_command: Some("pytest".to_string()),
            build_command: None,
        }
    }

    pub fn for_javascript() -> Self {
        Self {
            formatter: Some("prettier".to_string()),
            linter: Some("eslint".to_string()),
            test_command: Some("npm test".to_string()),
            build_command: Some("npm run build".to_string()),
        }
    }

    pub fn for_typescript() -> Self {
        Self {
            formatter: Some("prettier".to_string()),
            linter: Some("eslint".to_string()),
            test_command: Some("npm test".to_string()),
            build_command: Some("npm run build".to_string()),
        }
    }
}
