//! Claude MCP配置处理实现
use std::path::Path;
use std::collections::BTreeMap;
use serde_json::{Value, Map};
use crate::dto::mcp::{McpScanTargetDto, LocalMcpServerDto, EditableLocalMcpDto, ImportedMcpServer, ImportLocalMcpResultDto, UpdateLocalMcpResultDto};
use crate::constants::*;
use crate::utils::path::{ensure_parent_directory, atomic_write, user_home_dir, normalize_path};
use super::{McpConfigHandler, common::*};
use crate::dto::mcp::McpImportConflictStrategy;

pub struct ClaudeHandler;

impl McpConfigHandler for ClaudeHandler {
    fn scan_servers(&self, target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
        let config_path = user_home_dir().join(CLAUDE_CONFIG_FILE);
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

        let mut items = Vec::new();
        // 全局用户配置
        if let Some(servers) = value.get(FIELD_MCP_SERVERS).and_then(Value::as_object) {
            for (server_name, server_value) in servers {
                let Some(server) = server_value.as_object() else {
                    continue;
                };
                let transport = transport_from_config(
                    server.get(FIELD_TYPE).and_then(Value::as_str),
                    server.get(FIELD_COMMAND).and_then(Value::as_str),
                    server.get(FIELD_URL).and_then(Value::as_str),
                );
                let endpoint = server
                    .get(FIELD_URL)
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| {
                        server
                            .get(FIELD_COMMAND)
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .unwrap_or_default();
                items.push(build_local_mcp(
                    target,
                    server_name,
                    "user",
                    &config_path,
                    None,
                    transport,
                    endpoint,
                    masked_json_config(server),
                    Vec::new(),
                    Vec::new(),
                ));
            }
        }

        // 项目级配置
        if let Some(projects) = value.get("projects").and_then(Value::as_object) {
            for (project_path, project_value) in projects {
                let Some(project) = project_value.as_object() else {
                    continue;
                };
                let Some(servers) = project.get(FIELD_MCP_SERVERS).and_then(Value::as_object) else {
                    continue;
                };
                for (server_name, server_value) in servers {
                    let Some(server) = server_value.as_object() else {
                        continue;
                    };
                    let transport = transport_from_config(
                        server.get(FIELD_TYPE).and_then(Value::as_str),
                        server.get(FIELD_COMMAND).and_then(Value::as_str),
                        server.get(FIELD_URL).and_then(Value::as_str),
                    );
                    let endpoint = server
                        .get(FIELD_URL)
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| {
                            server
                                .get(FIELD_COMMAND)
                                .and_then(Value::as_str)
                                .map(str::to_string)
                        })
                        .unwrap_or_default();
                    items.push(build_local_mcp(
                        target,
                        server_name,
                        "local",
                        &config_path,
                        Some(project_path),
                        transport,
                        endpoint,
                        masked_json_config(server),
                        Vec::new(),
                        Vec::new(),
                    ));
                }
            }
        }

        items
    }

    fn read_server(&self, config_path: &Path, server_name: &str, scope: &str, project_path: Option<&str>) -> Result<EditableLocalMcpDto, String> {
        let content = std::fs::read_to_string(config_path).map_err(|error| error.to_string())?;
        let root = serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())?;

        // 查找对应的服务器
        let servers = match scope {
            "user" => root
                .get(FIELD_MCP_SERVERS)
                .and_then(Value::as_object)
                .ok_or_else(|| "Claude MCP config does not contain user mcpServers.".into()),
            "local" => {
                let Some(project_path) = project_path else {
                    return Err("Claude local MCP edit requires project_path.".into());
                };
                root.get("projects")
                    .and_then(Value::as_object)
                    .and_then(|projects| projects.get(project_path))
                    .and_then(Value::as_object)
                    .and_then(|project| project.get(FIELD_MCP_SERVERS))
                    .and_then(Value::as_object)
                    .ok_or_else(|| format!("Claude local MCP project not found: {project_path}"))
            }
            _ => Err(format!("Unsupported Claude MCP scope: {scope}")),
        }?;

        let server = servers
            .get(server_name)
            .ok_or_else(|| format!("Claude MCP server not found: {server_name}"))?;

