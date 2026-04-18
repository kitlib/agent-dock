use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri_plugin_opener::OpenerExt;

use crate::dto::mcp::{LocalMcpServerDto, McpScanTargetDto};
use crate::services::mcp_discovery_service;

const CLAUDE_CONFIG_FILE: &str = ".claude.json";
const CODEX_CONFIG_FILE: &str = "config.toml";
const GEMINI_CONFIG_FILE: &str = "settings.json";

#[derive(Clone, Copy, PartialEq, Eq)]
enum McpImportConflictStrategy {
    Overwrite,
    Skip,
}

#[derive(Clone)]
struct ImportedMcpServer {
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    url: Option<String>,
    headers: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalMcpResultDto {
    pub config_path: String,
    pub imported_count: u32,
    pub skipped_count: u32,
    pub imported_names: Vec<String>,
    pub skipped_names: Vec<String>,
}

fn user_home_dir() -> PathBuf {
    if let Ok(home) = env::var("AGENT_DOCK_TEST_HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_path(path: &str) -> PathBuf {
    if let Some(relative_path) = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix("~\\"))
    {
        return user_home_dir().join(relative_path);
    }

    let path = PathBuf::from(path);
    if path.is_absolute() {
        return path;
    }

    user_home_dir().join(path)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn resolve_import_conflict_strategy(value: &str) -> Result<McpImportConflictStrategy, String> {
    match value {
        "overwrite" => Ok(McpImportConflictStrategy::Overwrite),
        "skip" => Ok(McpImportConflictStrategy::Skip),
        _ => Err(format!("Unsupported MCP import conflict strategy: {value}")),
    }
}

fn resolve_mcp_config_path(agent_type: &str, root_path: &str) -> Result<PathBuf, String> {
    match agent_type {
        "claude" => Ok(user_home_dir().join(CLAUDE_CONFIG_FILE)),
        "codex" => Ok(resolve_path(root_path).join(CODEX_CONFIG_FILE)),
        "gemini" => Ok(resolve_path(root_path).join(GEMINI_CONFIG_FILE)),
        "opencode" => Ok(user_home_dir()
            .join(".config")
            .join("opencode")
            .join("opencode.json")),
        _ => Err(format!("Unsupported MCP agent type: {agent_type}")),
    }
}

fn ensure_parent_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!(
            "MCP config path has no parent directory: {}",
            normalize_path(path)
        ));
    };

    fs::create_dir_all(parent).map_err(|error| error.to_string())
}

fn write_json_value(path: &Path, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn read_json_root_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }

    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())
}

fn read_toml_root_or_empty(path: &Path) -> Result<toml::Value, String> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::value::Table::new()));
    }

    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())
}

fn parse_string_array(field: &str, value: Option<&Value>) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(format!("MCP field '{field}' must be an array of strings."));
    };

    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("MCP field '{field}' must contain only strings."))
        })
        .collect()
}

fn parse_string_map(field: &str, value: Option<&Value>) -> Result<BTreeMap<String, String>, String> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Some(object) = value.as_object() else {
        return Err(format!("MCP field '{field}' must be an object of strings."));
    };

    object
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|entry| (key.clone(), entry.to_string()))
                .ok_or_else(|| format!("MCP field '{field}' must contain only string values."))
        })
        .collect()
}

fn reserved_server_fields() -> BTreeSet<&'static str> {
    ["type", "command", "args", "env", "url", "httpUrl", "headers"]
        .into_iter()
        .collect()
}

