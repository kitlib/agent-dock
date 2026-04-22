# Backend Architecture Refactoring Plan

## Current Architecture Issues

### 1. Responsibility Confusion

- **Commands layer** contains excessive business logic (e.g., `commands/skills.rs` has 1048 lines)
- **Services layer** is too simple, only performing data transformation
- **commands/mcp.rs** directly handles complex logic like process management and file parsing

### 2. Code Duplication

- Multiple implementations of `user_home_dir()`
- Path normalization logic scattered across multiple files
- MCP configuration parsing logic coupled with business logic

### 3. Single Responsibility Principle Violations

- `agent_discovery_service.rs` mixes data transformation, matching logic, and business orchestration
- `commands/mcp.rs` simultaneously handles command reception, process management, config parsing, and error handling

### 4. Missing Abstraction Layers

- No independent Repository layer for persistence management
- Lack of Domain Model layer
- Scanners directly return DTOs, missing domain objects

## Proposed Layered Architecture

```
┌─────────────────────────────────────────┐
│     Commands (Tauri Interface Layer)    │  ← Thin layer: parameter validation & result conversion
├─────────────────────────────────────────┤
│     Services (Business Orchestration)   │  ← Orchestrate multiple Domain/Repository operations
├─────────────────────────────────────────┤
│     Domain (Domain Logic Layer)         │  ← Core business rules and entities
├─────────────────────────────────────────┤
│     Repository (Data Access Abstraction)│  ← Unified persistence interface
├─────────────────────────────────────────┤
│     Infrastructure (Infrastructure)     │  ← Scanner/Store/Utils implementations
└─────────────────────────────────────────┘
```

### Layer Responsibilities

#### Commands Layer (Tauri Interface)

**Responsibility:**
- Receive frontend requests
- Parameter validation
- Call Services layer
- Convert domain objects to DTOs
- Handle Tauri-specific concerns

**Principles:**
- Keep handlers thin (< 20 lines)
- No business logic
- Only DTO types in signatures

**Example:**
```rust
// src-tauri/src/commands/agents.rs
#[tauri::command]
pub fn list_managed_agents(
    state: State<AppState>
) -> Result<Vec<ManagedAgentDto>, String> {
    let agents = state.agent_service.list_managed()?;
    Ok(agents.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub fn import_agents(
    state: State<AppState>,
    candidate_ids: Vec<String>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<ImportAgentsResultDto, String> {
    let targets = scan_targets.into_iter().map(Into::into).collect();
    let result = state.agent_service.import_agents(candidate_ids, targets)?;
    Ok(result.into())
}
```

#### Services Layer (Business Orchestration)

**Responsibility:**
- Orchestrate multiple domain operations
- Coordinate Repository and Scanner calls
- Transaction management
- Business workflow control

**Principles:**
- No direct file system access
- No direct database access
- Depend on Repository abstractions
- Return domain objects

**Example:**
```rust
// src-tauri/src/services/agent_service.rs
pub struct AgentService {
    agent_repo: Arc<dyn AgentRepository>,
    scanner: Arc<dyn AgentScanner>,
}

impl AgentService {
    pub fn list_resolved(&self, targets: Vec<ScanTarget>) -> Result<Vec<ResolvedAgent>> {
        let discovered = self.scanner.scan(targets)?;
        let managed = self.agent_repo.find_all()?;
        Ok(self.merge_agents(discovered, managed))
    }

    pub fn import_agents(
        &self,
        candidate_ids: Vec<String>,
        targets: Vec<ScanTarget>,
    ) -> Result<ImportResult> {
        let discovered = self.scanner.scan(targets)?;
        let mut managed = self.agent_repo.find_all()?;
        
        let imported = self.process_imports(&candidate_ids, &discovered, &mut managed)?;
        self.agent_repo.save_all(&managed)?;
        
        Ok(ImportResult {
            imported_agents: imported,
            resolved_agents: self.merge_agents(discovered, managed),
        })
    }

    fn merge_agents(
        &self,
        discovered: Vec<DiscoveredAgent>,
        managed: Vec<ManagedAgent>,
    ) -> Vec<ResolvedAgent> {
        // Complex merge logic
    }

    fn process_imports(
        &self,
        candidate_ids: &[String],
        discovered: &[DiscoveredAgent],
        managed: &mut Vec<ManagedAgent>,
    ) -> Result<Vec<ResolvedAgent>> {
        // Import processing logic
    }
}
```

#### Domain Layer (Core Business Logic)

**Responsibility:**
- Define domain entities
- Implement business rules
- Domain validation
- State transitions

