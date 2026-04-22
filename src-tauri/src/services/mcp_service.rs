use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::constants::AgentType;
use crate::dto::mcp::{
    EditableLocalMcpDto, ImportedMcpServer, ImportLocalMcpResultDto, LocalMcpServerDto,
    McpImportConflictStrategy, McpScanTargetDto, UpdateLocalMcpResultDto,
};
use crate::scanners::mcp::create_mcp_handler;
use crate::infrastructure::utils::path::resolve_path;
use crate::services::ServiceError;

use tauri_plugin_opener::OpenerExt;

/// MCP configuration parser - pure domain logic without I/O
pub struct McpConfigParser;

impl McpConfigParser {
    /// Resolve MCP config file path for a given agent type and root path
    pub fn resolve_config_path(agent_type: &str, root_path: &str) -> Result<PathBuf, String> {
        let agent_type = crate::constants::AgentType::from_str(agent_type)?;
        let root_path = Path::new(root_path);

        let config_file = match agent_type {
            crate::constants::AgentType::Claude => crate::constants::CLAUDE_CONFIG_FILE,
            crate::constants::AgentType::Codex => crate::constants::CODEX_CONFIG_FILE,
            crate::constants::AgentType::Gemini => crate::constants::GEMINI_CONFIG_FILE,
            crate::constants::AgentType::OpenCode => {
                return Ok(crate::infrastructure::utils::path::user_home_dir()
                    .join(".config")
                    .join("opencode")
                    .join(crate::constants::OPENCODE_CONFIG_FILE));
            }
        };

        Ok(root_path.join(config_file))
    }

    /// Parse imported MCP payload JSON into server map
    pub fn parse_imported_payload(
        payload: &str,
    ) -> Result<BTreeMap<String, ImportedMcpServer>, String> {
        let root = serde_json::from_str::<serde_json::Value>(payload).map_err(|e| e.to_string())?;
        let Some(object) = root.as_object() else {
            return Err("MCP import JSON must be an object.".into());
        };

        let reserved_fields = vec![
            crate::constants::FIELD_TYPE,
            crate::constants::FIELD_COMMAND,
            crate::constants::FIELD_ARGS,
            crate::constants::FIELD_ENV,
            crate::constants::FIELD_URL,
            crate::constants::FIELD_HTTP_URL,
            crate::constants::FIELD_HEADERS,
        ];

        let server_map = if let Some(servers) = object.get(crate::constants::FIELD_MCP_SERVERS) {
            servers
                .as_object()
                .ok_or_else(|| "MCP field 'mcpServers' must be an object.".to_string())?
        } else {
            if object
                .keys()
                .any(|key| reserved_fields.contains(&key.as_str()))
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
                Self::parse_server_entry(server_name, value, &reserved_fields)
            })
            .collect()
    }

    fn parse_server_entry(
        server_name: &str,
        value: &serde_json::Value,
        reserved_fields: &[&str],
    ) -> Result<(String, ImportedMcpServer), String> {
        let Some(server) = value.as_object() else {
            return Err(format!("MCP server '{server_name}' must be an object."));
        };

        for key in server.keys() {
            if !reserved_fields.contains(&key.as_str()) {
                return Err(format!(
                    "MCP server '{server_name}' contains unsupported field '{key}'."
                ));
            }
        }

        let explicit_type = server.get(crate::constants::FIELD_TYPE).and_then(|v| v.as_str());
        let command = server
            .get(crate::constants::FIELD_COMMAND)
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let args = server
            .get(crate::constants::FIELD_ARGS)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let env = server
            .get(crate::constants::FIELD_ENV)
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        let http_url = server
            .get(crate::constants::FIELD_HTTP_URL)
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let url = server
            .get(crate::constants::FIELD_URL)
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let final_url = http_url.or(url);
        let headers = server
            .get(crate::constants::FIELD_HEADERS)
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let transport = Self::determine_transport(explicit_type, command.as_ref(), final_url.as_ref(), server_name)?;

        Ok((server_name.to_string(), ImportedMcpServer {
            transport,
            command,
            args,
            env,
            url: final_url,
            headers,
        }))
    }

    fn determine_transport(
        explicit_type: Option<&str>,
        command: Option<&String>,
        url: Option<&String>,
        server_name: &str,
    ) -> Result<String, String> {
        if let Some(t) = explicit_type.filter(|s| !s.is_empty()) {
            match t {
                crate::constants::TRANSPORT_STDIO | "local" => Ok(crate::constants::TRANSPORT_STDIO.to_string()),
                crate::constants::TRANSPORT_HTTP => Ok(crate::constants::TRANSPORT_HTTP.to_string()),
                crate::constants::TRANSPORT_SSE | crate::constants::TRANSPORT_REMOTE => {
                    Ok(crate::constants::TRANSPORT_SSE.to_string())
                }
                other => {
                    Err(format!(
                        "MCP server '{server_name}' has unsupported type '{other}'."
                    ))
                }
            }
        } else if command.is_some() {
            Ok(crate::constants::TRANSPORT_STDIO.to_string())
        } else if url.is_some() {
            Ok(crate::constants::TRANSPORT_HTTP.to_string())
        } else {
            Err(format!(
                "MCP server '{server_name}' must include either 'command' or 'url'."
            ))
        }
    }
}

