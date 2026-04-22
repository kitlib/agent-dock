use std::path::PathBuf;
use std::fs;

use crate::dto::agents::ManagedAgentDto;
use crate::repositories::agent_repository::{AgentId, AgentRepository, ManagedAgent};
use crate::repositories::RepositoryError;

const STORE_DIR_NAME: &str = ".agentdock";
const STORE_FILE_NAME: &str = "managed-agents.json";

/// JSON-based implementation of AgentRepository
pub struct JsonAgentRepository {
    store_path: PathBuf,
}

impl JsonAgentRepository {
    pub fn new() -> Self {
        Self {
            store_path: Self::default_store_path(),
        }
    }

    fn default_store_path() -> PathBuf {
        crate::infrastructure::utils::path::user_home_dir()
            .join(STORE_DIR_NAME)
            .join(STORE_FILE_NAME)
    }

    fn ensure_store_seeded(&self) -> Result<Vec<ManagedAgentDto>, RepositoryError> {
        if self.store_path.exists() {
            let contents = fs::read_to_string(&self.store_path)
                .map_err(RepositoryError::IoError)?;
            let agents = serde_json::from_str(&contents)
                .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;
            return Ok(agents);
        }

        self.save_dtos(&[])?;
        Ok(Vec::new())
    }

    fn save_dtos(&self, agents: &[ManagedAgentDto]) -> Result<(), RepositoryError> {
        let dir_path = self.store_path.parent()
            .ok_or_else(|| RepositoryError::StorageError("Invalid store path".into()))?;

        fs::create_dir_all(dir_path)
            .map_err(RepositoryError::IoError)?;

        let contents = serde_json::to_string_pretty(agents)
            .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;

        fs::write(&self.store_path, contents)
            .map_err(RepositoryError::IoError)
    }

}

impl Default for JsonAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

// Conversion between domain model and DTO
impl From<ManagedAgentDto> for ManagedAgent {
    fn from(dto: ManagedAgentDto) -> Self {
        Self {
            id: AgentId::new(dto.managed_agent_id),
            fingerprint: dto.fingerprint,
            agent_type: dto.agent_type,
            alias: dto.alias,
            enabled: dto.enabled,
            hidden: dto.hidden,
            imported_at: dto.imported_at,
            source: dto.source,
            root_path: dto.root_path.map(PathBuf::from),
        }
    }
}

impl From<&ManagedAgent> for ManagedAgentDto {
    fn from(agent: &ManagedAgent) -> Self {
        Self {
            managed_agent_id: agent.id.0.clone(),
            fingerprint: agent.fingerprint.clone(),
            agent_type: agent.agent_type.clone(),
            alias: agent.alias.clone(),
            enabled: agent.enabled,
            hidden: agent.hidden,
            imported_at: agent.imported_at.clone(),
            source: agent.source.clone(),
            root_path: agent.root_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        }
    }
}

impl AgentRepository for JsonAgentRepository {
    fn find_all(&self) -> Result<Vec<ManagedAgent>, RepositoryError> {
        let dtos = self.ensure_store_seeded()?;
        Ok(dtos.into_iter().map(Into::into).collect())
    }

    fn find_by_id(&self, id: &AgentId) -> Result<Option<ManagedAgent>, RepositoryError> {
        let agents = self.find_all()?;
        Ok(agents.into_iter().find(|a| &a.id == id))
    }

    fn find_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<Option<ManagedAgent>, RepositoryError> {
        let agents = self.find_all()?;
        Ok(agents.into_iter().find(|a| a.fingerprint == fingerprint))
    }

    fn save(&self, agent: &ManagedAgent) -> Result<(), RepositoryError> {
        let mut agents = self.find_all()?;

        if let Some(existing) = agents.iter_mut().find(|a| a.id == agent.id) {
            *existing = agent.clone();
        } else {
            agents.push(agent.clone());
        }

        self.save_all(&agents)
    }

    fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError> {
        let dtos: Vec<ManagedAgentDto> = agents.iter().map(Into::into).collect();
        self.save_dtos(&dtos)
    }

    fn delete(&self, id: &AgentId) -> Result<(), RepositoryError> {
        let agents = self.find_all()?;
        let filtered: Vec<_> = agents.into_iter().filter(|a| &a.id != id).collect();
        self.save_all(&filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_store_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-repo-test-{unique}"))
    }

    fn create_test_repo() -> JsonAgentRepository {
        let store_path = temp_store_path().join("managed-agents.json");
        JsonAgentRepository { store_path }
    }

    fn cleanup_test_repo(repo: &JsonAgentRepository) {
        if let Some(parent) = repo.store_path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_find_all_returns_default_agents_on_first_load() {
        let repo = create_test_repo();
        let agents = repo.find_all().expect("find all");

        assert_eq!(agents.len(), 3);
        assert!(agents.iter().any(|a| a.fingerprint == "cursor-workspace-default"));

        cleanup_test_repo(&repo);
    }

    #[test]
    fn test_save_and_find_by_id() {
        let repo = create_test_repo();

        let agent = ManagedAgent {
            id: AgentId::new("test-agent".into()),
            fingerprint: "test-fingerprint".into(),
            agent_type: Some("claude".into()),
            alias: Some("Test Agent".into()),
            enabled: true,
            hidden: false,
            imported_at: "2026-04-21T00:00:00Z".into(),
            source: "test".into(),
            root_path: Some(PathBuf::from("/test")),
        };

        repo.save(&agent).expect("save agent");

        let found = repo.find_by_id(&agent.id).expect("find by id");
        assert!(found.is_some());
        assert_eq!(found.unwrap().fingerprint, "test-fingerprint");

        cleanup_test_repo(&repo);
    }

    #[test]
    fn test_find_by_fingerprint() {
        let repo = create_test_repo();

        let found = repo.find_by_fingerprint("claude-default").expect("find by fingerprint");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id.as_str(), "managed-claude");

        cleanup_test_repo(&repo);
    }

    #[test]
    fn test_delete_removes_agent() {
        let repo = create_test_repo();

        let agents = repo.find_all().expect("find all");
        let first_id = agents[0].id.clone();

        repo.delete(&first_id).expect("delete agent");

        let remaining = repo.find_all().expect("find all after delete");
        assert_eq!(remaining.len(), 2);
        assert!(!remaining.iter().any(|a| a.id == first_id));

        cleanup_test_repo(&repo);
    }

    #[test]
    fn test_save_all_overwrites_existing() {
        let repo = create_test_repo();

        let new_agents = vec![
            ManagedAgent {
                id: AgentId::new("new-1".into()),
                fingerprint: "fp-1".into(),
                agent_type: Some("claude".into()),
                alias: None,
                enabled: true,
                hidden: false,
                imported_at: "2026-04-21T00:00:00Z".into(),
                source: "test".into(),
                root_path: None,
            },
        ];

        repo.save_all(&new_agents).expect("save all");

        let loaded = repo.find_all().expect("find all");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].fingerprint, "fp-1");

        cleanup_test_repo(&repo);
    }
}