**Principles:**
- No external dependencies (no I/O, no framework)
- Pure business logic
- Self-contained validation
- Rich domain models

**Example:**
```rust
// src-tauri/src/domain/agent.rs
#[derive(Debug, Clone)]
pub struct AgentId(String);

#[derive(Debug, Clone)]
pub struct ManagedAgent {
    id: AgentId,
    fingerprint: String,
    agent_type: AgentType,
    alias: Option<String>,
    enabled: bool,
    hidden: bool,
    imported_at: DateTime,
    source: ImportSource,
    root_path: Option<PathBuf>,
}

impl ManagedAgent {
    pub fn new(
        fingerprint: String,
        agent_type: AgentType,
        root_path: PathBuf,
    ) -> Result<Self, DomainError> {
        if fingerprint.is_empty() {
            return Err(DomainError::InvalidFingerprint);
        }
        
        Ok(Self {
            id: AgentId::generate(),
            fingerprint,
            agent_type,
            alias: None,
            enabled: true,
            hidden: false,
            imported_at: DateTime::now(),
            source: ImportSource::AutoImported,
            root_path: Some(root_path),
        })
    }

    pub fn matches(&self, discovered: &DiscoveredAgent) -> bool {
        self.fingerprint == discovered.fingerprint
            || (self.agent_type == discovered.agent_type 
                && self.root_path.as_ref() == Some(&discovered.root_path))
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.hidden = false;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.hidden = true;
    }

    pub fn set_alias(&mut self, alias: String) -> Result<(), DomainError> {
        if alias.trim().is_empty() {
            return Err(DomainError::InvalidAlias);
        }
        self.alias = Some(alias);
        Ok(())
    }
}

// src-tauri/src/domain/skill.rs
#[derive(Debug, Clone)]
pub struct Skill {
    id: SkillId,
    name: String,
    path: PathBuf,
    entry_file: PathBuf,
    enabled: bool,
    source_kind: SkillSourceKind,
    owner_agent_id: AgentId,
}

impl Skill {
    pub fn toggle_enabled(&mut self) -> Result<(), DomainError> {
        let (active_path, disabled_path) = self.resolve_entry_paths()?;
        
        if self.enabled {
            if disabled_path.exists() {
                return Err(DomainError::ConflictingEntryFiles);
            }
            // Will be renamed by infrastructure layer
            self.enabled = false;
        } else {
            if active_path.exists() {
                return Err(DomainError::ConflictingEntryFiles);
            }
            self.enabled = true;
        }
        
        Ok(())
    }

    pub fn validate_copy_destination(
        &self,
        target_agent: &Agent,
    ) -> Result<(), DomainError> {
        if self.owner_agent_id == target_agent.id {
            return Err(DomainError::CannotCopyToSameAgent);
        }
        
        if !target_agent.supports_skill_source(&self.source_kind) {
            return Err(DomainError::UnsupportedSkillSource);
        }
        
        Ok(())
    }

    fn resolve_entry_paths(&self) -> Result<(PathBuf, PathBuf), DomainError> {
        // Path resolution logic
    }
}

// src-tauri/src/domain/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid fingerprint")]
    InvalidFingerprint,
    
    #[error("Invalid alias")]
    InvalidAlias,
    
    #[error("Conflicting entry files")]
    ConflictingEntryFiles,
    
    #[error("Cannot copy skill to the same agent")]
    CannotCopyToSameAgent,
    
    #[error("Unsupported skill source")]
    UnsupportedSkillSource,
}
```

#### Repository Layer (Data Access Abstraction)

**Responsibility:**
- Define persistence interfaces
- Abstract storage details
- Provide CRUD operations
- Handle data mapping

**Principles:**
- Interface-based (trait)
- No business logic
- Return domain objects
- Hide storage implementation

**Example:**
```rust
// src-tauri/src/repositories/agent_repository.rs
pub trait AgentRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<ManagedAgent>, RepositoryError>;
    fn find_by_id(&self, id: &AgentId) -> Result<Option<ManagedAgent>, RepositoryError>;
    fn find_by_fingerprint(&self, fingerprint: &str) -> Result<Option<ManagedAgent>, RepositoryError>;
    fn save(&self, agent: &ManagedAgent) -> Result<(), RepositoryError>;
    fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError>;
    fn delete(&self, id: &AgentId) -> Result<(), RepositoryError>;
}

// src-tauri/src/repositories/skill_repository.rs
pub trait SkillRepository: Send + Sync {
    fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<Skill>, RepositoryError>;
    fn find_by_id(&self, id: &SkillId) -> Result<Option<Skill>, RepositoryError>;
    fn save(&self, skill: &Skill) -> Result<(), RepositoryError>;
    fn delete(&self, id: &SkillId) -> Result<(), RepositoryError>;
}

// src-tauri/src/repositories/mcp_config_repository.rs
pub trait McpConfigRepository: Send + Sync {
    fn load(&self, agent_type: AgentType, root_path: &Path) -> Result<McpConfig, RepositoryError>;
    fn save(&self, config: &McpConfig) -> Result<(), RepositoryError>;
    fn delete_server(&self, config_path: &Path, server_name: &str) -> Result<(), RepositoryError>;
}
```