/// MCP server validator - pure domain validation logic
pub struct McpServerValidator;

impl McpServerValidator {
    /// Validate editable MCP server configuration
    pub fn validate(
        server_name: &str,
        server: &ImportedMcpServer,
    ) -> Result<(), String> {
        if server_name.trim().is_empty() {
            return Err("MCP server name is required.".into());
        }

        match server.transport.as_str() {
            crate::constants::TRANSPORT_STDIO => {
                if server.command.as_deref().unwrap_or("").trim().is_empty() {
                    return Err("stdio MCP servers require a command.".into());
                }
            }
            crate::constants::TRANSPORT_HTTP | crate::constants::TRANSPORT_SSE => {
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
}

#[derive(Clone)]
pub struct McpService {}

impl Default for McpService {
    fn default() -> Self {
        Self::new()
    }
}

impl McpService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn open_mcp_config_folder(&self, app: tauri::AppHandle, config_path: &str) -> Result<(), ServiceError> {
        let path = resolve_path(config_path);
        if !path.exists() {
            return Err(ServiceError::BusinessRuleViolation(format!("MCP config path not found: {config_path}")));
        }

        let open_path = if path.is_dir() {
            path
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| ServiceError::BusinessRuleViolation(format!("MCP config path has no parent directory: {config_path}")))?
        };
        let open_path = open_path.to_string_lossy().to_string();

        app.opener()
            .open_path(&open_path, None::<&str>)
            .map_err(|e| ServiceError::Internal(e.to_string()))
    }

    pub fn open_mcp_config_file(&self, app: tauri::AppHandle, config_path: &str) -> Result<(), ServiceError> {
        let path = resolve_path(config_path);
        if !path.exists() || !path.is_file() {
            return Err(ServiceError::BusinessRuleViolation(format!("MCP config file not found: {config_path}")));
        }

        let open_path = path.to_string_lossy().to_string();
        app.opener()
            .open_path(&open_path, None::<&str>)
            .map_err(|e| ServiceError::Internal(e.to_string()))
    }

    pub fn list_local_mcps(&self, scan_targets: Vec<McpScanTargetDto>) -> Result<Vec<LocalMcpServerDto>, ServiceError> {
        println!(
            "[MCP] Scan requested for {} targets: {:?}",
            scan_targets.len(),
            scan_targets
                .iter()
                .map(|t| format!("agent_id={}, root_path={}", t.agent_id, t.root_path))
                .collect::<Vec<_>>()
        );

        let mut grouped = BTreeMap::<String, LocalMcpServerDto>::new();

        for target in scan_targets {
            let items = match target.agent_type.parse::<AgentType>() {
                Ok(agent_type) => {
                    let handler = create_mcp_handler(agent_type);
                    handler.scan_servers(&target)
                }
                Err(_) => Vec::new(),
            };

            for item in items {
                grouped.insert(item.id.clone(), item);
            }
        }

        let result: Vec<LocalMcpServerDto> = grouped.into_values().collect();

        println!("[MCP] Scan completed, found {} servers", result.len());
        for server in &result {
            println!(
                "[MCP] Found server: id={}, name={}, transport={}, config_path={}",
                server.id, server.name, server.transport, server.config_path
            );
        }

        Ok(result)
    }

    pub fn get_local_mcp_edit_data(
        &self,
        agent_type: String,
        config_path: String,
        server_name: String,
        scope: String,
        project_path: Option<String>,
    ) -> Result<EditableLocalMcpDto, ServiceError> {
        let path = resolve_path(&config_path);
        if !path.exists() || !path.is_file() {
            return Err(ServiceError::BusinessRuleViolation(format!("MCP config file not found: {config_path}")));
        }

        let agent_type_enum = agent_type.parse::<AgentType>().map_err(ServiceError::BusinessRuleViolation)?;
        let handler = create_mcp_handler(agent_type_enum);
        handler.read_server(&path, &server_name, &scope, project_path.as_deref())
            .map_err(ServiceError::Scanner)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_local_mcp(
        &self,
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
    ) -> Result<UpdateLocalMcpResultDto, ServiceError> {
        let path = resolve_path(&config_path);
        let next_server_name = next_server_name.trim().to_string();
        let next_server = ImportedMcpServer {
            transport,
            command: command
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            args: args
                .into_iter()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect(),
            env: env
                .into_iter()
                .map(|(k, v)| (k.trim().to_string(), v))
                .filter(|(k, _)| !k.is_empty())
                .collect(),
            url: url
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.trim().to_string(), v))
                .filter(|(k, _)| !k.is_empty())
                .collect(),
        };
        McpServerValidator::validate(&next_server_name, &next_server).map_err(ServiceError::BusinessRuleViolation)?;

        let agent_type_enum = agent_type.parse::<AgentType>().map_err(ServiceError::BusinessRuleViolation)?;
        let handler = create_mcp_handler(agent_type_enum);
        handler.write_server(
            &path,
            &server_name,
            &next_server_name,
            &next_server,
            &scope,
            project_path.as_deref(),
        ).map_err(ServiceError::Scanner)
    }

