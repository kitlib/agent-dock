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
| DTOs | `src/dto/` | Serializable request/response types shared across commands and services |
| Scanners | `src/scanners/` | Local disk scanning for providers and skills |
| Services | `src/services/` | Orchestration of discovery and detail lookup |
| Persistence | `src/persistence/` | Managed agent storage and local durable state |
| Plugins | `src/plugins/` | System tray and other runtime plugin integration |

## Current Backend Focus

1. Agent discovery and managed agent resolution.
2. Skill discovery under managed agent roots.
3. Tauri command exposure for agent and skill operations.
4. Desktop shell integrations such as tray behavior, updater gating, and single-instance handling.

## Important Files

- `src/lib.rs` — plugin setup and command registration
- `src/commands/agents.rs` — agent command handlers
- `src/commands/skills.rs` — skill command handlers
- `src/services/agent_discovery_service.rs` — resolved agent orchestration
- `src/services/skill_discovery_service.rs` — local skill summary/detail orchestration
- `src/scanners/provider_scanner.rs` — provider scan logic
- `src/scanners/skill_scanner.rs` — skill metadata discovery from local folders
- `src/persistence/managed_agents_store.rs` — durable managed-agent storage

## Command Flow Pattern

### Add or extend a command family

1. Define serializable types in `src/dto/`.
2. Implement scanner or persistence helpers as needed.
3. Add orchestration in `src/services/`.
4. Expose thin `#[tauri::command]` wrappers in `src/commands/`.
5. Export the module from the relevant `mod.rs` files.
6. Register commands in `src/lib.rs`.
7. Keep command names aligned with frontend `invoke()` usage.

### Current registered command groups

- Core demo/system: `greet`, `update_tray_menu`
- Agent management: `list_managed_agents`, `list_resolved_agents`, `scan_agents`, `import_agents`, `remove_managed_agent`, `delete_agent`, `create_agent`, `refresh_agent_discovery`
- Skill discovery: `list_local_skills`, `get_local_skill_detail`

## Dependency Notes

Key backend crates currently include:

- `tauri` with `tray-icon`
- `tauri-plugin-opener`
- `tauri-plugin-global-shortcut`
- `tauri-plugin-single-instance`
- `tauri-plugin-updater`
- `serde`, `serde_json`, `serde_yaml`
- `chrono`

`serde_yaml` and `chrono` indicate local metadata parsing and timestamp handling in discovery flows, so keep DTOs and parsers consistent when extending scanner behavior.

## Development Commands

```bash
pnpm tauri dev
pnpm tauri build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## Backend Conventions

- Keep `commands/` thin and free of business logic.
- Put scan-time file-system traversal in `scanners/`.
- Put multi-step orchestration and shaping in `services/`.
- Put local durable state reads/writes in `persistence/`.
- Update `mod.rs` exports whenever adding a new backend module.
- Register every new frontend-facing command in `src/lib.rs` immediately.
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
