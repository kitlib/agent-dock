use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::sync::Mutex;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

use crate::dto::mcp::{EditableLocalMcpDto, LocalMcpServerDto, McpScanTargetDto};
use crate::services::mcp_discovery_service;

// Global store for running MCP Inspector processes
static INSPECTOR_PROCESSES: LazyLock<Mutex<HashMap<u32, CommandChild>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalMcpResultDto {
    pub config_path: String,
    pub imported_count: u32,
    pub skipped_count: u32,
    pub imported_names: Vec<String>,
    pub skipped_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalMcpResultDto {
    pub config_path: String,
    pub server_name: String,
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
    path.strip_prefix("~/")
        .or_else(|| path.strip_prefix("~\\"))
        .map(|rel| user_home_dir().join(rel))
        .unwrap_or_else(|| {
            let p = PathBuf::from(path);
            if p.is_absolute() { p } else { user_home_dir().join(p) }
        })
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


/// Helper to read JSON MCP config and get server by name
fn read_json_mcp_server(
    path: &Path,
    server_name: &str,
    agent_type: &str,
    get_servers: impl Fn(&Value) -> Result<&Map<String, Value>, String>,
) -> Result<EditableLocalMcpDto, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;
    let servers = get_servers(&root)?;
    let server = servers
        .get(server_name)
        .ok_or_else(|| format!("{agent_type} MCP server not found: {server_name}"))?;
    normalize_json_server(server_name, server)
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

fn read_json5_root_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }

    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    json5::from_str::<Value>(&content).map_err(|error| error.to_string())
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

fn parse_string_map(
    field: &str,
    value: Option<&Value>,
) -> Result<BTreeMap<String, String>, String> {
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
    [
        "type", "command", "args", "env", "url", "httpUrl", "headers",
    ]
    .into_iter()
    .collect()
}

fn parse_imported_mcp_server(
    server_name: &str,
    value: &Value,
) -> Result<ImportedMcpServer, String> {
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
        other => {
            return Err(format!(
                "MCP server '{server_name}' has unsupported type '{other}'."
            ))
        }
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

fn parse_imported_mcp_payload(
    payload: &str,
) -> Result<BTreeMap<String, ImportedMcpServer>, String> {
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
        if object
            .keys()
            .any(|key| reserved_fields.contains(key.as_str()))
        {
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
            parse_imported_mcp_server(server_name, value)
                .map(|server| (server_name.clone(), server))
        })
        .collect()
}

fn build_json_server_value(server: &ImportedMcpServer, use_http_url: bool) -> Value {
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
        let url_field = if use_http_url && server.transport == "http" {
            "httpUrl"
        } else {
            "url"
        };
        value.insert(url_field.into(), Value::String(url.clone()));
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

    Value::Object(value)
}

fn build_codex_server_value(server: &ImportedMcpServer) -> toml::Value {
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

    toml::Value::Table(table)
}

fn build_opencode_server_value(server: &ImportedMcpServer) -> Result<Value, String> {
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
        "sse" => {
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
        other => {
            return Err(format!(
                "OpenCode edit does not support transport '{other}'."
            ))
        }
    }

    Ok(Value::Object(value))
}

fn find_claude_servers<'a>(
    root: &'a Value,
    scope: &str,
    project_path: Option<&str>,
) -> Result<&'a Map<String, Value>, String> {
    match scope {
        "user" => root
            .get("mcpServers")
            .and_then(Value::as_object)
            .ok_or_else(|| "Claude MCP config does not contain user mcpServers.".into()),
        "local" => {
            let Some(project_path) = project_path else {
                return Err("Claude local MCP edit requires projectPath.".into());
            };
            root.get("projects")
                .and_then(Value::as_object)
                .and_then(|projects| projects.get(project_path))
                .and_then(Value::as_object)
                .and_then(|project| project.get("mcpServers"))
                .and_then(Value::as_object)
                .ok_or_else(|| format!("Claude local MCP project not found: {project_path}"))
        }
        _ => Err(format!("Unsupported Claude MCP scope: {scope}")),
    }
}