    pub fn delete_local_mcp(
        &self,
        agent_type: String,
        config_path: String,
        server_name: String,
        scope: String,
        project_path: Option<String>,
    ) -> Result<(), ServiceError> {
        let path = resolve_path(&config_path);
        if !path.exists() || !path.is_file() {
            return Err(ServiceError::BusinessRuleViolation(format!("MCP config file not found: {config_path}")));
        }

        if scope != "user" && scope != "local" {
            return Err(ServiceError::BusinessRuleViolation(format!("Invalid scope: {scope}, must be 'user' or 'local'")));
        }

        if scope == "local" && project_path.as_deref().unwrap_or_default().is_empty() {
            return Err(ServiceError::BusinessRuleViolation("Local scope requires project_path".to_string()));
        }

        let agent_type_enum = agent_type.parse::<AgentType>().map_err(ServiceError::BusinessRuleViolation)?;
        let handler = create_mcp_handler(agent_type_enum);
        handler.delete_server(&path, &server_name, &scope, project_path.as_deref())
            .map_err(ServiceError::Scanner)
    }

    pub fn import_local_mcp_json(
        &self,
        agent_type: String,
        root_path: String,
        json_payload: String,
        conflict_strategy: String,
    ) -> Result<ImportLocalMcpResultDto, ServiceError> {
        let strategy = conflict_strategy.parse::<McpImportConflictStrategy>().map_err(ServiceError::BusinessRuleViolation)?;
        let servers = McpConfigParser::parse_imported_payload(&json_payload).map_err(ServiceError::BusinessRuleViolation)?;
        let config_path = McpConfigParser::resolve_config_path(&agent_type, &root_path).map_err(ServiceError::BusinessRuleViolation)?;

        let agent_type_enum = agent_type.parse::<AgentType>().map_err(ServiceError::BusinessRuleViolation)?;
        let handler = create_mcp_handler(agent_type_enum);
        handler.import_servers(&config_path, &servers, strategy)
            .map_err(ServiceError::Scanner)
    }

    pub async fn launch_mcp_inspector(
        &self,
        app: tauri::AppHandle,
        config: EditableLocalMcpDto,
    ) -> Result<(), String> {
        crate::infrastructure::mcp::McpInspectorManager::launch(app, config).await
    }

    pub async fn stop_mcp_inspector(&self) -> Result<(), String> {
        crate::infrastructure::mcp::McpInspectorManager::stop().await
    }

    pub async fn cleanup_inspector_on_exit(&self) -> Result<(), String> {
        crate::infrastructure::mcp::McpInspectorManager::cleanup_on_exit().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use crate::scanners::mcp::create_mcp_handler;

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-mcp-service-test-{name}-{unique}"))
    }

