use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: SessionId,
    pub request_type: PermissionType,
    pub description: String,
    pub status: PermissionStatus,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionType {
    FileRead {
        path: PathBuf,
    },
    FileWrite {
        path: PathBuf,
        content_preview: Option<String>,
    },
    FileDelete {
        path: PathBuf,
    },
    DirectoryCreate {
        path: PathBuf,
    },
    DirectoryList {
        path: PathBuf,
    },
    CommandExecute {
        command: String,
        args: Vec<String>,
    },
    NetworkRequest {
        url: String,
        method: String,
    },
    EnvironmentAccess {
        variable: String,
    },
    ProcessSpawn {
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionStatus {
    Pending,
    Granted,
    Denied,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub request_id: String,
    pub granted: bool,
    pub reason: Option<String>,
    pub remember_choice: bool,
}

#[derive(Debug, Default)]
pub struct PermissionManager {
    pending_requests: std::collections::HashMap<String, PermissionRequest>,
    granted_permissions: std::collections::HashMap<SessionId, Vec<PermissionType>>,
    permission_rules: Vec<PermissionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub pattern: PermissionPattern,
    pub action: PermissionAction,
    pub expires_after: Option<chrono::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionPattern {
    FilePathGlob(String),
    CommandPrefix(String),
    NetworkDomain(String),
    Always,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionAction {
    Allow,
    Deny,
    Prompt,
}

impl PermissionRequest {
    pub fn new(session_id: SessionId, request_type: PermissionType, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            request_type,
            description,
            status: PermissionStatus::Pending,
            requested_at: chrono::Utc::now(),
            responded_at: None,
        }
    }

    pub fn respond(&mut self, response: PermissionResponse) {
        self.status = if response.granted {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        };
        self.responded_at = Some(chrono::Utc::now());
    }

    pub fn expire(&mut self) {
        self.status = PermissionStatus::Expired;
        self.responded_at = Some(chrono::Utc::now());
    }

    pub fn is_pending(&self) -> bool {
        self.status == PermissionStatus::Pending
    }

    pub fn risk_level(&self) -> RiskLevel {
        match &self.request_type {
            PermissionType::FileRead { .. } => RiskLevel::Low,
            PermissionType::DirectoryList { .. } => RiskLevel::Low,
            PermissionType::FileWrite { .. } => RiskLevel::Medium,
            PermissionType::FileDelete { .. } => RiskLevel::High,
            PermissionType::DirectoryCreate { .. } => RiskLevel::Medium,
            PermissionType::CommandExecute { command, .. } => {
                if is_safe_command(command) {
                    RiskLevel::Medium
                } else {
                    RiskLevel::High
                }
            }
            PermissionType::NetworkRequest { .. } => RiskLevel::Medium,
            PermissionType::EnvironmentAccess { .. } => RiskLevel::Low,
            PermissionType::ProcessSpawn { .. } => RiskLevel::High,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_permission(
        &mut self,
        session_id: SessionId,
        request_type: PermissionType,
        description: String,
    ) -> Result<String> {
        let request = PermissionRequest::new(session_id, request_type, description);
        let request_id = request.id.clone();

        self.pending_requests.insert(request_id.clone(), request);

        Ok(request_id)
    }

    pub fn respond_to_request(
        &mut self,
        request_id: &str,
        response: PermissionResponse,
    ) -> Result<()> {
        if let Some(mut request) = self.pending_requests.remove(request_id) {
            request.respond(response.clone());

            if response.granted && response.remember_choice {
                self.granted_permissions
                    .entry(request.session_id.clone())
                    .or_insert_with(Vec::new)
                    .push(request.request_type);
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Permission request not found: {}",
                request_id
            ))
        }
    }

    pub fn get_pending_requests(&self) -> Vec<&PermissionRequest> {
        self.pending_requests.values().collect()
    }

    pub fn get_request(&self, request_id: &str) -> Option<&PermissionRequest> {
        self.pending_requests.get(request_id)
    }

    pub fn check_auto_permission(
        &self,
        session_id: &SessionId,
        request_type: &PermissionType,
    ) -> Option<bool> {
        // Check if this permission type was previously granted for this session
        if let Some(granted) = self.granted_permissions.get(session_id) {
            if granted
                .iter()
                .any(|p| permission_types_match(p, request_type))
            {
                return Some(true);
            }
        }

        // Check permission rules
        for rule in &self.permission_rules {
            if rule.pattern.matches(request_type) {
                match rule.action {
                    PermissionAction::Allow => return Some(true),
                    PermissionAction::Deny => return Some(false),
                    PermissionAction::Prompt => return None,
                }
            }
        }

        None
    }

    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.permission_rules.push(rule);
    }

    pub fn cleanup_expired(&mut self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        self.pending_requests.retain(|_, request| {
            if request.requested_at < cutoff {
                false
            } else {
                true
            }
        });
    }
}

impl PermissionPattern {
    pub fn matches(&self, request_type: &PermissionType) -> bool {
        match (self, request_type) {
            (PermissionPattern::Always, _) => true,
            (PermissionPattern::Never, _) => false,
            (PermissionPattern::FilePathGlob(pattern), PermissionType::FileRead { path })
            | (PermissionPattern::FilePathGlob(pattern), PermissionType::FileWrite { path, .. })
            | (PermissionPattern::FilePathGlob(pattern), PermissionType::FileDelete { path }) => {
                glob_match(pattern, path.to_str().unwrap_or(""))
            }
            (
                PermissionPattern::CommandPrefix(prefix),
                PermissionType::CommandExecute { command, .. },
            ) => command.starts_with(prefix),
            (
                PermissionPattern::NetworkDomain(domain),
                PermissionType::NetworkRequest { url, .. },
            ) => url.contains(domain),
            _ => false,
        }
    }
}

fn permission_types_match(a: &PermissionType, b: &PermissionType) -> bool {
    use PermissionType::*;
    match (a, b) {
        (FileRead { path: p1 }, FileRead { path: p2 }) => p1 == p2,
        (FileWrite { path: p1, .. }, FileWrite { path: p2, .. }) => p1 == p2,
        (FileDelete { path: p1 }, FileDelete { path: p2 }) => p1 == p2,
        (DirectoryCreate { path: p1 }, DirectoryCreate { path: p2 }) => p1 == p2,
        (DirectoryList { path: p1 }, DirectoryList { path: p2 }) => p1 == p2,
        (
            CommandExecute {
                command: c1,
                args: a1,
            },
            CommandExecute {
                command: c2,
                args: a2,
            },
        ) => c1 == c2 && a1 == a2,
        (
            NetworkRequest {
                url: u1,
                method: m1,
            },
            NetworkRequest {
                url: u2,
                method: m2,
            },
        ) => u1 == u2 && m1 == m2,
        (EnvironmentAccess { variable: v1 }, EnvironmentAccess { variable: v2 }) => v1 == v2,
        (ProcessSpawn { command: c1 }, ProcessSpawn { command: c2 }) => c1 == c2,
        _ => false,
    }
}

fn is_safe_command(command: &str) -> bool {
    const SAFE_COMMANDS: &[&str] = &[
        "ls", "cat", "head", "tail", "grep", "find", "pwd", "whoami", "date", "echo", "which",
        "git", "cargo", "npm", "python", "node",
    ];

    SAFE_COMMANDS.iter().any(|&safe| command.starts_with(safe))
}

fn glob_match(pattern: &str, text: &str) -> bool {
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
