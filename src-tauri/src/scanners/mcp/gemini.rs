//! Gemini MCP配置处理实现
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value};

use crate::constants::*;
use crate::dto::mcp::{
    EditableLocalMcpDto, ImportLocalMcpResultDto, ImportedMcpServer, LocalMcpServerDto,
    McpImportConflictStrategy, McpScanTargetDto, UpdateLocalMcpResultDto,
};
use crate::infrastructure::utils::fs::ensure_parent_dir;
use crate::infrastructure::utils::path::{atomic_write, normalize_path, resolve_agent_root};

use super::common::*;
use super::McpConfigHandler;

pub struct GeminiHandler;

impl McpConfigHandler for GeminiHandler {
    fn scan_servers(&self, target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
        let root_path = resolve_agent_root(&target.root_path);
        let config_path = root_path.join(GEMINI_CONFIG_FILE);
        if !config_path.exists() || !config_path.is_file() {
            return Vec::new();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(_) => return Vec::new(),
        };
        let value = match serde_json::from_str::<Value>(&contents) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        let Some(servers) = value.get(FIELD_MCP_SERVERS).and_then(Value::as_object) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        for (server_name, server_value) in servers {
            let Some(server) = server_value.as_object() else {
                continue;
            };

            let explicit_type = server.get(FIELD_TYPE).and_then(Value::as_str);
            let command = server.get(FIELD_COMMAND).and_then(Value::as_str);
            let http_url = server.get(FIELD_HTTP_URL).and_then(Value::as_str);
            let url = server.get(FIELD_URL).and_then(Value::as_str);
            let transport = if http_url.is_some() {
                TRANSPORT_HTTP.to_string()
            } else if explicit_type.is_some() {
                transport_from_config(explicit_type, command, url)
            } else if command.is_some() {
                TRANSPORT_STDIO.to_string()
            } else if url.is_some() {
                TRANSPORT_SSE.to_string()
            } else {
                TRANSPORT_UNKNOWN.to_string()
            };
            let endpoint = http_url
                .map(str::to_string)
                .or_else(|| url.map(str::to_string))
                .or_else(|| command.map(str::to_string))
                .unwrap_or_default();

            // 标准化配置，把http_url转换为url
            let mut normalized_server = server.clone();
            if let Some(http_url_value) = normalized_server.remove(FIELD_HTTP_URL) {
                normalized_server.insert(FIELD_URL.into(), http_url_value);
            }
            if !normalized_server.contains_key(FIELD_TYPE) {
                normalized_server.insert(FIELD_TYPE.into(), Value::String(transport.clone()));
            }

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
        let root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;
        let servers = root
            .get(FIELD_MCP_SERVERS)
            .and_then(Value::as_object)
            .ok_or_else(|| "Gemini MCP config does not contain mcpServers.".to_string())?;

        let server = servers
            .get(server_name)
            .ok_or_else(|| format!("Gemini MCP server not found: {server_name}"))?;
        let Some(server_obj) = server.as_object() else {
            return Err(format!("Gemini MCP server '{server_name}' must be an object."));
        };

        // 解析为EditableLocalMcpDto
        let explicit_type = server_obj.get(FIELD_TYPE).and_then(Value::as_str);
        let command = server_obj.get(FIELD_COMMAND).and_then(Value::as_str).map(str::to_string);
        let args = server_obj.get(FIELD_ARGS).and_then(Value::as_array).map(|arr| {
            arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }).unwrap_or_default();
        let env = server_obj.get(FIELD_ENV).and_then(Value::as_object).map(|obj| {
            obj.iter().filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string()))).collect()
        }).unwrap_or_default();
        let http_url = server_obj.get(FIELD_HTTP_URL).and_then(Value::as_str).map(str::to_string);
        let url = server_obj.get(FIELD_URL).and_then(Value::as_str).map(str::to_string);
        let final_url = http_url.or(url);
        let headers = server_obj.get(FIELD_HEADERS).and_then(Value::as_object).map(|obj| {
            obj.iter().filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string()))).collect()
        }).unwrap_or_default();

        let transport = if let Some(t) = explicit_type.filter(|s| !s.is_empty()) {
            match t {
                TRANSPORT_STDIO | "local" => TRANSPORT_STDIO.to_string(),
                TRANSPORT_HTTP => TRANSPORT_HTTP.to_string(),
                TRANSPORT_SSE | TRANSPORT_REMOTE => TRANSPORT_SSE.to_string(),
                other => {
                    return Err(format!(
                        "MCP server '{server_name}' has unsupported type '{other}'."
                    ))
                }
            }
        } else if command.is_some() {
            TRANSPORT_STDIO.to_string()
        } else if final_url.is_some() {
            TRANSPORT_HTTP.to_string()
        } else {
            return Err(format!(
                "MCP server '{server_name}' must include either 'command' or 'url'."
            ));
        };

        Ok(EditableLocalMcpDto {
            server_name: server_name.to_string(),
            transport,
            command,
            args,
            env,
            url: final_url,
            headers,
        })
    }

    fn write_server(&self, config_path: &Path, old_name: &str, new_name: &str, server: &ImportedMcpServer, _scope: &str, _project_path: Option<&str>) -> Result<UpdateLocalMcpResultDto, String> {
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
        let mut root = read_json_root_or_empty(config_path)?;
        let Some(root_object) = root.as_object_mut() else {
            return Err("Gemini MCP config root must be an object.".into());
        };
        let servers = root_object
            .entry(FIELD_MCP_SERVERS)
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(servers) = servers.as_object_mut() else {
            return Err("Gemini MCP config field 'mcpServers' must be an object.".into());
        };

        if old_name != new_name && servers.contains_key(new_name) {
            return Err(format!(
                "Gemini MCP server already exists: {new_name}"
            ));
        }
        if servers.remove(old_name).is_none() {
            return Err(format!("Gemini MCP server not found: {old_name}"));
        }

        // 构建JSON配置，Gemini用http_url而不是url
        let mut value = Map::new();
        value.insert(FIELD_TYPE.into(), Value::String(server.transport.clone()));
        if let Some(command) = &server.command {
            value.insert(FIELD_COMMAND.into(), Value::String(command.clone()));
        }
        if !server.args.is_empty() {
            value.insert(
                FIELD_ARGS.into(),
                Value::Array(server.args.iter().cloned().map(Value::String).collect()),
            );
        }
        if !server.env.is_empty() {
            value.insert(
                FIELD_ENV.into(),
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
            if server.transport == TRANSPORT_HTTP {
                value.insert(FIELD_HTTP_URL.into(), Value::String(url.clone()));
            } else {
                value.insert(FIELD_URL.into(), Value::String(url.clone()));
            }
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

        servers.insert(new_name.to_string(), Value::Object(value));
        write_json_value(config_path, &root)?;

        Ok(UpdateLocalMcpResultDto {
            config_path: normalize_path(config_path),
            server_name: new_name.to_string(),
        })
    }

    fn delete_server(&self, config_path: &Path, server_name: &str, scope: &str, project_path: Option<&str>) -> Result<(), String> {
        // Gemini and Claude use same JSON format, reuse logic
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
        let mut root = read_json_root_or_empty(config_path)?;

        let servers = match scope {
            "user" => {
                let Some(root_object) = root.as_object_mut() else {
                    return Err("Gemini MCP config root must be an object.".into());
                };
                let servers = root_object
                    .entry(FIELD_MCP_SERVERS)
                    .or_insert_with(|| Value::Object(Map::new()));
                servers
                    .as_object_mut()
                    .ok_or_else(|| "Gemini MCP config field 'mcpServers' must be an object.".to_string())?
            },
            "local" => {
                let Some(project_path) = project_path else {
                    return Err("Gemini local MCP edit requires project_path.".into());
                };
                let Some(root_object) = root.as_object_mut() else {
                    return Err("Gemini MCP config root must be an object.".into());
                };
                let projects = root_object
                    .entry("projects")
                    .or_insert_with(|| Value::Object(Map::new()));
                let Some(projects_object) = projects.as_object_mut() else {
                    return Err("Gemini MCP config field 'projects' must be an object.".into());
                };
                let project = projects_object
                    .entry(project_path)
                    .or_insert_with(|| Value::Object(Map::new()));
                let Some(project_object) = project.as_object_mut() else {
                    return Err("Gemini MCP config project must be an object.".into());
                };
                let servers = project_object
                    .entry(FIELD_MCP_SERVERS)
                    .or_insert_with(|| Value::Object(Map::new()));
                servers
                    .as_object_mut()
                    .ok_or_else(|| "Gemini MCP config field 'mcpServers' must be an object.".to_string())?
            },
            _ => return Err(format!("Unsupported scope: {}", scope))
        };

        if servers.remove(server_name).is_none() {
            return Err(format!("Gemini MCP server not found: {server_name}"));
        }

        // Clean up empty parent objects
        if servers.is_empty() {
            match scope {
                "user" => {
                    if let Some(root_object) = root.as_object_mut() {
                        root_object.remove(FIELD_MCP_SERVERS);
                    }
                },
                "local" => {
                    if let Some(project_path) = project_path {
                        if let Some(root_object) = root.as_object_mut() {
                            if let Some(projects) = root_object.get_mut("projects").and_then(Value::as_object_mut) {
                                if let Some(project) = projects.get_mut(project_path).and_then(Value::as_object_mut) {
                                    project.remove(FIELD_MCP_SERVERS);
                                    if project.is_empty() {
                                        projects.remove(project_path);
                                    }
                                    if projects.is_empty() {
                                        root_object.remove("projects");
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        write_json_value(config_path, &root)
    }

    fn import_servers(&self, config_path: &Path, servers: &BTreeMap<String, ImportedMcpServer>, conflict_strategy: McpImportConflictStrategy) -> Result<ImportLocalMcpResultDto, String> {
        // Gemini和Claude导入逻辑一致，复用
        ensure_parent_dir(config_path).map_err(|e| e.to_string())?;
        let mut root = read_json_root_or_empty(config_path)?;
        let Some(root_object) = root.as_object_mut() else {
            return Err("Gemini MCP config root must be an object.".into());
        };
        let servers_value = root_object
            .entry(FIELD_MCP_SERVERS)
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(existing_servers) = servers_value.as_object_mut() else {
            return Err("Gemini MCP config field 'mcpServers' must be an object.".into());
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
            value.insert(FIELD_TYPE.into(), Value::String(server.transport.clone()));
            if let Some(command) = &server.command {
                value.insert(FIELD_COMMAND.into(), Value::String(command.clone()));
            }
            if !server.args.is_empty() {
                value.insert(
                    FIELD_ARGS.into(),
                    Value::Array(server.args.iter().cloned().map(Value::String).collect()),
                );
            }
            if !server.env.is_empty() {
                value.insert(
                    FIELD_ENV.into(),
                    Value::Object(
                        server
                            .env
                            .iter()
                            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                            .collect(),
                    ),
                );
            }
            // Gemini导入时http类型用http_url字段
            if let Some(url) = &server.url {
                if server.transport == TRANSPORT_HTTP {
                    value.insert(FIELD_HTTP_URL.into(), Value::String(url.clone()));
                } else {
                    value.insert(FIELD_URL.into(), Value::String(url.clone()));
                }
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

// 辅助工具：读取JSON配置
fn read_json_root_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())
}

// 辅助工具：写入JSON配置（原子写入）
fn write_json_value(path: &Path, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    atomic_write(path, &content)
}
