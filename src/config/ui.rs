use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub keybindings: KeybindingConfig,
    pub effects: EffectsConfig,
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub custom_colors: HashMap<String, String>,
    pub syntax_highlighting: bool,
    pub agent_colors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub default_layout: String,
    pub show_status_bar: bool,
    pub show_agent_selector: bool,
    pub show_session_tabs: bool,
    pub sidebar_width: u16,
    pub terminal_height: u16,
    pub chat_history_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub quit: String,
    pub new_session: String,
    pub switch_agent: String,
    pub accept_edit: String,
    pub reject_edit: String,
    pub toggle_terminal: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub custom_bindings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    pub enabled: bool,
    pub animation_speed: f32,
    pub typewriter_delay_ms: u64,
    pub diff_animation: bool,
    pub status_animation: bool,
    pub smooth_scrolling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub tab_size: usize,
    pub auto_save: bool,
    pub diff_context_lines: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            keybindings: KeybindingConfig::default(),
            effects: EffectsConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        let mut agent_colors = HashMap::new();
        agent_colors.insert("claude-code".to_string(), "#FF6B35".to_string());
        agent_colors.insert("gemini".to_string(), "#4285F4".to_string());

        Self {
            name: "default".to_string(),
            custom_colors: HashMap::new(),
            syntax_highlighting: true,
            agent_colors,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            default_layout: "tabbed".to_string(),
            show_status_bar: true,
            show_agent_selector: true,
            show_session_tabs: true,
            sidebar_width: 25,
            terminal_height: 20,
            chat_history_limit: 100,
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        let mut custom_bindings = HashMap::new();

        Self {
            quit: "q".to_string(),
            new_session: "n".to_string(),
            switch_agent: "a".to_string(),
            accept_edit: "y".to_string(),
            reject_edit: "n".to_string(),
            toggle_terminal: "t".to_string(),
            next_tab: "Tab".to_string(),
            prev_tab: "BackTab".to_string(),
            custom_bindings,
        }
    }
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            animation_speed: 1.0,
            typewriter_delay_ms: 50,
            diff_animation: true,
            status_animation: true,
            smooth_scrolling: true,
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            word_wrap: true,
            tab_size: 4,
            auto_save: false,
            diff_context_lines: 3,
        }
    }
}

impl UiConfig {
    pub fn validate(&self) -> Result<()> {
        if self.layout.sidebar_width == 0 || self.layout.sidebar_width > 80 {
            return Err(anyhow::anyhow!("sidebar_width must be between 1 and 80"));
        }

        if self.layout.terminal_height == 0 || self.layout.terminal_height > 50 {
            return Err(anyhow::anyhow!("terminal_height must be between 1 and 50"));
        }

        if self.layout.chat_history_limit == 0 {
            return Err(anyhow::anyhow!("chat_history_limit must be greater than 0"));
        }

        if self.effects.animation_speed <= 0.0 || self.effects.animation_speed > 5.0 {
            return Err(anyhow::anyhow!(
                "animation_speed must be between 0.0 and 5.0"
            ));
        }

        if self.editor.tab_size == 0 || self.editor.tab_size > 16 {
            return Err(anyhow::anyhow!("tab_size must be between 1 and 16"));
        }

        let valid_layouts = ["tabbed", "split", "dashboard"];
        if !valid_layouts.contains(&self.layout.default_layout.as_str()) {
            return Err(anyhow::anyhow!(
                "default_layout must be one of: {:?}",
                valid_layouts
            ));
        }

        Ok(())
    }

    pub fn merge_with(&mut self, other: UiConfig) {
        self.theme.merge_with(other.theme);
        self.layout.merge_with(other.layout);
        self.keybindings.merge_with(other.keybindings);
        self.effects.merge_with(other.effects);
        self.editor.merge_with(other.editor);
    }

    pub fn get_agent_color(&self, agent_name: &str) -> Option<&String> {
        self.theme.agent_colors.get(agent_name)
    }

    pub fn get_custom_color(&self, color_name: &str) -> Option<&String> {
        self.theme.custom_colors.get(color_name)
    }

    pub fn get_keybinding(&self, action: &str) -> Option<&String> {
        match action {
            "quit" => Some(&self.keybindings.quit),
            "new_session" => Some(&self.keybindings.new_session),
            "switch_agent" => Some(&self.keybindings.switch_agent),
            "accept_edit" => Some(&self.keybindings.accept_edit),
            "reject_edit" => Some(&self.keybindings.reject_edit),
            "toggle_terminal" => Some(&self.keybindings.toggle_terminal),
            "next_tab" => Some(&self.keybindings.next_tab),
            "prev_tab" => Some(&self.keybindings.prev_tab),
            _ => self.keybindings.custom_bindings.get(action),
        }
    }
}

