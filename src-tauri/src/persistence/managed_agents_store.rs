use std::{env, fs, path::PathBuf};

use crate::dto::agents::ManagedAgentDto;

const STORE_DIR_NAME: &str = ".agentdock";
const STORE_FILE_NAME: &str = "managed-agents.json";

fn store_dir_path() -> PathBuf {
    let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_dir.join(STORE_DIR_NAME)
}

fn store_file_path() -> PathBuf {
    store_dir_path().join(STORE_FILE_NAME)
}

fn ensure_store_seeded() -> Result<Vec<ManagedAgentDto>, String> {
    let file_path = store_file_path();
    if file_path.exists() {
        let contents = fs::read_to_string(&file_path).map_err(|error| error.to_string())?;
        let agents = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
        return Ok(agents);
    }

    let agents = default_managed_agents();
    save_managed_agents(&agents)?;
    Ok(agents)
}

pub fn load_managed_agents() -> Vec<ManagedAgentDto> {
    ensure_store_seeded().unwrap_or_else(|_| default_managed_agents())
}

pub fn save_managed_agents(agents: &[ManagedAgentDto]) -> Result<(), String> {
    let dir_path = store_dir_path();
    fs::create_dir_all(&dir_path).map_err(|error| error.to_string())?;

    let file_path = dir_path.join(STORE_FILE_NAME);
    let contents = serde_json::to_string_pretty(agents).map_err(|error| error.to_string())?;
    fs::write(file_path, contents).map_err(|error| error.to_string())
}

pub fn default_managed_agents() -> Vec<ManagedAgentDto> {
    vec![
        ManagedAgentDto {
            managed_agent_id: "managed-cursor".into(),
            fingerprint: "cursor-workspace-default".into(),
            alias: None,
            enabled: true,
            hidden: false,
            imported_at: "2026-03-24T08:00:00Z".into(),
            source: "auto-imported".into(),
            agent_type: Some("cursor".into()),
            root_path: Some(".cursor".into()),
        },
        ManagedAgentDto {
            managed_agent_id: "managed-claude".into(),
            fingerprint: "claude-default".into(),
            alias: Some("Claude Main".into()),
            enabled: true,
            hidden: false,
            imported_at: "2026-03-24T08:30:00Z".into(),
            source: "manual-imported".into(),
            agent_type: Some("claude".into()),
            root_path: Some(".claude".into()),
        },
        ManagedAgentDto {
            managed_agent_id: "managed-antigravity".into(),
            fingerprint: "antigravity-manual-default".into(),
            alias: None,
            enabled: false,
            hidden: false,
            imported_at: "2026-03-24T09:00:00Z".into(),
            source: "manual-imported".into(),
            agent_type: Some("antigravity".into()),
            root_path: Some(".agent".into()),
        },
    ]
}