        // 解析为EditableLocalMcpDto
        let explicit_type = server.get(FIELD_TYPE).and_then(Value::as_str);
        let command = server.get(FIELD_COMMAND).and_then(Value::as_str).map(str::to_string);
        let args = server.get(FIELD_ARGS).and_then(Value::as_array).map(|arr| {
            arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }).unwrap_or_default();
        let env = server.get(FIELD_ENV).and_then(Value::as_object).map(|obj| {
            obj.iter().filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string()))).collect()
        }).unwrap_or_default();
        let url = server.get(FIELD_URL).and_then(Value::as_str).map(str::to_string);
        let headers = server.get(FIELD_HEADERS).and_then(Value::as_object).map(|obj| {
            obj.iter().filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string()))).collect()
        }).unwrap_or_default();

        let transport = match explicit_type.unwrap_or_default() {
            "" => {
                if command.is_some() {
                    TRANSPORT_STDIO.to_string()
                } else if url.is_some() {
                    TRANSPORT_HTTP.to_string()
                } else {
                    return Err(format!(
                        "MCP server '{server_name}' must include either 'command' or 'url'."
                    ));
                }
            }
            TRANSPORT_STDIO | "local" => TRANSPORT_STDIO.to_string(),
            TRANSPORT_HTTP => TRANSPORT_HTTP.to_string(),
            TRANSPORT_SSE | TRANSPORT_REMOTE => TRANSPORT_SSE.to_string(),
            other => {
                return Err(format!(
                    "MCP server '{server_name}' has unsupported type '{other}'."
                ))
            }
        };

        Ok(EditableLocalMcpDto {
            server_name: server_name.to_string(),
            transport,
            command,
            args,
            env,
            url,
            headers,
        })
    }

    fn write_server(&self, config_path: &Path, old_name: &str, new_name: &str, server: &ImportedMcpServer, scope: &str, project_path: Option<&str>) -> Result<UpdateLocalMcpResultDto, String> {
        ensure_parent_directory(config_path)?;
        let mut root = read_json_root_or_empty(config_path)?;
        let servers = find_claude_servers_mut(&mut root, scope, project_path)?;

        if old_name != new_name && servers.contains_key(new_name) {
            return Err(format!(
                "Claude MCP server already exists: {new_name}"
            ));
        }
        if servers.remove(old_name).is_none() {
            return Err(format!("Claude MCP server not found: {old_name}"));
        }

        // 构建JSON配置
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

        servers.insert(new_name.to_string(), Value::Object(value));
        write_json_value(config_path, &root)?;

        Ok(UpdateLocalMcpResultDto {
            config_path: normalize_path(config_path),
            server_name: new_name.to_string(),
        })
    }

    fn delete_server(&self, config_path: &Path, server_name: &str, scope: &str, project_path: Option<&str>) -> Result<(), String> {
        ensure_parent_directory(config_path)?;
        let mut root = read_json_root_or_empty(config_path)?;
        let servers = find_claude_servers_mut(&mut root, scope, project_path)?;

        if servers.remove(server_name).is_none() {
            return Err(format!("Claude MCP server not found: {server_name}"));
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
        ensure_parent_directory(config_path)?;
        let mut root = read_json_root_or_empty(config_path)?;
        let Some(root_object) = root.as_object_mut() else {
            return Err("Claude MCP config root must be an object.".into());
        };
        let servers_value = root_object
            .entry(FIELD_MCP_SERVERS)
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

// 辅助函数：查找Claude服务器可变引用
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
                .entry(FIELD_MCP_SERVERS)
                .or_insert_with(|| Value::Object(Map::new()));
            servers
                .as_object_mut()
                .ok_or_else(|| "Claude MCP config field 'mcpServers' must be an object.".into())
        }
        "local" => {
            let Some(project_path) = project_path else {
                return Err("Claude local MCP edit requires project_path.".into());
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
                .entry(FIELD_MCP_SERVERS)
                .or_insert_with(|| Value::Object(Map::new()));
            servers.as_object_mut().ok_or_else(|| {
                "Claude local MCP config field 'mcpServers' must be an object.".into()
            })
        }
        _ => Err(format!("Unsupported Claude MCP scope: {scope}")),
    }
}

// 公共工具：读取JSON配置，不存在返回空对象
fn read_json_root_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&content).map_err(|error| error.to_string())
}

// 公共工具：写入JSON配置（原子写入）
fn write_json_value(path: &Path, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    atomic_write(path, &content)
}