fn find_claude_servers_mut<'a>(
    root: &'a mut Value,
    scope: &str,
    project_path: Option<&str>,
) -> Result<&'a mut Map<String, Value>, String> {
    match scope {
        "user" => {
            let Some(root_object) = root.as_object_mut() else {
                return Err("Claude MCP config root must be an object.".into());
            };
            let servers = root_object
                .entry("mcpServers")
                .or_insert_with(|| Value::Object(Map::new()));
            servers
                .as_object_mut()
                .ok_or_else(|| "Claude MCP config field 'mcpServers' must be an object.".into())
        }
        "local" => {
            let Some(project_path) = project_path else {
                return Err("Claude local MCP edit requires projectPath.".into());
            };
            let Some(root_object) = root.as_object_mut() else {
                return Err("Claude MCP config root must be an object.".into());
            };
            let projects = root_object
                .entry("projects")
                .or_insert_with(|| Value::Object(Map::new()));
            let Some(projects_object) = projects.as_object_mut() else {
                return Err("Claude MCP config field 'projects' must be an object.".into());
            };
            let project = projects_object
                .entry(project_path.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            let Some(project_object) = project.as_object_mut() else {
                return Err("Claude MCP project entry must be an object.".into());
            };
            let servers = project_object
                .entry("mcpServers")
                .or_insert_with(|| Value::Object(Map::new()));
            servers.as_object_mut().ok_or_else(|| {
                "Claude local MCP config field 'mcpServers' must be an object.".into()
            })
        }
        _ => Err(format!("Unsupported Claude MCP scope: {scope}")),
    }
}

fn normalize_json_server(server_name: &str, value: &Value) -> Result<EditableLocalMcpDto, String> {
    let imported = parse_imported_mcp_server(server_name, value)?;
    Ok(EditableLocalMcpDto {
        server_name: server_name.to_string(),
        transport: imported.transport,
        command: imported.command,
        args: imported.args,
        env: imported.env,
        url: imported.url,
        headers: imported.headers,
    })
}

