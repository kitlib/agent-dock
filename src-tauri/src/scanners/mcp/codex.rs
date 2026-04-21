//! Codex MCP配置处理实现
use std::path::Path;
use std::collections::BTreeMap;
use toml;
use crate::dto::mcp::{McpScanTargetDto, LocalMcpServerDto, EditableLocalMcpDto, ImportedMcpServer, ImportLocalMcpResultDto, UpdateLocalMcpResultDto};
use crate::constants::*;
use crate::utils::path::{ensure_parent_directory, atomic_write, normalize_path, resolve_agent_root};
use super::{McpConfigHandler, common::*};
use crate::dto::mcp::McpImportConflictStrategy;

pub struct CodexHandler;

impl McpConfigHandler for CodexHandler {
    fn scan_servers(&self, target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
        let root_path = resolve_agent_root(&target.root_path);
        let config_path = root_path.join(CODEX_CONFIG_FILE);
        if !config_path.exists() || !config_path.is_file() {
            return Vec::new();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(_) => return Vec::new(),
        };
        let value = match toml::from_str::<toml::Value>(&contents) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        let mut items = Vec::new();
        // 支持mcp_servers和mcp.servers两种格式
        if let Some(servers) = value.get("mcp_servers").and_then(toml::Value::as_table) {
            items.extend(scan_codex_server_tables(target, &config_path, servers));
        }
        if let Some(servers) = value
            .get("mcp")
            .and_then(toml::Value::as_table)
            .and_then(|mcp| mcp.get("servers"))
            .and_then(toml::Value::as_table)
        {
            items.extend(scan_codex_server_tables(target, &config_path, servers));
        }

        items
    }

    fn read_server(&self, config_path: &Path, server_name: &str, _scope: &str, _project_path: Option<&str>) -> Result<EditableLocalMcpDto, String> {
        let content = std::fs::read_to_string(config_path).map_err(|error| error.to_string())?;
        let root = toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())?;

        // 查找服务器（支持两种格式）
        let server = if let Some(server) = root
            .get("mcp_servers")
            .and_then(toml::Value::as_table)
            .and_then(|servers| servers.get(server_name))
        {
            server
        } else if let Some(server) = root
            .get("mcp")
            .and_then(toml::Value::as_table)
            .and_then(|mcp| mcp.get("servers"))
            .and_then(toml::Value::as_table)
            .and_then(|servers| servers.get(server_name))
        {
            server
        } else {
            return Err(format!("Codex MCP server not found: {server_name}"));
        };

        // 解析为EditableLocalMcpDto
        let Some(table) = server.as_table() else {
            return Err(format!("Codex MCP server '{server_name}' is not a table."));
        };