fn parse_imported_mcp_server(server_name: &str, value: &Value) -> Result<ImportedMcpServer, String> {
    let Some(server) = value.as_object() else {
        return Err(format!("MCP server '{server_name}' must be an object."));
    };

    let allowed_fields = reserved_server_fields();
    for key in server.keys() {
        if !allowed_fields.contains(key.as_str()) {
            return Err(format!(
                "MCP server '{server_name}' contains unsupported field '{key}'."
            ));
        }
    }

    let explicit_type = server.get("type").and_then(Value::as_str);
    let command = server
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string);
    let args = parse_string_array("args", server.get("args"))?;
    let env = parse_string_map("env", server.get("env"))?;
    let headers = parse_string_map("headers", server.get("headers"))?;
    let url = server
        .get("httpUrl")
        .or_else(|| server.get("url"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let transport = match explicit_type.unwrap_or_default() {
        "" => {
            if command.is_some() {
                "stdio".to_string()
            } else if url.is_some() {
                "http".to_string()
            } else {
                return Err(format!(
                    "MCP server '{server_name}' must include either 'command' or 'url'."
                ));
            }
        }
        "stdio" | "local" => "stdio".to_string(),
        "http" => "http".to_string(),
        "sse" | "remote" => "sse".to_string(),
        other => return Err(format!("MCP server '{server_name}' has unsupported type '{other}'.")),
    };

    if transport == "stdio" && command.is_none() {
        return Err(format!(
            "MCP server '{server_name}' requires 'command' for stdio transport."
        ));
    }
    if transport != "stdio" && url.is_none() {
        return Err(format!(
            "MCP server '{server_name}' requires 'url' for remote transport."
        ));
    }

    Ok(ImportedMcpServer {
        transport,
        command,
        args,
        env,
        url,
        headers,
    })
}

fn parse_imported_mcp_payload(payload: &str) -> Result<BTreeMap<String, ImportedMcpServer>, String> {
    let root = serde_json::from_str::<Value>(payload).map_err(|error| error.to_string())?;
    let Some(object) = root.as_object() else {
        return Err("MCP import JSON must be an object.".into());
    };

    let reserved_fields = reserved_server_fields();
    let server_map = if let Some(servers) = object.get("mcpServers") {
        servers
            .as_object()
            .ok_or_else(|| "MCP field 'mcpServers' must be an object.".to_string())?
    } else {
        if object.keys().any(|key| reserved_fields.contains(key.as_str())) {
            return Err(
                "MCP import JSON must be either a 'mcpServers' object or a map keyed by server name."
                    .into(),
            );
        }
        object
    };

    if server_map.is_empty() {
        return Err("MCP import JSON does not contain any servers.".into());
    }

    server_map
        .iter()
        .map(|(server_name, value)| {
            parse_imported_mcp_server(server_name, value).map(|server| (server_name.clone(), server))
        })
        .collect()
}

fn import_claude_mcp_servers(
    path: &Path,
    servers: &BTreeMap<String, ImportedMcpServer>,
    conflict_strategy: McpImportConflictStrategy,
) -> Result<ImportLocalMcpResultDto, String> {
    ensure_parent_directory(path)?;
    let mut root = read_json_root_or_empty(path)?;
    let Some(root_object) = root.as_object_mut() else {
        return Err("Claude MCP config root must be an object.".into());
    };
    let servers_value = root_object
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(existing_servers) = servers_value.as_object_mut() else {
        return Err("Claude MCP config field 'mcpServers' must be an object.".into());
    };

    let mut imported_names = Vec::new();
    let mut skipped_names = Vec::new();
    for (server_name, server) in servers {
        if existing_servers.contains_key(server_name)
            && conflict_strategy == McpImportConflictStrategy::Skip
        {
            skipped_names.push(server_name.clone());
            continue;
        }

        let mut value = Map::new();
        value.insert("type".into(), Value::String(server.transport.clone()));
        if let Some(command) = &server.command {
            value.insert("command".into(), Value::String(command.clone()));
        }
        if !server.args.is_empty() {
            value.insert(
                "args".into(),
                Value::Array(server.args.iter().cloned().map(Value::String).collect()),
            );
        }
        if !server.env.is_empty() {
            value.insert(
                "env".into(),
                Value::Object(
                    server
                        .env
                        .iter()
                        .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                        .collect(),
                ),
            );
        }
        if let Some(url) = &server.url {
            value.insert("url".into(), Value::String(url.clone()));
        }
        if !server.headers.is_empty() {
            value.insert(
                "headers".into(),
                Value::Object(
                    server
                        .headers
                        .iter()
                        .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                        .collect(),
                ),
            );
        }

        existing_servers.insert(server_name.clone(), Value::Object(value));
        imported_names.push(server_name.clone());
    }

    write_json_value(path, &root)?;
    Ok(ImportLocalMcpResultDto {
        config_path: normalize_path(path),
        imported_count: imported_names.len() as u32,
        skipped_count: skipped_names.len() as u32,
        imported_names,
        skipped_names,
    })
}

fn import_gemini_mcp_servers(
    path: &Path,
    servers: &BTreeMap<String, ImportedMcpServer>,
    conflict_strategy: McpImportConflictStrategy,
) -> Result<ImportLocalMcpResultDto, String> {
    import_claude_mcp_servers(path, servers, conflict_strategy)
}

fn import_codex_mcp_servers(
    path: &Path,
    servers: &BTreeMap<String, ImportedMcpServer>,
    conflict_strategy: McpImportConflictStrategy,
) -> Result<ImportLocalMcpResultDto, String> {
    ensure_parent_directory(path)?;
    let mut root = read_toml_root_or_empty(path)?;
    let Some(root_table) = root.as_table_mut() else {
        return Err("Codex config root must be a table.".into());
    };

    let servers_value = root_table
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let Some(existing_servers) = servers_value.as_table_mut() else {
        return Err("Codex config field 'mcp_servers' must be a table.".into());
    };

    let mut imported_names = Vec::new();
    let mut skipped_names = Vec::new();
    for (server_name, server) in servers {
        if existing_servers.contains_key(server_name)
            && conflict_strategy == McpImportConflictStrategy::Skip
        {
            skipped_names.push(server_name.clone());
            continue;
        }

        let mut table = toml::value::Table::new();
        if server.transport != "stdio" {
            table.insert("type".into(), toml::Value::String(server.transport.clone()));
        }
        if let Some(command) = &server.command {
            table.insert("command".into(), toml::Value::String(command.clone()));
        }
        if !server.args.is_empty() {
            table.insert(
                "args".into(),
                toml::Value::Array(
                    server
                        .args
                        .iter()
                        .cloned()
                        .map(toml::Value::String)
                        .collect(),
                ),
            );
        }
        if !server.env.is_empty() {
            table.insert(
                "env".into(),
                toml::Value::Table(
                    server
                        .env
                        .iter()
                        .map(|(key, value)| (key.clone(), toml::Value::String(value.clone())))
                        .collect(),
                ),
            );
        }
        if let Some(url) = &server.url {
            table.insert("url".into(), toml::Value::String(url.clone()));
        }
        if !server.headers.is_empty() {
            table.insert(
                "http_headers".into(),
                toml::Value::Table(
                    server
                        .headers
                        .iter()
                        .map(|(key, value)| (key.clone(), toml::Value::String(value.clone())))
                        .collect(),
                ),
            );
        }

        existing_servers.insert(server_name.clone(), toml::Value::Table(table));
        imported_names.push(server_name.clone());
    }

    let content = toml::to_string_pretty(&root).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())?;
    Ok(ImportLocalMcpResultDto {
        config_path: normalize_path(path),
        imported_count: imported_names.len() as u32,
        skipped_count: skipped_names.len() as u32,
        imported_names,
        skipped_names,
    })
}