fn normalize_codex_server(
    server_name: &str,
    value: &toml::Value,
) -> Result<EditableLocalMcpDto, String> {
    let Some(table) = value.as_table() else {
        return Err(format!("Codex MCP server '{server_name}' is not a table."));
    };

    let transport = match table.get("type").and_then(toml::Value::as_str) {
        Some("http") => "http".to_string(),
        Some("sse") => "sse".to_string(),
        Some(other) => other.to_string(),
        None => {
            if table.get("command").and_then(toml::Value::as_str).is_some() {
                "stdio".to_string()
            } else if table.get("url").and_then(toml::Value::as_str).is_some() {
                "http".to_string()
            } else {
                "stdio".to_string()
            }
        }
    };
    let args = table
        .get("args")
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let env = table
        .get("env")
        .and_then(toml::Value::as_table)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|entry| (key.clone(), entry.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let headers = table
        .get("http_headers")
        .or_else(|| table.get("headers"))
        .and_then(toml::Value::as_table)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|entry| (key.clone(), entry.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    Ok(EditableLocalMcpDto {
        server_name: server_name.to_string(),
        transport,
        command: table
            .get("command")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        args,
        env,
        url: table
            .get("url")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        headers,
    })
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
    let mut root = read_json5_root_or_empty(path)?;
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

fn get_local_mcp_edit_data_impl(
    agent_type: &str,
    config_path: &Path,
    server_name: &str,
    scope: &str,
    project_path: Option<&str>,
) -> Result<EditableLocalMcpDto, String> {
    match agent_type {
        "claude" => read_json_mcp_server(config_path, server_name, "Claude", |root| {
            find_claude_servers(root, scope, project_path)
        }),
        "gemini" => read_json_mcp_server(config_path, server_name, "Gemini", |root| {
            root.get("mcpServers")
                .and_then(Value::as_object)
                .ok_or_else(|| "Gemini MCP config does not contain mcpServers.".to_string())
        }),
        "codex" => {
            let content = fs::read_to_string(config_path).map_err(|error| error.to_string())?;
            let root =
                toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())?;
            if let Some(server) = root
                .get("mcp_servers")
                .and_then(toml::Value::as_table)
                .and_then(|servers| servers.get(server_name))
            {
                return normalize_codex_server(server_name, server);
            }
            if let Some(server) = root
                .get("mcp")
                .and_then(toml::Value::as_table)
                .and_then(|mcp| mcp.get("servers"))
                .and_then(toml::Value::as_table)
                .and_then(|servers| servers.get(server_name))
            {
                return normalize_codex_server(server_name, server);
            }
            Err(format!("Codex MCP server not found: {server_name}"))
        }
        "opencode" => {
            let content = fs::read_to_string(config_path).map_err(|error| error.to_string())?;
            let root = json5::from_str::<Value>(&content).map_err(|error| error.to_string())?;
            let servers = root
                .get("mcp")
                .and_then(Value::as_object)
                .ok_or_else(|| "OpenCode config does not contain mcp.".to_string())?;
            let server = servers
                .get(server_name)
                .ok_or_else(|| format!("OpenCode MCP server not found: {server_name}"))?;
            let Some(server_object) = server.as_object() else {
                return Err(format!(
                    "OpenCode MCP server '{server_name}' must be an object."
                ));
            };
            let transport = match server_object.get("type").and_then(Value::as_str) {
                Some("local") => "stdio".to_string(),
                Some("remote") => "sse".to_string(),
                Some(other) => return Err(format!("Unsupported OpenCode MCP type: {other}")),
                None => "stdio".to_string(),
            };
            let command_parts = server_object
                .get("command")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let command = command_parts
                .first()
                .and_then(Value::as_str)
                .map(str::to_string);
            let args = command_parts
                .iter()
                .skip(1)
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>();
            let env = server_object
                .get("environment")
                .and_then(Value::as_object)
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|(key, value)| {
                            value.as_str().map(|entry| (key.clone(), entry.to_string()))
                        })
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default();
            let headers = server_object
                .get("headers")
                .and_then(Value::as_object)
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|(key, value)| {
                            value.as_str().map(|entry| (key.clone(), entry.to_string()))
                        })
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default();

            Ok(EditableLocalMcpDto {
                server_name: server_name.to_string(),
                transport,
                command,
                args,
                env,
                url: server_object
                    .get("url")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                headers,
            })
        }
        _ => Err(format!("Unsupported MCP agent type: {agent_type}")),
    }
}

