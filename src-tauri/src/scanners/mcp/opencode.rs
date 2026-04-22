//! OpenCode MCP配置处理实现
use std::path::Path;
use std::collections::BTreeMap;
use serde_json::{Value, Map};
use crate::dto::mcp::{McpScanTargetDto, LocalMcpServerDto, EditableLocalMcpDto, ImportedMcpServer, ImportLocalMcpResultDto, UpdateLocalMcpResultDto};
use crate::constants::*;
use crate::infrastructure::utils::path::{atomic_write, normalize_path, user_home_dir};
use crate::infrastructure::utils::fs::ensure_parent_dir;
use super::{McpConfigHandler, common::*};
use crate::dto::mcp::McpImportConflictStrategy;

pub struct OpenCodeHandler;

impl McpConfigHandler for OpenCodeHandler {
    fn scan_servers(&self, target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
        let config_path = OPENCODE_CONFIG_PATH
            .iter()
            .fold(user_home_dir(), |path, segment| path.join(segment));
        if !config_path.exists() || !config_path.is_file() {
            return Vec::new();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(_) => return Vec::new(),
        };
        let value = match json5::from_str::<Value>(&contents) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        let Some(servers) = value.get("mcp").and_then(Value::as_object) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        for (server_name, server_value) in servers {
            let Some(server) = server_value.as_object() else {
                continue;
            };
            let open_code_type = server
                .get(FIELD_TYPE)
                .and_then(Value::as_str)
                .unwrap_or("local");
            let mut normalized_server = Map::new();
            let (transport, endpoint) = match open_code_type {
                "local" => {
                    normalized_server.insert(FIELD_TYPE.into(), Value::String(TRANSPORT_STDIO.into()));
                    let command_parts = server.get(FIELD_COMMAND).and_then(Value::as_array);
                    let command = command_parts
                        .and_then(|parts| parts.first())
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if !command.is_empty() {
                        normalized_server.insert(FIELD_COMMAND.into(), Value::String(command.clone()));
                    }
                    if let Some(args) = command_parts {
                        let normalized_args = args
                            .iter()
                            .skip(1)
                            .filter_map(Value::as_str)
                            .map(|value| Value::String(value.to_string()))
                            .collect::<Vec<_>>();
                        if !normalized_args.is_empty() {
                            normalized_server.insert(FIELD_ARGS.into(), Value::Array(normalized_args));
                        }
                    }
                    if let Some(environment) = server.get("environment").and_then(Value::as_object) {
                        normalized_server.insert(FIELD_ENV.into(), Value::Object(environment.clone()));
                    }
                    (TRANSPORT_STDIO.to_string(), command)
                }
                "remote" => {
                    normalized_server.insert(FIELD_TYPE.into(), Value::String(TRANSPORT_SSE.into()));
                    if let Some(url) = server.get(FIELD_URL).and_then(Value::as_str) {
                        normalized_server.insert(FIELD_URL.into(), Value::String(url.to_string()));
                    }
                    if let Some(headers) = server.get(FIELD_HEADERS).and_then(Value::as_object) {
                        normalized_server.insert(FIELD_HEADERS.into(), Value::Object(headers.clone()));
                    }
                    (
                        TRANSPORT_SSE.to_string(),
                        server
                            .get(FIELD_URL)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    )
                }
                _ => continue,
            };

            items.push(build_local_mcp(
                target,
                server_name,
                "user",
                &config_path,
                None,
                transport,
                endpoint,
                masked_json_config(&normalized_server),
                Vec::new(),
                Vec::new(),
            ));
        }

        items
    }