fn import_opencode_mcp_servers(
    path: &Path,
    servers: &BTreeMap<String, ImportedMcpServer>,
    conflict_strategy: McpImportConflictStrategy,
) -> Result<ImportLocalMcpResultDto, String> {
    ensure_parent_directory(path)?;
    let mut root = read_json_root_or_empty(path)?;
    let Some(root_object) = root.as_object_mut() else {
        return Err("OpenCode config root must be an object.".into());
    };
    let servers_value = root_object
        .entry("mcp")
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(existing_servers) = servers_value.as_object_mut() else {
        return Err("OpenCode config field 'mcp' must be an object.".into());
    };

    let mut imported_names = Vec::new();
    let mut skipped_names = Vec::new();
    for (server_name, server) in servers {
        if existing_servers.contains_key(server_name)
            && conflict_strategy == McpImportConflictStrategy::Skip
        {
            skipped_names.push(server_name.clone());
            continue;
        }

        let mut value = Map::new();
        match server.transport.as_str() {
            "stdio" => {
                value.insert("type".into(), Value::String("local".into()));
                let mut command = Vec::new();
                if let Some(entry) = &server.command {
                    command.push(Value::String(entry.clone()));
                }
                command.extend(server.args.iter().cloned().map(Value::String));
                value.insert("command".into(), Value::Array(command));
                if !server.env.is_empty() {
                    value.insert(
                        "environment".into(),
                        Value::Object(
                            server
                                .env
                                .iter()
                                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                                .collect(),
                        ),
                    );
                }
            }
            "http" | "sse" => {
                value.insert("type".into(), Value::String("remote".into()));
                if let Some(url) = &server.url {
                    value.insert("url".into(), Value::String(url.clone()));
                }
                if !server.headers.is_empty() {
                    value.insert(
                        "headers".into(),
                        Value::Object(
                            server
                                .headers
                                .iter()
                                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                                .collect(),
                        ),
                    );
                }
            }
            _ => {
                return Err(format!(
                    "OpenCode import does not support transport '{}'.",
                    server.transport
                ))
            }
        }

        existing_servers.insert(server_name.clone(), Value::Object(value));
        imported_names.push(server_name.clone());
    }

    write_json_value(path, &root)?;
    Ok(ImportLocalMcpResultDto {
        config_path: normalize_path(path),
        imported_count: imported_names.len() as u32,
        skipped_count: skipped_names.len() as u32,
        imported_names,
        skipped_names,
    })
}

