use crate::repositories::errors::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid fingerprint: {0}")]
    InvalidFingerprint(String),

    #[error("Invalid alias: {0}")]
    InvalidAlias(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Conflicting entry files at {0}")]
    ConflictingEntryFiles(String),

    #[error("Cannot copy skill to the same agent")]
    CannotCopyToSameAgent,

    #[error("Unsupported skill source: {0}")]
    UnsupportedSkillSource(String),

    #[error("Invalid skill path: {0}")]
    InvalidSkillPath(String),
}

impl From<ServiceError> for String {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Repository(e) => format!("Storage error: {}", e),
            ServiceError::Scanner(e) => format!("Scan error: {}", e),
            ServiceError::BusinessRuleViolation(e) => format!("Operation failed: {}", e),
            ServiceError::Internal(e) => format!("Internal error: {}", e),
            ServiceError::InvalidFingerprint(e) => format!("Invalid fingerprint: {}", e),
            ServiceError::InvalidAlias(e) => format!("Invalid alias: {}", e),
            ServiceError::AgentNotFound(e) => format!("Agent not found: {}", e),
            ServiceError::SkillNotFound(e) => format!("Skill not found: {}", e),
            ServiceError::ConflictingEntryFiles(e) => format!("Conflicting entry files: {}", e),
            ServiceError::CannotCopyToSameAgent => "Cannot copy skill to the same agent".into(),
            ServiceError::UnsupportedSkillSource(e) => format!("Unsupported skill source: {}", e),
            ServiceError::InvalidSkillPath(e) => format!("Invalid skill path: {}", e),
        }
    }
}