    fn read_server(&self, config_path: &Path, server_name: &str, _scope: &str, _project_path: Option<&str>) -> Result<EditableLocalMcpDto, String> {
        let content = std::fs::read_to_string(config_path).map_err(|error| error.to_string())?;
        let root = json5::from_str::<Value>(&content).map_err(|error| error.to_string())?;
        let servers = root
            .get("mcp")
            .and_then(Value::as_object)
            .ok_or_else(|| "OpenCode config does not contain mcp.".to_string())?;

        let server = servers
            .get(server_name)
            .ok_or_else(|| format!("OpenCode MCP server not found: {server_name}"))?;
        let Some(server_obj) = server.as_object() else {
            return Err(format!(
                "OpenCode MCP server '{server_name}' must be an object."
            ));
        };

        // 解析为EditableLocalMcpDto
        let transport = match server_obj.get(FIELD_TYPE).and_then(Value::as_str) {
            Some("local") => TRANSPORT_STDIO.to_string(),
            Some("remote") => TRANSPORT_SSE.to_string(),
            Some(other) => return Err(format!("Unsupported OpenCode MCP type: {other}")),
            None => TRANSPORT_STDIO.to_string(),
        };
        let command_parts = server_obj
            .get(FIELD_COMMAND)
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
        let env = server_obj
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
        let headers = server_obj
            .get(FIELD_HEADERS)
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
            url: server_obj
                .get(FIELD_URL)
                .and_then(Value::as_str)
                .map(str::to_string),
            headers,
        })
    }

    fn write_server(&self, config_path: &Path, old_name: &str, new_name: &str, server: &ImportedMcpServer, _scope: &str, _project_path: Option<&str>) -> Result<UpdateLocalMcpResultDto, String> {
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
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

        if old_name != new_name && servers.contains_key(new_name) {
            return Err(format!(
                "OpenCode MCP server already exists: {new_name}"
            ));
        }
        if servers.remove(old_name).is_none() {
            return Err(format!("OpenCode MCP server not found: {old_name}"));
        }

        // Build OpenCode format config
        let value = build_opencode_server_value(server)?;

        servers.insert(new_name.to_string(), Value::Object(value));
        write_json_value(config_path, &root)?;

        Ok(UpdateLocalMcpResultDto {
            config_path: normalize_path(config_path),
            server_name: new_name.to_string(),
        })
    }

    fn delete_server(&self, config_path: &Path, server_name: &str, _scope: &str, _project_path: Option<&str>) -> Result<(), String> {
        // Original delete_opencode_mcp_server logic
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
        let mut root = read_json5_root_or_empty(config_path)?;
        let Some(mcp) = root.get_mut("mcp").and_then(Value::as_object_mut) else {
            return Err("OpenCode config does not contain mcp.".into());
        };
        if mcp.remove(server_name).is_none() {
            return Err(format!("OpenCode MCP server not found: {server_name}"));
        }

        // Clean up empty mcp object
        if mcp.is_empty() {
            if let Some(root_object) = root.as_object_mut() {
                root_object.remove("mcp");
            }
        }

        write_json_value(config_path, &root)
    }

    fn import_servers(&self, config_path: &Path, servers: &BTreeMap<String, ImportedMcpServer>, conflict_strategy: McpImportConflictStrategy) -> Result<ImportLocalMcpResultDto, String> {
        // 原有import_opencode_mcp_servers逻辑
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
        let mut root = read_json5_root_or_empty(config_path)?;
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

            let value = build_opencode_server_value(server)?;

            existing_servers.insert(server_name.clone(), Value::Object(value));
            imported_names.push(server_name.clone());
        }

        write_json_value(config_path, &root)?;
        Ok(ImportLocalMcpResultDto {
            config_path: normalize_path(config_path),
            imported_count: imported_names.len() as u32,
            skipped_count: skipped_names.len() as u32,
            imported_names,
            skipped_names,
        })
    }
}

// 辅助工具：将 ImportedMcpServer 转换为 OpenCode 格式的 JSON Value
fn build_opencode_server_value(server: &ImportedMcpServer) -> Result<Map<String, Value>, String> {
    let mut value = Map::new();
    match server.transport.as_str() {
        TRANSPORT_STDIO => {
            value.insert(FIELD_TYPE.into(), Value::String("local".into()));
            let mut command = Vec::new();
            if let Some(entry) = &server.command {
                command.push(Value::String(entry.clone()));
            }
            command.extend(server.args.iter().cloned().map(Value::String));
            value.insert(FIELD_COMMAND.into(), Value::Array(command));
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
        TRANSPORT_SSE | TRANSPORT_HTTP => {
            value.insert(FIELD_TYPE.into(), Value::String("remote".into()));
            if let Some(url) = &server.url {
                value.insert(FIELD_URL.into(), Value::String(url.clone()));
            }
            if !server.headers.is_empty() {
                value.insert(
                    FIELD_HEADERS.into(),
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
                "OpenCode does not support transport '{other}'."
            ))
        }
    }
    Ok(value)
}

// 辅助工具：读取JSON5配置
fn read_json5_root_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    json5::from_str::<Value>(&content).map_err(|error| error.to_string())
}

// 辅助工具：写入JSON配置（JSON5兼容标准JSON，原子写入）
fn write_json_value(path: &Path, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    atomic_write(path, &content)
}