#[tauri::command]
pub fn list_local_mcps(
    scan_targets: Vec<McpScanTargetDto>,
) -> Result<Vec<LocalMcpServerDto>, String> {
    Ok(mcp_discovery_service::list_local_mcps(scan_targets))
}

#[tauri::command]
pub fn open_mcp_config_folder(app: tauri::AppHandle, config_path: String) -> Result<(), String> {
    let path = resolve_path(&config_path);
    if !path.exists() {
        return Err(format!("MCP config path not found: {config_path}"));
    }

    let open_path = if path.is_dir() {
        path
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("MCP config path has no parent directory: {config_path}"))?
    };
    let open_path = open_path.to_string_lossy().to_string();

    app.opener()
        .open_path(&open_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}

#[tauri::command]
pub fn open_mcp_config_file(app: tauri::AppHandle, config_path: String) -> Result<(), String> {
    let path = resolve_path(&config_path);
    if !path.exists() || !path.is_file() {
        return Err(format!("MCP config file not found: {config_path}"));
    }

    let open_path = path.to_string_lossy().to_string();
    app.opener()
        .open_path(&open_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}

fn delete_claude_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;
    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Err("Claude MCP config does not contain mcpServers.".into());
    };
    if servers.remove(server_name).is_none() {
        return Err(format!("Claude MCP server not found: {server_name}"));
    }

    write_json_value(path, &root)
}

fn delete_gemini_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;
    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Err("Gemini MCP config does not contain mcpServers.".into());
    };
    if servers.remove(server_name).is_none() {
        return Err(format!("Gemini MCP server not found: {server_name}"));
    }

    write_json_value(path, &root)
}

fn delete_codex_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut root = toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())?;
    let Some(root_table) = root.as_table_mut() else {
        return Err("Codex config root is not a table.".into());
    };

    let mut removed = false;
    if let Some(servers) = root_table
        .get_mut("mcp_servers")
        .and_then(toml::Value::as_table_mut)
    {
        removed = servers.remove(server_name).is_some() || removed;
        if servers.is_empty() {
            root_table.remove("mcp_servers");
        }
    }

    if let Some(mcp_table) = root_table.get_mut("mcp").and_then(toml::Value::as_table_mut) {
        if let Some(servers) = mcp_table
            .get_mut("servers")
            .and_then(toml::Value::as_table_mut)
        {
            removed = servers.remove(server_name).is_some() || removed;
            if servers.is_empty() {
                mcp_table.remove("servers");
            }
        }
        if mcp_table.is_empty() {
            root_table.remove("mcp");
        }
    }

    if !removed {
        return Err(format!("Codex MCP server not found: {server_name}"));
    }

    let content = toml::to_string_pretty(&root).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn delete_opencode_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut root = json5::from_str::<Value>(&content).map_err(|error| error.to_string())?;
    let Some(mcp) = root.get_mut("mcp").and_then(Value::as_object_mut) else {
        return Err("OpenCode config does not contain mcp.".into());
    };
    if mcp.remove(server_name).is_none() {
        return Err(format!("OpenCode MCP server not found: {server_name}"));
    }

    write_json_value(path, &root)
}

