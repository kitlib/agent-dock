use std::path::{Path, PathBuf};
use crate::repositories::errors::RepositoryError;

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Managed agent domain model
#[derive(Debug, Clone)]
pub struct ManagedAgent {
    pub id: AgentId,
    pub fingerprint: String,
    pub agent_type: Option<String>,
    pub alias: Option<String>,
    pub enabled: bool,
    pub hidden: bool,
    pub imported_at: String,
    pub source: String,
    pub root_path: Option<PathBuf>,
}

impl ManagedAgent {
    /// Check if this managed agent matches a discovered agent
    pub fn matches(
        &self,
        discovered_fingerprint: &str,
        discovered_agent_type: &str,
        discovered_root: &Path,
    ) -> bool {
        // Match by fingerprint
        if self.fingerprint == discovered_fingerprint {
            return true;
        }

        // Match by agent type and root path
        if let (Some(agent_type), Some(root_path)) = (&self.agent_type, &self.root_path) {
            agent_type == discovered_agent_type && root_path == discovered_root
        } else {
            false
        }
    }

    /// Enable this agent
    pub fn enable(&mut self) {
        self.enabled = true;
        self.hidden = false;
    }

    /// Disable this agent
    pub fn disable(&mut self) {
        self.enabled = false;
        self.hidden = true;
    }

    /// Set alias for this agent
    pub fn set_alias(&mut self, alias: String) -> Result<(), crate::services::ServiceError> {
        let trimmed = alias.trim();
        if trimmed.is_empty() {
            return Err(crate::services::ServiceError::InvalidAlias("Alias cannot be empty".into()));
        }
        self.alias = Some(trimmed.to_string());
        Ok(())
    }

    /// Clear alias
    pub fn clear_alias(&mut self) {
        self.alias = None;
    }
}

/// Agent repository trait for persistence operations
pub trait AgentRepository: Send + Sync {
    /// Find all managed agents
    fn find_all(&self) -> Result<Vec<ManagedAgent>, RepositoryError>;

    /// Find agent by ID
    fn find_by_id(&self, id: &AgentId) -> Result<Option<ManagedAgent>, RepositoryError>;

    /// Find agent by fingerprint
    fn find_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<Option<ManagedAgent>, RepositoryError>;

    /// Save a single agent
    fn save(&self, agent: &ManagedAgent) -> Result<(), RepositoryError>;

    /// Save multiple agents (batch operation)
    fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError>;

    /// Delete agent by ID
    fn delete(&self, id: &AgentId) -> Result<(), RepositoryError>;
}