fn update_local_mcp_impl(
    agent_type: &str,
    config_path: &Path,
    server_name: &str,
    scope: &str,
    project_path: Option<&str>,
    next_server_name: &str,
    next_server: &ImportedMcpServer,
) -> Result<UpdateLocalMcpResultDto, String> {
    match agent_type {
        "claude" => {
            ensure_parent_directory(config_path)?;
            let mut root = read_json_root_or_empty(config_path)?;
            let servers = find_claude_servers_mut(&mut root, scope, project_path)?;
            if server_name != next_server_name && servers.contains_key(next_server_name) {
                return Err(format!(
                    "Claude MCP server already exists: {next_server_name}"
                ));
            }
            if servers.remove(server_name).is_none() {
                return Err(format!("Claude MCP server not found: {server_name}"));
            }
            servers.insert(
                next_server_name.to_string(),
                build_json_server_value(next_server, false),
            );
            write_json_value(config_path, &root)?;
        }
        "gemini" => {
            ensure_parent_directory(config_path)?;
            let mut root = read_json_root_or_empty(config_path)?;
            let Some(root_object) = root.as_object_mut() else {
                return Err("Gemini MCP config root must be an object.".into());
            };
            let servers = root_object
                .entry("mcpServers")
                .or_insert_with(|| Value::Object(Map::new()));
            let Some(servers) = servers.as_object_mut() else {
                return Err("Gemini MCP config field 'mcpServers' must be an object.".into());
            };
            if server_name != next_server_name && servers.contains_key(next_server_name) {
                return Err(format!(
                    "Gemini MCP server already exists: {next_server_name}"
                ));
            }
            if servers.remove(server_name).is_none() {
                return Err(format!("Gemini MCP server not found: {server_name}"));
            }
            servers.insert(
                next_server_name.to_string(),
                build_json_server_value(next_server, true),
            );
            write_json_value(config_path, &root)?;
        }
        "codex" => {
            ensure_parent_directory(config_path)?;
            let mut root = read_toml_root_or_empty(config_path)?;
            let Some(root_table) = root.as_table_mut() else {
                return Err("Codex config root must be a table.".into());
            };
            let has_conflict = server_name != next_server_name
                && (root_table
                    .get("mcp_servers")
                    .and_then(toml::Value::as_table)
                    .is_some_and(|servers| servers.contains_key(next_server_name))
                    || root_table
                        .get("mcp")
                        .and_then(toml::Value::as_table)
                        .and_then(|mcp| mcp.get("servers"))
                        .and_then(toml::Value::as_table)
                        .is_some_and(|servers| servers.contains_key(next_server_name)));
            if has_conflict {
                return Err(format!(
                    "Codex MCP server already exists: {next_server_name}"
                ));
            }
            if let Some(servers) = root_table
                .get_mut("mcp_servers")
                .and_then(toml::Value::as_table_mut)
            {
                servers.remove(server_name);
                if servers.is_empty() {
                    root_table.remove("mcp_servers");
                }
            }
            if let Some(mcp_table) = root_table
                .get_mut("mcp")
                .and_then(toml::Value::as_table_mut)
            {
                if let Some(servers) = mcp_table
                    .get_mut("servers")
                    .and_then(toml::Value::as_table_mut)
                {
                    servers.remove(server_name);
                    if servers.is_empty() {
                        mcp_table.remove("servers");
                    }
                }
                if mcp_table.is_empty() {
                    root_table.remove("mcp");
                }
            }
            let servers_value = root_table
                .entry("mcp_servers")
                .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
            let Some(servers) = servers_value.as_table_mut() else {
                return Err("Codex config field 'mcp_servers' must be a table.".into());
            };
            if server_name != next_server_name && servers.contains_key(next_server_name) {
                return Err(format!(
                    "Codex MCP server already exists: {next_server_name}"
                ));
            }
            servers.insert(
                next_server_name.to_string(),
                build_codex_server_value(next_server),
            );
            let content = toml::to_string_pretty(&root).map_err(|error| error.to_string())?;
            fs::write(config_path, content).map_err(|error| error.to_string())?;
        }
        "opencode" => {
            ensure_parent_directory(config_path)?;
            let mut root = read_json5_root_or_empty(config_path)?;
            let Some(root_object) = root.as_object_mut() else {
                return Err("OpenCode config root must be an object.".into());
            };
            let servers = root_object
                .entry("mcp")
                .or_insert_with(|| Value::Object(Map::new()));
            let Some(servers) = servers.as_object_mut() else {
                return Err("OpenCode config field 'mcp' must be an object.".into());
            };
            if server_name != next_server_name && servers.contains_key(next_server_name) {
                return Err(format!(
                    "OpenCode MCP server already exists: {next_server_name}"
                ));
            }
            if servers.remove(server_name).is_none() {
                return Err(format!("OpenCode MCP server not found: {server_name}"));
            }
            servers.insert(
                next_server_name.to_string(),
                build_opencode_server_value(next_server)?,
            );
            write_json_value(config_path, &root)?;
        }
        _ => return Err(format!("Unsupported MCP agent type: {agent_type}")),
    }

    Ok(UpdateLocalMcpResultDto {
        config_path: normalize_path(config_path),
        server_name: next_server_name.to_string(),
    })
}