        let transport = match table.get(FIELD_TYPE).and_then(toml::Value::as_str) {
            Some(TRANSPORT_HTTP) => TRANSPORT_HTTP.to_string(),
            Some(TRANSPORT_SSE) => TRANSPORT_SSE.to_string(),
            Some(other) => other.to_string(),
            None => {
                if table.get(FIELD_COMMAND).and_then(toml::Value::as_str).is_some() {
                    TRANSPORT_STDIO.to_string()
                } else if table.get(FIELD_URL).and_then(toml::Value::as_str).is_some() {
                    TRANSPORT_HTTP.to_string()
                } else {
                    TRANSPORT_STDIO.to_string()
                }
            }
        };
        let args = table
            .get(FIELD_ARGS)
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
            .get(FIELD_ENV)
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
            .or_else(|| table.get(FIELD_HEADERS))
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
                .get(FIELD_COMMAND)
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            args,
            env,
            url: table
                .get(FIELD_URL)
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            headers,
        })
    }

    fn write_server(&self, config_path: &Path, old_name: &str, new_name: &str, server: &ImportedMcpServer, _scope: &str, _project_path: Option<&str>) -> Result<UpdateLocalMcpResultDto, String> {
        ensure_parent_directory(config_path)?;
        let mut root = read_toml_root_or_empty(config_path)?;
        let Some(root_table) = root.as_table_mut() else {
            return Err("Codex config root must be a table.".into());
        };

        // 检查重名冲突
        let has_conflict = old_name != new_name
            && (root_table
                .get("mcp_servers")
                .and_then(toml::Value::as_table)
                .is_some_and(|servers| servers.contains_key(new_name))
                || root_table
                    .get("mcp")
                    .and_then(toml::Value::as_table)
                    .and_then(|mcp| mcp.get("servers"))
                    .and_then(toml::Value::as_table)
                    .is_some_and(|servers| servers.contains_key(new_name)));
        if has_conflict {
            return Err(format!(
                "Codex MCP server already exists: {new_name}"
            ));
        }

        // 删除旧配置（两种格式都要删）
        let mut removed = false;
        if let Some(servers) = root_table
            .get_mut("mcp_servers")
            .and_then(toml::Value::as_table_mut)
        {
            removed = servers.remove(old_name).is_some() || removed;
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
                removed = servers.remove(old_name).is_some() || removed;
                if servers.is_empty() {
                    mcp_table.remove("servers");
                }
            }
            if mcp_table.is_empty() {
                root_table.remove("mcp");
            }
        }
        if !removed {
            return Err(format!("Codex MCP server not found: {old_name}"));
        }

        // 写入新配置到mcp_servers格式
        let servers_value = root_table
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
        let Some(servers) = servers_value.as_table_mut() else {
            return Err("Codex config field 'mcp_servers' must be a table.".into());
        };
        servers.insert(
            new_name.to_string(),
            build_codex_server_value(server),
        );

        // 保存文件（原子写入）
        let content = toml::to_string_pretty(&root).map_err(|error| error.to_string())?;
        atomic_write(config_path, &content)?;

        Ok(UpdateLocalMcpResultDto {
            config_path: normalize_path(config_path),
            server_name: new_name.to_string(),
        })
    }

    fn delete_server(&self, config_path: &Path, server_name: &str, _scope: &str, _project_path: Option<&str>) -> Result<(), String> {
        let content = std::fs::read_to_string(config_path).map_err(|error| error.to_string())?;
        let mut root = toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())?;
        let Some(root_table) = root.as_table_mut() else {
            return Err("Codex config root is not a table.".into());
        };

        let mut removed = false;
        // 删除两种格式的配置
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
        atomic_write(config_path, &content)?;
        Ok(())
    }

    fn import_servers(&self, config_path: &Path, servers: &BTreeMap<String, ImportedMcpServer>, conflict_strategy: McpImportConflictStrategy) -> Result<ImportLocalMcpResultDto, String> {
        ensure_parent_directory(config_path)?;
        let mut root = read_toml_root_or_empty(config_path)?;
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
            if server.transport != TRANSPORT_STDIO {
                table.insert(FIELD_TYPE.into(), toml::Value::String(server.transport.clone()));
            }
            if let Some(command) = &server.command {
                table.insert(FIELD_COMMAND.into(), toml::Value::String(command.clone()));
            }
            if !server.args.is_empty() {
                table.insert(
                    FIELD_ARGS.into(),
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
                    FIELD_ENV.into(),
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
                table.insert(FIELD_URL.into(), toml::Value::String(url.clone()));
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
        atomic_write(config_path, &content)?;
        Ok(ImportLocalMcpResultDto {
            config_path: normalize_path(config_path),
            imported_count: imported_names.len() as u32,
            skipped_count: skipped_names.len() as u32,
            imported_names,
            skipped_names,
        })
    }
}

// 辅助函数：扫描Codex服务器表
fn scan_codex_server_tables(
    target: &McpScanTargetDto,
    config_path: &Path,
    server_tables: &toml::value::Table,
) -> Vec<LocalMcpServerDto> {
    let mut items = Vec::new();
    for (server_name, server_value) in server_tables {
        let Some(server) = server_value.as_table() else {
            continue;
        };
        let transport = transport_from_config(
            server.get(FIELD_TYPE).and_then(toml::Value::as_str),
            server.get(FIELD_COMMAND).and_then(toml::Value::as_str),
            server.get(FIELD_URL).and_then(toml::Value::as_str),
        );
        let endpoint = server
            .get(FIELD_URL)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                server
                    .get(FIELD_COMMAND)
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default();
        items.push(build_local_mcp(
            target,
            server_name,
            "user",
            config_path,
            None,
            transport,
            endpoint,
            masked_toml_config(server),
            Vec::new(),
            Vec::new(),
        ));
    }
    items
}

// 辅助函数：读取TOML配置，不存在返回空对象
fn read_toml_root_or_empty(path: &Path) -> Result<toml::Value, String> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::value::Table::new()));
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    toml::from_str::<toml::Value>(&content).map_err(|error| error.to_string())
}

// 辅助函数：构建Codex服务器配置
fn build_codex_server_value(server: &ImportedMcpServer) -> toml::Value {
    let mut table = toml::value::Table::new();
    if server.transport != TRANSPORT_STDIO {
        table.insert(FIELD_TYPE.into(), toml::Value::String(server.transport.clone()));
    }
    if let Some(command) = &server.command {
        table.insert(FIELD_COMMAND.into(), toml::Value::String(command.clone()));
    }
    if !server.args.is_empty() {
        table.insert(
            FIELD_ARGS.into(),
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
            FIELD_ENV.into(),
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
        table.insert(FIELD_URL.into(), toml::Value::String(url.clone()));
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