#### Infrastructure Layer (Implementation Details)

**Responsibility:**
- Implement Repository traits
- File system operations
- External service integration
- Utility functions

**Principles:**
- Implement abstractions from upper layers
- Handle I/O operations
- No business logic
- Reusable utilities

**Example:**
```rust
// src-tauri/src/infrastructure/persistence/json_agent_repository.rs
pub struct JsonAgentRepository {
    store_path: PathBuf,
}

impl AgentRepository for JsonAgentRepository {
    fn find_all(&self) -> Result<Vec<ManagedAgent>, RepositoryError> {
        let records = self.load_records()?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError> {
        let records: Vec<ManagedAgentRecord> = agents.iter().map(Into::into).collect();
        self.write_records(&records)
    }

    // ... other implementations
}

// src-tauri/src/infrastructure/scanners/agent_scanner.rs
pub struct FileSystemAgentScanner;

impl AgentScanner for FileSystemAgentScanner {
    fn scan(&self, targets: Vec<ScanTarget>) -> Result<Vec<DiscoveredAgent>, ScanError> {
        targets
            .into_iter()
            .filter_map(|target| self.scan_target(target).ok())
            .collect()
    }

    fn scan_target(&self, target: ScanTarget) -> Result<DiscoveredAgent, ScanError> {
        // File system scanning logic
    }
}

// src-tauri/src/infrastructure/utils/path.rs
pub fn user_home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn resolve_agent_root(root_path: &str) -> PathBuf {
    if let Some(relative) = root_path.strip_prefix("~/").or_else(|| root_path.strip_prefix("~\\")) {
        return user_home_dir().join(relative);
    }
    
    let path = PathBuf::from(root_path);
    if path.is_absolute() {
        path
    } else {
        user_home_dir().join(path)
    }
}

// src-tauri/src/infrastructure/utils/fs.rs
pub fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn copy_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let next_dst = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_recursive(&path, &next_dst)?;
        } else {
            ensure_parent_dir(&next_dst)?;
            fs::copy(&path, &next_dst)?;
        }
    }
    
    Ok(())
}
```

## Refactoring Strategy

### Phase 1: Foundation (High Priority)

**Goal:** Establish shared utilities and repository abstractions

1. **Extract shared utilities**
   - Create `src-tauri/src/infrastructure/utils/path.rs`
   - Create `src-tauri/src/infrastructure/utils/fs.rs`
   - Consolidate all path/file operations

2. **Define Repository traits**
   - Create `src-tauri/src/repositories/agent_repository.rs`
   - Create `src-tauri/src/repositories/skill_repository.rs`
   - Create `src-tauri/src/repositories/mcp_config_repository.rs`

3. **Implement JSON-based repositories**
   - Move `persistence/managed_agents_store.rs` logic to `JsonAgentRepository`
   - Keep backward compatibility with existing storage format

**Estimated effort:** 2-3 days

### Phase 2: Domain Extraction (Medium Priority)

**Goal:** Extract domain models and business rules

4. **Create domain entities**
   - Extract `ManagedAgent` domain model
   - Extract `Skill` domain model with business rules
   - Define domain errors

5. **Refactor agent_discovery_service**
   - Split data transformation from business logic
   - Move matching logic to domain layer
   - Simplify service to orchestration only

6. **Refactor skill operations**
   - Move file operation logic from `commands/skills.rs` to domain
   - Extract skill copy validation to domain
   - Keep infrastructure concerns in infrastructure layer

**Estimated effort:** 3-4 days

### Phase 3: Service Simplification (Medium Priority)

**Goal:** Simplify services and commands layers

7. **Refactor MCP module**
   - Extract `McpConfigParser` as independent module
   - Create `McpInspector` domain service
   - Separate process management from business logic

8. **Simplify Commands layer**
   - Reduce command handlers to < 20 lines
   - Move all business logic to services
   - Keep only DTO conversion in commands

**Estimated effort:** 2-3 days

### Phase 4: Advanced Patterns (Low Priority)

**Goal:** Introduce advanced architectural patterns