    fn sample_imported_server() -> ImportedMcpServer {
        ImportedMcpServer {
            transport: crate::constants::TRANSPORT_STDIO.into(),
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
    fn delete_claude_mcp_server_removes_entry() {
        let root = temp_dir("claude");
        fs::create_dir_all(&root).expect("create temp dir");
        let config_path = root.join(".claude.json");
        fs::write(
            &config_path,
            r#"{"mcpServers":{"docs":{"url":"https://example.com"},"keep":{"command":"npx"}}}"#,
        )
        .expect("write config");

        let agent_type = crate::constants::AgentType::Claude;
        let handler = create_mcp_handler(agent_type);
        handler.delete_server(&config_path, "docs", "user", None).expect("delete server");

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

        let agent_type = crate::constants::AgentType::Gemini;
        let handler = create_mcp_handler(agent_type);
        handler.delete_server(&config_path, "docs", "user", None).expect("delete server");

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

        let agent_type = crate::constants::AgentType::Codex;
        let handler = create_mcp_handler(agent_type);
        handler.delete_server(&config_path, "docs", "user", None).expect("delete docs server");
        handler.delete_server(&config_path, "legacy", "user", None).expect("delete legacy server");

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
        "mcp": {
            "docs": { "type": "remote", "url": "https://example.com" },
            "keep": { "type": "local", "command": ["npx"] },
        },
    }"#,
        )
        .expect("write config");

        let agent_type = crate::constants::AgentType::OpenCode;
        let handler = create_mcp_handler(agent_type);
        handler.delete_server(&config_path, "docs", "user", None).expect("delete server");

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
                    transport: crate::constants::TRANSPORT_HTTP.into(),
                    command: None,
                    args: Vec::new(),
                    env: BTreeMap::new(),
                    url: Some("https://example.com/mcp".into()),
                    headers: BTreeMap::from([("Authorization".into(), "secret".into())]),
                },
            ),
        ]);

        let agent_type = crate::constants::AgentType::Claude;
        let handler = create_mcp_handler(agent_type);
        let result = handler
            .import_servers(&config_path, &servers, crate::dto::mcp::McpImportConflictStrategy::Skip)
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

        let agent_type = crate::constants::AgentType::Codex;
        let handler = create_mcp_handler(agent_type);
        let result = handler
            .import_servers(&config_path, &servers, crate::dto::mcp::McpImportConflictStrategy::Overwrite)
            .expect("import servers");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(result.imported_count, 1);
        assert!(updated.contains("[mcp_servers.filesystem]"));
        assert!(updated.contains("[mcp_servers.filesystem.env]"));

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

        let agent_type = crate::constants::AgentType::Claude;
        let handler = create_mcp_handler(agent_type);
        let result = handler
            .read_server(
                &config_path,
                "docs",
                "local",
                Some("D:/Workspace/demo"),
            )
            .expect("load edit data");

        assert_eq!(result.server_name, "docs");
        assert_eq!(result.transport, crate::constants::TRANSPORT_STDIO);
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
            transport: crate::constants::TRANSPORT_STDIO.into(),
            command: Some("node".into()),
            args: vec!["server.js".into()],
            env: BTreeMap::new(),
            url: None,
            headers: BTreeMap::new(),
        };

        let agent_type = crate::constants::AgentType::Codex;
        let handler = create_mcp_handler(agent_type);
        handler
            .write_server(
                &config_path,
                "docs",
                "filesystem",
                &updated_server,
                "user",
                None,
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

        let updated_server = sample_imported_server();
        let agent_type = crate::constants::AgentType::Codex;
        let handler = create_mcp_handler(agent_type);
        let error = handler
            .write_server(
                &config_path,
                "docs",
                "keep",
                &updated_server,
                "user",
                None,
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
        "mcp": {
            "docs": { "type": "remote", "url": "https://example.com/docs" },
        },
    }"#,
        )
        .expect("write config");

        let updated_server = ImportedMcpServer {
            transport: crate::constants::TRANSPORT_SSE.into(),
            command: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            url: Some("https://example.com/next".into()),
            headers: BTreeMap::from([("Authorization".into(), "secret".into())]),
        };

        let agent_type = crate::constants::AgentType::OpenCode;
        let handler = create_mcp_handler(agent_type);
        handler
            .write_server(
                &config_path,
                "docs",
                "docs",
                &updated_server,
                "user",
                None,
            )
            .expect("update opencode server");

        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("\"type\": \"remote\""));
        assert!(updated.contains("\"https://example.com/next\""));
        assert!(updated.contains("\"headers\""));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
