[Root](../CLAUDE.md) > **src-tauri**

# Backend Module (src-tauri)

## Responsibilities

The backend owns Tauri runtime wiring, desktop plugins, local file-system discovery, durable managed agent state, and the command surface consumed by the React frontend.

## Entry Points

- **Binary entry**: `src/main.rs`
- **Runtime wiring**: `src/lib.rs`
- **Build manifest**: `Cargo.toml`
- **Tauri config**: `tauri.conf.json`
- **Permissions**: `capabilities/default.json`

## Module Map

| Module | Path | Responsibility |
| --- | --- | --- |
| Commands | `src/commands/` | Thin Tauri command handlers exposed to the frontend |
| DTOs | `src/dto/` | Serializable request/response types |
| Domain | `src/domain/` | Business logic, entities, and domain errors |
| Repositories | `src/repositories/` | Data access abstractions and repository errors |
| Infrastructure | `src/infrastructure/` | Concrete implementations (persistence, filesystem, clients) |
| Services | `src/services/` | Domain orchestration and service logic |
| Scanners | `src/scanners/` | Local discovery scanners |
| Plugins | `src/plugins/` | Tauri plugins and tray integration |

## Current Backend Focus

1. Agent discovery and managed agent resolution using the 5-layer architecture.
2. Skill discovery under managed agent roots.
3. Dependency injection and repository-based persistence.
4. Error hierarchy implementation across layers.

## Important Files

- `src/lib.rs` — plugin setup and command registration
- `src/domain/mod.rs` — core domain models and error types
- `src/repositories/mod.rs` — repository trait definitions
- `src/infrastructure/mod.rs` — concrete repository implementations
- `src/services/mod.rs` — orchestration services
- `src/commands/mod.rs` — tauri commands

## Command Flow Pattern

### Add or extend a command family

1. Define domain models in `src/domain/`.
2. Define repository traits in `src/repositories/`.
3. Implement infrastructure in `src/infrastructure/`.
4. Add orchestration in `src/services/`.
5. Expose thin `#[tauri::command]` wrappers in `src/commands/`.
6. Update `mod.rs` exports and register commands in `src/lib.rs`.

## Backend Architecture (5-Layer)

- **Commands**: Entry point for frontend calls. Minimal logic.
- **Services**: Orchestrates domain logic and repositories.
- **Domain**: Pure business logic and entities. No infrastructure dependencies.
- **Repositories**: Interfaces for data access.
- **Infrastructure**: Low-level details (FS, DB, external APIs).

## Error Handling

- `DomainError`: Logic-related errors.
- `RepositoryError`: Data access errors.
- `ServiceError`: Orchestration errors (wraps others).
- `Command Result`: Commands return `Result<T, E>` where E is serializable.

## Backend Conventions

- Use constructor-based dependency injection for services and repositories.
- Keep `domain` free of `serde` if possible (except for DTOs).
- Register every new command in `src/lib.rs`.
- Keep comments, logs, and user-facing strings in English.

## Safety and Scope

- Avoid leaking raw filesystem assumptions into the frontend; expose typed DTOs instead.
- Prefer returning structured DTOs over ad-hoc JSON strings.
- If a new feature touches both scan logic and command exposure, verify scanner → service → command → frontend alignment before considering it done.

## Verification Checklist

Before finishing backend changes in this module, verify:

1. New modules are exported from the corresponding `mod.rs` files.
2. Commands are registered in `src/lib.rs`.
3. DTOs serialize cleanly for Tauri.
4. Rust checks pass for the backend crate.
5. Frontend invoke names still match backend command names.