9. **Introduce Domain Events**
   - Define events (AgentImported, SkillCopied, etc.)
   - Implement event bus
   - Decouple modules via events

10. **Add Use Case layer** (optional)
    - Create explicit use case classes
    - Document business scenarios
    - Improve testability

11. **Unified error handling**
    - Create error hierarchy
    - Implement error conversion chain
    - Improve error messages

**Estimated effort:** 3-4 days

## Directory Structure After Refactoring

```
src-tauri/src/
├── commands/              # Tauri command handlers (thin layer)
│   ├── agents.rs
│   ├── skills.rs
│   ├── mcp.rs
│   └── marketplace.rs
├── services/              # Business orchestration
│   ├── agent_service.rs
│   ├── skill_service.rs
│   ├── mcp_service.rs
│   └── marketplace_service.rs
├── domain/                # Domain models and business rules
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── managed_agent.rs
│   │   ├── discovered_agent.rs
│   │   └── resolved_agent.rs
│   ├── skill/
│   │   ├── mod.rs
│   │   ├── skill.rs
│   │   └── skill_copy.rs
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── server.rs
│   │   └── inspector.rs
│   └── errors.rs
├── repositories/          # Data access abstractions
│   ├── agent_repository.rs
│   ├── skill_repository.rs
│   ├── mcp_config_repository.rs
│   └── marketplace_repository.rs
├── infrastructure/        # Implementation details
│   ├── persistence/
│   │   ├── json_agent_repository.rs
│   │   ├── json_skill_repository.rs
│   │   └── file_mcp_config_repository.rs
│   ├── scanners/
│   │   ├── agent_scanner.rs
│   │   ├── skill_scanner.rs
│   │   └── mcp_scanner.rs
│   └── utils/
│       ├── path.rs
│       ├── fs.rs
│       └── process.rs
├── dto/                   # Data transfer objects
│   ├── agents.rs
│   ├── skills.rs
│   ├── mcp.rs
│   └── marketplace.rs
├── constants.rs
├── lib.rs
└── main.rs
```

## Migration Guidelines

### For Each Module

1. **Start with tests**
   - Write tests for existing behavior
   - Ensure tests pass before refactoring

2. **Extract utilities first**
   - Move shared code to infrastructure/utils
   - Update all references

3. **Define domain models**
   - Create domain entities
   - Move business rules to domain

4. **Create repository interface**
   - Define trait
   - Implement with existing storage logic

5. **Refactor service**
   - Use repository instead of direct storage
   - Use domain models instead of DTOs
   - Keep orchestration logic only

6. **Simplify command**
   - Remove business logic
   - Add DTO conversion
   - Call service methods

7. **Run tests**
   - Verify all tests still pass
   - Add new tests for domain logic

### Backward Compatibility

- Keep existing storage format unchanged
- Maintain existing API contracts
- Use adapter pattern for gradual migration
- Run integration tests after each phase

## Benefits After Refactoring

### Code Quality

- **Single Responsibility:** Each layer has clear, focused responsibilities
- **DRY Principle:** Shared utilities eliminate duplication
- **Testability:** Domain logic can be tested without I/O
- **Maintainability:** Changes are localized to specific layers

### Architecture

- **Separation of Concerns:** Clear boundaries between layers
- **Dependency Inversion:** High-level modules don't depend on low-level details
- **Open/Closed Principle:** Easy to extend without modifying existing code
- **Interface Segregation:** Focused, minimal interfaces

### Development

- **Easier onboarding:** Clear structure helps new developers
- **Parallel development:** Teams can work on different layers independently
- **Reduced bugs:** Business logic isolated from infrastructure concerns
- **Better documentation:** Architecture itself documents the system

## Testing Strategy

### Unit Testing by Layer

#### Domain Layer Testing
```rust
// src-tauri/src/domain/agent.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_agent_matches_by_fingerprint() {
        let managed = ManagedAgent::new(
            "claude-default".into(),
            AgentType::Claude,
            PathBuf::from(".claude"),
        ).unwrap();
        
        let discovered = DiscoveredAgent {
            fingerprint: "claude-default".into(),
            agent_type: AgentType::Claude,
            root_path: PathBuf::from(".claude"),
        };
        
        assert!(managed.matches(&discovered));
    }

    #[test]
    fn set_alias_rejects_empty_string() {
        let mut agent = ManagedAgent::new(
            "test".into(),
            AgentType::Claude,
            PathBuf::from("."),
        ).unwrap();
        
        let result = agent.set_alias("   ".into());
        assert!(matches!(result, Err(DomainError::InvalidAlias)));
    }
}
```