#[tauri::command]
pub fn delete_local_mcp(
    agent_type: String,
    config_path: String,
    server_name: String,
) -> Result<(), String> {
    let path = resolve_path(&config_path);
    if !path.exists() || !path.is_file() {
        return Err(format!("MCP config file not found: {config_path}"));
    }

    match agent_type.as_str() {
        "claude" => delete_claude_mcp_server(&path, &server_name),
        "codex" => delete_codex_mcp_server(&path, &server_name),
        "gemini" => delete_gemini_mcp_server(&path, &server_name),
        "opencode" => delete_opencode_mcp_server(&path, &server_name),
        _ => Err(format!("Unsupported MCP agent type: {agent_type}")),
    }
}

#[tauri::command]
pub fn import_local_mcp_json(
    agent_type: String,
    root_path: String,
    json_payload: String,
    conflict_strategy: String,
) -> Result<ImportLocalMcpResultDto, String> {
    let strategy = resolve_import_conflict_strategy(&conflict_strategy)?;
    let servers = parse_imported_mcp_payload(&json_payload)?;
    let config_path = resolve_mcp_config_path(&agent_type, &root_path)?;

    match agent_type.as_str() {
        "claude" => import_claude_mcp_servers(&config_path, &servers, strategy),
        "codex" => import_codex_mcp_servers(&config_path, &servers, strategy),
        "gemini" => import_gemini_mcp_servers(&config_path, &servers, strategy),
        "opencode" => import_opencode_mcp_servers(&config_path, &servers, strategy),
        _ => Err(format!("Unsupported MCP agent type: {agent_type}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        delete_claude_mcp_server, delete_codex_mcp_server, delete_gemini_mcp_server,
        delete_opencode_mcp_server, import_claude_mcp_servers, import_codex_mcp_servers,
        import_gemini_mcp_servers, import_opencode_mcp_servers, parse_imported_mcp_payload,
        ImportedMcpServer, McpImportConflictStrategy,
    };
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-mcp-delete-{name}-{unique}"))
    }

    fn sample_imported_server() -> ImportedMcpServer {
        ImportedMcpServer {
            transport: "stdio".into(),
            command: Some("npx".into()),
            args: vec!["-y".into(), "@modelcontextprotocol/server-filesystem".into()],
            env: BTreeMap::from([("TOKEN".into(), "secret".into())]),
            url: None,
            headers: BTreeMap::new(),
        }
    }

    #[test]
    fn parse_imported_mcp_payload_accepts_mcp_servers_wrapper() {
        let servers = parse_imported_mcp_payload(
            r#"{
  "mcpServers": {
    "docs": {
      "type": "http",
      "url": "https://example.com/mcp",
      "headers": {
        "Authorization": "secret"
      }
    }
  }
}"#,
        )
        .expect("parse import payload");

        assert_eq!(servers.len(), 1);
        assert_eq!(
            servers.get("docs").map(|server| server.transport.as_str()),
            Some("http")
        );
    }

    #[test]
    fn delete_claude_mcp_server_removes_entry() {
        let root = temp_dir("claude");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join(".claude.json");
        fs::write(
            &config_path,
            r#"{"mcpServers":{"docs":{"url":"https://example.com"},"keep":{"command":"npx"}}}"#,
        )
        .expect("write config");

        delete_claude_mcp_server(&config_path, "docs").expect("delete server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("\"keep\""));
        assert!(!updated.contains("\"docs\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn delete_gemini_mcp_server_removes_entry() {
        let root = temp_dir("gemini");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("settings.json");
        fs::write(
            &config_path,
            r#"{"mcpServers":{"docs":{"url":"https://example.com"},"keep":{"command":"npx"}}}"#,
        )
        .expect("write config");

        delete_gemini_mcp_server(&config_path, "docs").expect("delete server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("\"keep\""));
        assert!(!updated.contains("\"docs\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn delete_codex_mcp_server_removes_supported_formats() {
        let root = temp_dir("codex");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("config.toml");
        fs::write(
            &config_path,
            r#"[mcp_servers.docs]
url = "https://example.com"

[mcp.servers.legacy]
command = "npx"
"#,
        )
        .expect("write config");

        delete_codex_mcp_server(&config_path, "docs").expect("delete docs server");
        delete_codex_mcp_server(&config_path, "legacy").expect("delete legacy server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(!updated.contains("[mcp_servers.docs]"));
        assert!(!updated.contains("[mcp.servers.legacy]"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn delete_opencode_mcp_server_removes_entry() {
        let root = temp_dir("opencode");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("opencode.json");
        fs::write(
            &config_path,
            r#"{
  mcp: {
    docs: { type: "remote", url: "https://example.com" },
    keep: { type: "local", command: ["npx"] },
  },
}"#,
        )
        .expect("write config");

        delete_opencode_mcp_server(&config_path, "docs").expect("delete server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("\"keep\""));
        assert!(!updated.contains("\"docs\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn import_claude_mcp_servers_creates_config_and_respects_skip() {
        let root = temp_dir("import-claude");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join(".claude.json");
        fs::write(&config_path, r#"{"mcpServers":{"keep":{"command":"old"}}}"#)
            .expect("write config");
        let servers = BTreeMap::from([
            ("keep".into(), sample_imported_server()),
            (
                "docs".into(),
                ImportedMcpServer {
                    transport: "http".into(),
                    command: None,
                    args: Vec::new(),
                    env: BTreeMap::new(),
                    url: Some("https://example.com/mcp".into()),
                    headers: BTreeMap::from([("Authorization".into(), "secret".into())]),
                },
            ),
        ]);

        let result =
            import_claude_mcp_servers(&config_path, &servers, McpImportConflictStrategy::Skip)
                .expect("import servers");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.skipped_count, 1);
        assert!(updated.contains("\"docs\""));
        assert!(updated.contains("\"keep\""));
        assert!(updated.contains("\"command\": \"old\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn import_codex_mcp_servers_writes_server_tables() {
        let root = temp_dir("import-codex");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("config.toml");
        let servers = BTreeMap::from([("filesystem".into(), sample_imported_server())]);

        let result =
            import_codex_mcp_servers(&config_path, &servers, McpImportConflictStrategy::Overwrite)
                .expect("import servers");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(result.imported_count, 1);
        assert!(updated.contains("[mcp_servers.filesystem]"));
        assert!(updated.contains("[mcp_servers.filesystem.env]"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn import_gemini_mcp_servers_writes_json_config() {
        let root = temp_dir("import-gemini");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("settings.json");
        let servers = BTreeMap::from([("filesystem".into(), sample_imported_server())]);

        let result = import_gemini_mcp_servers(
            &config_path,
            &servers,
            McpImportConflictStrategy::Overwrite,
        )
        .expect("import servers");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(result.imported_count, 1);
        assert!(updated.contains("\"mcpServers\""));
        assert!(updated.contains("\"filesystem\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn import_opencode_mcp_servers_converts_to_local_and_remote() {
        let root = temp_dir("import-opencode");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("opencode.json");
        let servers = BTreeMap::from([
            ("filesystem".into(), sample_imported_server()),
            (
                "docs".into(),
                ImportedMcpServer {
                    transport: "sse".into(),
                    command: None,
                    args: Vec::new(),
                    env: BTreeMap::new(),
                    url: Some("https://example.com/mcp".into()),
                    headers: BTreeMap::from([("Authorization".into(), "secret".into())]),
                },
            ),
        ]);

        let result = import_opencode_mcp_servers(
            &config_path,
            &servers,
            McpImportConflictStrategy::Overwrite,
        )
        .expect("import servers");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(result.imported_count, 2);
        assert!(updated.contains("\"type\": \"local\""));
        assert!(updated.contains("\"type\": \"remote\""));
        assert!(updated.contains("\"environment\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