fn validate_editable_mcp_server(
    server_name: &str,
    server: &ImportedMcpServer,
) -> Result<(), String> {
    if server_name.trim().is_empty() {
        return Err("MCP server name is required.".into());
    }

    match server.transport.as_str() {
        "stdio" => {
            if server.command.as_deref().unwrap_or("").trim().is_empty() {
                return Err("stdio MCP servers require a command.".into());
            }
        }
        "http" | "sse" => {
            if server.url.as_deref().unwrap_or("").trim().is_empty() {
                return Err(format!("{} MCP servers require a URL.", server.transport));
            }
        }
        other => {
            return Err(format!("Unsupported MCP transport: {other}"));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn list_local_mcps(
    scan_targets: Vec<McpScanTargetDto>,
) -> Result<Vec<LocalMcpServerDto>, String> {
    println!(
        "[MCP] Scan requested for {} targets: {:?}",
        scan_targets.len(),
        scan_targets
            .iter()
            .map(|t| format!("agent_id={}, root_path={}", t.agent_id, t.root_path))
            .collect::<Vec<_>>()
    );

    let result = mcp_discovery_service::list_local_mcps(scan_targets);

    println!("[MCP] Scan completed, found {} servers", result.len());
    for server in &result {
        println!(
            "[MCP] Found server: id={}, name={}, transport={}, config_path={}",
            server.id,
            server.name,
            server.transport,
            server.config_path
        );
    }

    Ok(result)
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

#[tauri::command]
pub fn get_local_mcp_edit_data(
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<EditableLocalMcpDto, String> {
    let path = resolve_path(&config_path);
    if !path.exists() || !path.is_file() {
        return Err(format!("MCP config file not found: {config_path}"));
    }

    get_local_mcp_edit_data_impl(
        &agent_type,
        &path,
        &server_name,
        &scope,
        project_path.as_deref(),
    )
}

#[tauri::command]
pub fn update_local_mcp(
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
    next_server_name: String,
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    url: Option<String>,
    headers: BTreeMap<String, String>,
) -> Result<UpdateLocalMcpResultDto, String> {
    let path = resolve_path(&config_path);
    let next_server_name = next_server_name.trim().to_string();
    let next_server = ImportedMcpServer {
        transport,
        command: command
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        args: args
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
        env: env
            .into_iter()
            .map(|(key, value)| (key.trim().to_string(), value))
            .filter(|(key, _)| !key.is_empty())
            .collect(),
        url: url
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        headers: headers
            .into_iter()
            .map(|(key, value)| (key.trim().to_string(), value))
            .filter(|(key, _)| !key.is_empty())
            .collect(),
    };
    validate_editable_mcp_server(&next_server_name, &next_server)?;

    update_local_mcp_impl(
        &agent_type,
        &path,
        &server_name,
        &scope,
        project_path.as_deref(),
        &next_server_name,
        &next_server,
    )
}

/// Helper to delete MCP server from JSON config with mcpServers root
fn delete_json_mcp_server(path: &Path, server_name: &str, agent_type: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;
    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Err(format!(
            "{agent_type} MCP config does not contain mcpServers."
        ));
    };
    if servers.remove(server_name).is_none() {
        return Err(format!("{agent_type} MCP server not found: {server_name}"));
    }

    write_json_value(path, &root)
}

fn delete_claude_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    delete_json_mcp_server(path, server_name, "Claude")
}

fn delete_gemini_mcp_server(path: &Path, server_name: &str) -> Result<(), String> {
    delete_json_mcp_server(path, server_name, "Gemini")
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

    if let Some(mcp_table) = root_table
        .get_mut("mcp")
        .and_then(toml::Value::as_table_mut)
    {
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
        "claude" | "gemini" => import_claude_mcp_servers(&config_path, &servers, strategy),
        "codex" => import_codex_mcp_servers(&config_path, &servers, strategy),
        "opencode" => import_opencode_mcp_servers(&config_path, &servers, strategy),
        _ => Err(format!("Unsupported MCP agent type: {agent_type}")),
    }
}

#[tauri::command]
pub async fn launch_mcp_inspector(
    app: tauri::AppHandle,
    config: EditableLocalMcpDto,
    server_name: Option<String>,
) -> Result<u32, String> {
    // Check if npx is available (comes with Node.js) - try .cmd suffix first on Windows
    let npx_cmd = if cfg!(windows) {
        // On Windows, test npx.cmd first, fall back to npx
        match app.shell().command("npx.cmd").args(["--version"]).output().await {
            Ok(output) if output.status.success() => "npx.cmd",
            _ => "npx",
        }
    } else {
        "npx"
    };

    let check_npx = app.shell().command(npx_cmd)
        .args(["--version"])
        .output()
        .await
        .map_err(|e| format!("Failed to find npx command: {}", e))?;

    if !check_npx.status.success() {
        return Err(
            "Node.js is not installed. Please install Node.js first:\n\nhttps://nodejs.org/"
                .to_string(),
        );
    }

    // Build inspector arguments - use npx to run directly without global installation
    let mut args = vec!["@modelcontextprotocol/inspector@latest".to_string()];

    match config.transport.as_str() {
        "stdio" => {
            if let Some(command) = &config.command {
                let mut cmd_parts = vec![command.clone()];
                cmd_parts.extend(config.args.clone());
                args.push("--cmd".to_string());
                args.push(cmd_parts.join(" "));
            } else {
                return Err("stdio transport requires a command".to_string());
            }
        }
        "http" | "sse" => {
            if let Some(url) = &config.url {
                args.push("--url".to_string());
                args.push(url.clone());
            } else {
                return Err(format!("{} transport requires a url", config.transport).to_string());
            }
        }
        _ => {
            return Err(format!("Unsupported transport type: {}", config.transport));
        }
    }

    // Add environment variables
    for (key, value) in &config.env {
        args.push("--env".to_string());
        args.push(format!("{}={}", key, value));
    }

    // Add headers
    for (key, value) in &config.headers {
        args.push("--header".to_string());
        args.push(format!("{}={}", key, value));
    }

    // Add server name if provided
    if let Some(name) = server_name {
        args.push("--name".to_string());
        args.push(name);
    }

    // Build npx command with Tauri shell plugin
    let cmd = app.shell().command(npx_cmd)
        .args(args);

    // Spawn process in background
    let (mut rx, child) = cmd
        .spawn()
        .map_err(|e| format!("Failed to launch MCP Inspector: {}", e))?;

    let pid = child.pid();

    // 后台监听进程输出，打印所有stdout/stderr日志到控制台
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(data) => {
                    if let Ok(output) = String::from_utf8(data) {
                        println!("[MCP Inspector] stdout: {}", output.trim());
                    }
                }
                CommandEvent::Stderr(data) => {
                    if let Ok(output) = String::from_utf8(data) {
                        eprintln!("[MCP Inspector] stderr: {}", output.trim());
                    }
                }
                _ => {} // 忽略其他事件
            }
        }
    });

    // Store process handle in global map for lifecycle management
    INSPECTOR_PROCESSES.lock().await.insert(pid, child);

    Ok(pid)
}