#### Repository Layer Testing (with Mock)
```rust
// src-tauri/src/repositories/agent_repository.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub AgentRepo {}
        
        impl AgentRepository for AgentRepo {
            fn find_all(&self) -> Result<Vec<ManagedAgent>, RepositoryError>;
            fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError>;
        }
    }

    #[test]
    fn test_service_with_mock_repository() {
        let mut mock_repo = MockAgentRepo::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|| Ok(vec![]));
        
        let service = AgentService::new(Arc::new(mock_repo), Arc::new(mock_scanner));
        let result = service.list_managed();
        assert!(result.is_ok());
    }
}
```

#### Service Layer Testing
```rust
// src-tauri/src/services/agent_service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_agents_merges_discovered_and_managed() {
        let mock_repo = create_mock_repository();
        let mock_scanner = create_mock_scanner();
        let service = AgentService::new(mock_repo, mock_scanner);
        
        let result = service.import_agents(
            vec!["candidate-1".into()],
            vec![create_scan_target()],
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().imported_agents.len(), 1);
    }
}
```

#### Integration Testing
```rust
// src-tauri/tests/integration/agent_workflow.rs
#[test]
fn complete_agent_import_workflow() {
    let temp_dir = create_temp_test_dir();
    let app_state = create_test_app_state(&temp_dir);
    
    // Scan agents
    let scan_result = commands::agents::scan_agents(
        State(&app_state),
        vec![create_test_scan_target()],
    ).unwrap();
    
    assert!(!scan_result.is_empty());
    
    // Import first candidate
    let import_result = commands::agents::import_agents(
        State(&app_state),
        vec![scan_result[0].id.clone()],
        vec![create_test_scan_target()],
    ).unwrap();
    
    assert_eq!(import_result.imported_agents.len(), 1);
    
    // Verify persistence
    let managed = commands::agents::list_managed_agents(State(&app_state)).unwrap();
    assert_eq!(managed.len(), 1);
}
```

### Test Coverage Goals

- **Domain Layer:** 90%+ coverage (pure logic, easy to test)
- **Repository Layer:** 80%+ coverage (test implementations, not traits)
- **Service Layer:** 85%+ coverage (orchestration logic)
- **Commands Layer:** 70%+ coverage (thin layer, mostly integration tests)
- **Infrastructure Layer:** 75%+ coverage (utility functions)

### Testing Tools

```toml
# Cargo.toml
[dev-dependencies]
mockall = "0.12"           # Mocking framework
tempfile = "3.8"           # Temporary directories for tests
serial_test = "3.0"        # Sequential test execution
proptest = "1.4"           # Property-based testing
criterion = "0.5"          # Benchmarking
```

## Dependency Injection & State Management

### AppState Structure

```rust
// src-tauri/src/lib.rs
pub struct AppState {
    pub agent_service: Arc<AgentService>,
    pub skill_service: Arc<SkillService>,
    pub mcp_service: Arc<McpService>,
    pub marketplace_service: Arc<MarketplaceService>,
}

impl AppState {
    pub fn new() -> Self {
        // Create infrastructure components
        let agent_repo = Arc::new(JsonAgentRepository::new());
        let skill_repo = Arc::new(JsonSkillRepository::new());
        let mcp_config_repo = Arc::new(FileMcpConfigRepository::new());
        
        let agent_scanner = Arc::new(FileSystemAgentScanner::new());
        let skill_scanner = Arc::new(FileSystemSkillScanner::new());
        
        // Create services with injected dependencies
        let agent_service = Arc::new(AgentService::new(
            agent_repo.clone(),
            agent_scanner,
        ));
        
        let skill_service = Arc::new(SkillService::new(
            skill_repo,
            agent_repo.clone(),
        ));
        
        let mcp_service = Arc::new(McpService::new(
            mcp_config_repo,
        ));
        
        let marketplace_service = Arc::new(MarketplaceService::new());
        
        Self {
            agent_service,
            skill_service,
            mcp_service,
            marketplace_service,
        }
    }
}

// Register in Tauri
pub fn run() {
    let app_state = AppState::new();
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::agents::list_managed_agents,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Service Lifetime Management

```rust
// Services are created once at app startup and shared via Arc
// This ensures:
// 1. Single instance per service (singleton pattern)
// 2. Thread-safe sharing across Tauri commands
// 3. Efficient memory usage
// 4. Consistent state across operations