impl ThemeConfig {
    pub fn merge_with(&mut self, other: ThemeConfig) {
        if other.name != ThemeConfig::default().name {
            self.name = other.name;
        }
        if other.syntax_highlighting != ThemeConfig::default().syntax_highlighting {
            self.syntax_highlighting = other.syntax_highlighting;
        }
        self.custom_colors.extend(other.custom_colors);
        self.agent_colors.extend(other.agent_colors);
    }
}

impl LayoutConfig {
    pub fn merge_with(&mut self, other: LayoutConfig) {
        if other.default_layout != LayoutConfig::default().default_layout {
            self.default_layout = other.default_layout;
        }
        if other.show_status_bar != LayoutConfig::default().show_status_bar {
            self.show_status_bar = other.show_status_bar;
        }
        if other.show_agent_selector != LayoutConfig::default().show_agent_selector {
            self.show_agent_selector = other.show_agent_selector;
        }
        if other.show_session_tabs != LayoutConfig::default().show_session_tabs {
            self.show_session_tabs = other.show_session_tabs;
        }
        if other.sidebar_width != LayoutConfig::default().sidebar_width {
            self.sidebar_width = other.sidebar_width;
        }
        if other.terminal_height != LayoutConfig::default().terminal_height {
            self.terminal_height = other.terminal_height;
        }
        if other.chat_history_limit != LayoutConfig::default().chat_history_limit {
            self.chat_history_limit = other.chat_history_limit;
        }
    }
}

impl KeybindingConfig {
    pub fn merge_with(&mut self, other: KeybindingConfig) {
        if other.quit != KeybindingConfig::default().quit {
            self.quit = other.quit;
        }
        if other.new_session != KeybindingConfig::default().new_session {
            self.new_session = other.new_session;
        }
        if other.switch_agent != KeybindingConfig::default().switch_agent {
            self.switch_agent = other.switch_agent;
        }
        if other.accept_edit != KeybindingConfig::default().accept_edit {
            self.accept_edit = other.accept_edit;
        }
        if other.reject_edit != KeybindingConfig::default().reject_edit {
            self.reject_edit = other.reject_edit;
        }
        if other.toggle_terminal != KeybindingConfig::default().toggle_terminal {
            self.toggle_terminal = other.toggle_terminal;
        }
        if other.next_tab != KeybindingConfig::default().next_tab {
            self.next_tab = other.next_tab;
        }
        if other.prev_tab != KeybindingConfig::default().prev_tab {
            self.prev_tab = other.prev_tab;
        }
        self.custom_bindings.extend(other.custom_bindings);
    }
}

impl EffectsConfig {
    pub fn merge_with(&mut self, other: EffectsConfig) {
        if other.enabled != EffectsConfig::default().enabled {
            self.enabled = other.enabled;
        }
        if other.animation_speed != EffectsConfig::default().animation_speed {
            self.animation_speed = other.animation_speed;
        }
        if other.typewriter_delay_ms != EffectsConfig::default().typewriter_delay_ms {
            self.typewriter_delay_ms = other.typewriter_delay_ms;
        }
        if other.diff_animation != EffectsConfig::default().diff_animation {
            self.diff_animation = other.diff_animation;
        }
        if other.status_animation != EffectsConfig::default().status_animation {
            self.status_animation = other.status_animation;
        }
        if other.smooth_scrolling != EffectsConfig::default().smooth_scrolling {
            self.smooth_scrolling = other.smooth_scrolling;
        }
    }
}

impl EditorConfig {
    pub fn merge_with(&mut self, other: EditorConfig) {
        if other.show_line_numbers != EditorConfig::default().show_line_numbers {
            self.show_line_numbers = other.show_line_numbers;
        }
        if other.word_wrap != EditorConfig::default().word_wrap {
            self.word_wrap = other.word_wrap;
        }
        if other.tab_size != EditorConfig::default().tab_size {
            self.tab_size = other.tab_size;
        }
        if other.auto_save != EditorConfig::default().auto_save {
            self.auto_save = other.auto_save;
        }
        if other.diff_context_lines != EditorConfig::default().diff_context_lines {
            self.diff_context_lines = other.diff_context_lines;
        }
    }
}