#[tauri::command]
pub async fn stop_mcp_inspector(
    _app: tauri::AppHandle,
    pid: u32
) -> Result<(), String> {
    let mut processes = INSPECTOR_PROCESSES.lock().await;

    // Remove process from map and kill it using Tauri official API
    if let Some(child) = processes.remove(&pid) {
        child.kill().map_err(|e| format!("Failed to stop MCP Inspector: {}", e))?;
    } else {
        return Err(format!("MCP Inspector process {} not found", pid));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        delete_claude_mcp_server, delete_codex_mcp_server, delete_gemini_mcp_server,
        delete_opencode_mcp_server, get_local_mcp_edit_data_impl, import_claude_mcp_servers,
        import_codex_mcp_servers, import_opencode_mcp_servers, parse_imported_mcp_payload,
        update_local_mcp_impl, ImportedMcpServer, McpImportConflictStrategy,
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
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-filesystem".into(),
            ],
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

        let result =
            import_gemini_mcp_servers(&config_path, &servers, McpImportConflictStrategy::Overwrite)
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

    #[test]
    fn get_local_mcp_edit_data_reads_claude_local_scope() {
        let root = temp_dir("edit-claude-local");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join(".claude.json");
        fs::write(
            &config_path,
            r#"{
  "projects": {
    "D:/Workspace/demo": {
      "mcpServers": {
        "docs": {
          "type": "stdio",
          "command": "npx",
          "args": ["-y", "@modelcontextprotocol/server-filesystem"],
          "env": {
            "TOKEN": "secret"
          }
        }
      }
    }
  }
}"#,
        )
        .expect("write config");

        let result = get_local_mcp_edit_data_impl(
            "claude",
            &config_path,
            "docs",
            "local",
            Some("D:/Workspace/demo"),
        )
        .expect("load edit data");

        assert_eq!(result.server_name, "docs");
        assert_eq!(result.transport, "stdio");
        assert_eq!(result.command.as_deref(), Some("npx"));
        assert_eq!(result.args.len(), 2);
        assert_eq!(result.env.get("TOKEN").map(String::as_str), Some("secret"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn update_local_mcp_updates_codex_and_removes_legacy_entry() {
        let root = temp_dir("edit-codex");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("config.toml");
        fs::write(
            &config_path,
            r#"[mcp_servers.docs]
url = "https://example.com/docs"

[mcp.servers.docs]
command = "npx"
"#,
        )
        .expect("write config");

        let updated_server = ImportedMcpServer {
            transport: "stdio".into(),
            command: Some("node".into()),
            args: vec!["server.js".into()],
            env: BTreeMap::new(),
            url: None,
            headers: BTreeMap::new(),
        };

        update_local_mcp_impl(
            "codex",
            &config_path,
            "docs",
            "user",
            None,
            "filesystem",
            &updated_server,
        )
        .expect("update codex server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("[mcp_servers.filesystem]"));
        assert!(!updated.contains("[mcp_servers.docs]"));
        assert!(!updated.contains("[mcp.servers.docs]"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn update_local_mcp_rejects_codex_name_conflict_before_removal() {
        let root = temp_dir("edit-codex-conflict");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("config.toml");
        fs::write(
            &config_path,
            r#"[mcp_servers.docs]
command = "npx"

[mcp_servers.keep]
url = "https://example.com/keep"
"#,
        )
        .expect("write config");

        let error = update_local_mcp_impl(
            "codex",
            &config_path,
            "docs",
            "user",
            None,
            "keep",
            &sample_imported_server(),
        )
        .expect_err("should reject conflict");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(error.contains("already exists"));
        assert!(updated.contains("[mcp_servers.docs]"));
        assert!(updated.contains("[mcp_servers.keep]"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn update_local_mcp_updates_opencode_remote_server() {
        let root = temp_dir("edit-opencode");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join("opencode.json");
        fs::write(
            &config_path,
            r#"{
  mcp: {
    docs: { type: "remote", url: "https://example.com/docs" },
  },
}"#,
        )
        .expect("write config");

        let updated_server = ImportedMcpServer {
            transport: "sse".into(),
            command: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            url: Some("https://example.com/next".into()),
            headers: BTreeMap::from([("Authorization".into(), "secret".into())]),
        };

        update_local_mcp_impl(
            "opencode",
            &config_path,
            "docs",
            "user",
            None,
            "docs",
            &updated_server,
        )
        .expect("update opencode server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("\"type\": \"remote\""));
        assert!(updated.contains("\"https://example.com/next\""));
        assert!(updated.contains("\"headers\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