pub struct AgentService {
    agent_repo: Arc<dyn AgentRepository>,
    scanner: Arc<dyn AgentScanner>,
    // Optional: Add cache with Mutex for mutable state
    cache: Arc<Mutex<HashMap<String, CachedData>>>,
}
```

## Error Handling Strategy

### Error Hierarchy

```rust
// src-tauri/src/domain/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid fingerprint: {0}")]
    InvalidFingerprint(String),
    
    #[error("Invalid alias: {0}")]
    InvalidAlias(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Conflicting entry files at {0}")]
    ConflictingEntryFiles(String),
    
    #[error("Cannot copy skill to the same agent")]
    CannotCopyToSameAgent,
}

// src-tauri/src/repositories/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// src-tauri/src/services/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Scanner error: {0}")]
    Scanner(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
}
```

### Error Conversion Chain

```rust
// src-tauri/src/commands/agents.rs
impl From<ServiceError> for String {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Domain(e) => format!("Validation error: {}", e),
            ServiceError::Repository(e) => format!("Storage error: {}", e),
            ServiceError::Scanner(e) => format!("Scan error: {}", e),
            ServiceError::BusinessRuleViolation(e) => format!("Operation failed: {}", e),
        }
    }
}

#[tauri::command]
pub fn import_agents(
    state: State<AppState>,
    candidate_ids: Vec<String>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<ImportAgentsResultDto, String> {
    let targets = scan_targets.into_iter().map(Into::into).collect();
    let result = state.agent_service
        .import_agents(candidate_ids, targets)
        .map_err(|e| e.into())?; // Convert ServiceError to String
    Ok(result.into())
}
```

### Logging Strategy

```rust
// Add to Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"

// src-tauri/src/lib.rs
use tracing::{info, warn, error, debug};

pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Starting AgentDock application");
    
    // ... rest of setup
}

// In services
impl AgentService {
    pub fn import_agents(&self, ids: Vec<String>, targets: Vec<ScanTarget>) 
        -> Result<ImportResult, ServiceError> 
    {
        info!("Importing {} agents", ids.len());
        debug!("Import targets: {:?}", targets);
        
        let result = self.process_imports(ids, targets)?;
        
        info!("Successfully imported {} agents", result.imported_agents.len());
        Ok(result)
    }
}
```

## Performance Considerations

### Caching Strategy

```rust
// src-tauri/src/services/agent_service.rs
use std::time::{Duration, Instant};

pub struct AgentService {
    agent_repo: Arc<dyn AgentRepository>,
    scanner: Arc<dyn AgentScanner>,
    cache: Arc<Mutex<AgentCache>>,
}

struct AgentCache {
    discovered_agents: Option<(Instant, Vec<DiscoveredAgent>)>,
    cache_duration: Duration,
}

impl AgentCache {
    fn new() -> Self {
        Self {
            discovered_agents: None,
            cache_duration: Duration::from_secs(60), // 1 minute cache
        }
    }
    
    fn get(&self) -> Option<&Vec<DiscoveredAgent>> {
        self.discovered_agents.as_ref().and_then(|(timestamp, agents)| {
            if timestamp.elapsed() < self.cache_duration {
                Some(agents)
            } else {
                None
            }
        })
    }
    
    fn set(&mut self, agents: Vec<DiscoveredAgent>) {
        self.discovered_agents = Some((Instant::now(), agents));
    }
    
    fn invalidate(&mut self) {
        self.discovered_agents = None;
    }
}

impl AgentService {
    pub fn list_resolved(&self, targets: Vec<ScanTarget>) -> Result<Vec<ResolvedAgent>> {
        let mut cache = self.cache.lock().unwrap();
        
        let discovered = if let Some(cached) = cache.get() {
            cached.clone()
        } else {
            let scanned = self.scanner.scan(targets)?;
            cache.set(scanned.clone());
            scanned
        };
        
        let managed = self.agent_repo.find_all()?;
        Ok(self.merge_agents(discovered, managed))
    }
    
    pub fn refresh_cache(&self) {
        self.cache.lock().unwrap().invalidate();
    }
}
```

### Async Operations

```rust
// For I/O-heavy operations, use async
use tokio::task;

impl AgentService {
    pub async fn scan_agents_async(&self, targets: Vec<ScanTarget>) 
        -> Result<Vec<DiscoveredAgent>, ServiceError> 
    {
        let scanner = self.scanner.clone();
        
        // Run blocking scan in separate thread pool
        let discovered = task::spawn_blocking(move || {
            scanner.scan(targets)
        })
        .await
        .map_err(|e| ServiceError::Scanner(e.to_string()))??;
        
        Ok(discovered)
    }
}

// For parallel scanning of multiple targets
impl FileSystemAgentScanner {
    pub fn scan(&self, targets: Vec<ScanTarget>) -> Result<Vec<DiscoveredAgent>, ScanError> {
        use rayon::prelude::*;
        
        targets
            .par_iter() // Parallel iteration
            .filter_map(|target| self.scan_target(target).ok())
            .collect()
    }
}
```

### Batch Operations

```rust
// Optimize bulk operations
impl AgentRepository for JsonAgentRepository {
    fn save_all(&self, agents: &[ManagedAgent]) -> Result<(), RepositoryError> {
        // Single file write instead of multiple writes
        let records: Vec<ManagedAgentRecord> = agents.iter().map(Into::into).collect();
        self.write_records(&records)
    }
}
```

## Complete Refactoring Example

### Before: commands/skills.rs (Excerpt)

```rust
// BEFORE: 1048 lines, mixed concerns
#[tauri::command]
pub fn set_local_skill_enabled(
    skill_path: String,
    entry_file_path: String,
    enabled: bool,
) -> Result<(), String> {
    // Direct file system operations in command handler
    let entry_path = Path::new(&entry_file_path);
    let active_entry_path = enabled_entry_path(entry_path, &entry_file_path)?;
    let disabled_entry_path = disabled_entry_path(&active_entry_path, &entry_file_path)?;
    
    // Validation logic mixed with I/O
    validate_skill_path(&skill_path, &active_entry_path)?;
    
    let active_exists = active_entry_path.is_file();
    let disabled_exists = disabled_entry_path.is_file();
    
    // Business logic in command handler
    if !active_exists && !disabled_exists {
        return Err(format!("Skill entry file not found: {entry_file_path}"));
    }
    
    // More file operations...
    if enabled {
        if active_exists {
            return Ok(());
        }
        return fs::rename(&disabled_entry_path, &active_entry_path)
            .map_err(|error| error.to_string());
    }
    
    // ... 50+ more lines
}
```

### After: Layered Architecture

```rust
// AFTER: Commands layer (thin)
// src-tauri/src/commands/skills.rs
#[tauri::command]
pub fn set_local_skill_enabled(
    state: State<AppState>,
    skill_id: String,
    enabled: bool,
) -> Result<(), String> {
    state.skill_service
        .set_enabled(&skill_id, enabled)
        .map_err(Into::into)
}

// Service layer (orchestration)
// src-tauri/src/services/skill_service.rs
impl SkillService {
    pub fn set_enabled(&self, skill_id: &str, enabled: bool) 
        -> Result<(), ServiceError> 
    {
        let mut skill = self.skill_repo
            .find_by_id(skill_id)?
            .ok_or(ServiceError::NotFound(skill_id.into()))?;
        
        skill.set_enabled(enabled)?; // Domain validation
        
        self.skill_repo.save(&skill)?; // Persistence
        
        Ok(())
    }
}

// Domain layer (business rules)
// src-tauri/src/domain/skill.rs
impl Skill {
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), DomainError> {
        if self.enabled == enabled {
            return Ok(()); // Already in desired state
        }
        
        let (active_path, disabled_path) = self.resolve_entry_paths()?;
        
        if enabled && disabled_path.exists() && active_path.exists() {
            return Err(DomainError::ConflictingEntryFiles(
                active_path.display().to_string()
            ));
        }
        
        if !enabled && active_path.exists() && disabled_path.exists() {
            return Err(DomainError::ConflictingEntryFiles(
                disabled_path.display().to_string()
            ));
        }
        
        self.enabled = enabled;
        Ok(())
    }
}

// Repository layer (persistence)
// src-tauri/src/infrastructure/persistence/json_skill_repository.rs
impl SkillRepository for JsonSkillRepository {
    fn save(&self, skill: &Skill) -> Result<(), RepositoryError> {
        // Handle file renaming based on enabled state
        if skill.enabled {
            self.rename_to_active(&skill.entry_file)?;
        } else {
            self.rename_to_disabled(&skill.entry_file)?;
        }
        
        // Update metadata store
        self.update_metadata(skill)?;
        
        Ok(())
    }
}
```

### Comparison

| Aspect | Before | After |
|--------|--------|-------|
| Lines in command | 50+ | 5 |
| Testability | Hard (I/O coupled) | Easy (pure logic) |
| Reusability | Low | High |
| Maintainability | Poor | Good |
| Separation of Concerns | None | Clear |

## Risk Assessment & Mitigation

### Identified Risks

#### 1. Breaking Changes
**Risk:** Refactoring may introduce bugs or break existing functionality

**Mitigation:**
- Comprehensive test suite before refactoring
- Feature flag for gradual rollout
- Parallel implementation (keep old code until new code is verified)
- Extensive integration testing

#### 2. Performance Regression
**Risk:** New abstraction layers may introduce overhead

**Mitigation:**
- Benchmark critical paths before and after
- Profile with `cargo flamegraph`
- Use `criterion` for micro-benchmarks
- Monitor real-world performance metrics

#### 3. Team Productivity Impact
**Risk:** Learning curve may slow down development

**Mitigation:**
- Comprehensive documentation
- Code examples for each layer
- Pair programming sessions
- Gradual migration (one module at a time)

#### 4. Incomplete Migration
**Risk:** Partial refactoring leaves codebase in inconsistent state

**Mitigation:**
- Clear phase boundaries
- Complete one module before starting next
- Document migration status
- Regular progress reviews

### Rollback Strategy

```rust
// Use feature flags for gradual migration
#[cfg(feature = "new-architecture")]
use crate::services::agent_service::AgentService;

#[cfg(not(feature = "new-architecture"))]
use crate::services::agent_discovery_service as agent_service;

#[tauri::command]
pub fn list_managed_agents(state: State<AppState>) -> Result<Vec<ManagedAgentDto>, String> {
    #[cfg(feature = "new-architecture")]
    {
        state.agent_service.list_managed()
            .map(|agents| agents.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
    
    #[cfg(not(feature = "new-architecture"))]
    {
        Ok(agent_service::list_managed_agents())
    }
}
```

### Minimizing Impact

1. **Backward Compatibility**
   - Keep existing storage format
   - Maintain API contracts
   - No breaking changes to frontend

2. **Incremental Deployment**
   - Deploy one module at a time
   - Monitor for issues after each deployment
   - Quick rollback capability

3. **Continuous Integration**
   - Run full test suite on every commit
   - Automated integration tests
   - Performance regression tests

## Success Metrics

### Code Quality Metrics

#### Before Refactoring (Baseline)
```bash
# Measure current state
cargo clippy -- -D warnings
cargo test
cargo tarpaulin --out Html  # Code coverage

# Complexity analysis
tokei src-tauri/src/          # Lines of code
cargo-geiger                   # Unsafe code usage
```

**Expected Baseline:**
- Total lines: ~8,000
- Average function length: 30-50 lines
- Cyclomatic complexity: 10-15 (high)
- Test coverage: 40-50%
- Clippy warnings: 20-30

#### After Refactoring (Target)
- Total lines: ~10,000 (increased due to abstractions, but better organized)
- Average function length: 10-20 lines
- Cyclomatic complexity: 3-5 (low)
- Test coverage: 75-85%
- Clippy warnings: 0

### Architecture Metrics

| Metric | Before | Target | Measurement |
|--------|--------|--------|-------------|
| Layers | 2 (Commands, Services) | 5 (Commands, Services, Domain, Repository, Infrastructure) | Manual count |
| Abstraction level | Low | High | Interface count |
| Coupling | High | Low | Dependency graph |
| Cohesion | Low | High | Module analysis |
| Testability | 3/10 | 8/10 | Mock usage |

### Performance Metrics

```rust
// Benchmark critical operations
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_agent_scan(c: &mut Criterion) {
    let service = create_test_service();
    let targets = create_test_targets();
    
    c.bench_function("agent_scan", |b| {
        b.iter(|| {
            service.scan_agents(black_box(targets.clone()))
        })
    });
}

criterion_group!(benches, benchmark_agent_scan);
criterion_main!(benches);
```

**Performance Targets:**
- Agent scan: < 100ms (no regression)
- Skill list: < 50ms (no regression)
- MCP config load: < 20ms (no regression)
- Memory usage: < 50MB baseline (no significant increase)

### Monitoring Dashboard

```rust
// Add metrics collection
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    command_duration: Histogram,
    command_errors: Counter,
    cache_hits: Counter,
    cache_misses: Counter,
}

impl Metrics {
    pub fn record_command(&self, name: &str, duration: Duration, success: bool) {
        self.command_duration
            .with_label_values(&[name])
            .observe(duration.as_secs_f64());
        
        if !success {
            self.command_errors
                .with_label_values(&[name])
                .inc();
        }
    }
}
```

## Next Steps

1. Review and approve this refactoring plan
2. Create feature branch for refactoring work
3. Set up metrics collection baseline
4. Start with Phase 1 (Foundation)
5. Conduct code review after each phase
6. Run performance benchmarks after each phase
7. Merge to main after all tests pass
8. Update documentation and CLAUDE.md files
9. Monitor production metrics for regressions

## References

- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design by Eric Evans](https://www.domainlanguage.com/ddd/)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Testing in Rust](https://doc.rust-lang.org/book/ch11-00-testing.html)
