use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;
use tokio::sync::Mutex;
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

// MCP Inspector structured error codes
#[derive(Serialize, Debug)]
#[serde(tag = "code", content = "message")]
pub enum McpInspectorError {
    #[serde(rename = "NODE_NOT_INSTALLED")]
    NodeNotInstalled(String),
    #[serde(rename = "LAUNCH_FAILED")]
    LaunchFailed(String),
    #[serde(rename = "MISSING_COMMAND")]
    MissingCommand(String),
    #[serde(rename = "MISSING_URL")]
    MissingUrl(String),
    #[serde(rename = "UNKNOWN")]
    Unknown(String),
}

impl ToString for McpInspectorError {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#"{"code":"UNKNOWN","message":"Invalid error format"}"#.to_string())
    }
}

// Regex to match MCP Inspector access URL from output
static INSPECTOR_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"http://localhost:\d+/\?MCP_PROXY_AUTH_TOKEN=[a-f0-9]+").unwrap()
});

use crate::dto::mcp::{
    EditableLocalMcpDto, ImportedMcpServer, ImportLocalMcpResultDto, LocalMcpServerDto,
    McpImportConflictStrategy, McpScanTargetDto, UpdateLocalMcpResultDto,
};
use crate::scanners::mcp::create_mcp_handler;
use crate::utils::path::*;

// Global singleton store for running MCP Inspector process (only one allowed at a time)
static CURRENT_INSPECTOR: LazyLock<Mutex<Option<(u32, CommandChild)>>> =
    LazyLock::new(|| Mutex::new(None));

fn resolve_mcp_config_path(agent_type: &str, root_path: &str) -> Result<std::path::PathBuf, String> {
    let agent_type = crate::constants::AgentType::from_str(agent_type)?;
    let root_path = Path::new(root_path);
    match agent_type {
        crate::constants::AgentType::Claude => {
            Ok(root_path.join(crate::constants::CLAUDE_CONFIG_FILE))
        }
        crate::constants::AgentType::Codex => Ok(root_path.join(crate::constants::CODEX_CONFIG_FILE)),
        crate::constants::AgentType::Gemini => {
            Ok(root_path.join(crate::constants::GEMINI_CONFIG_FILE))
        }
        crate::constants::AgentType::OpenCode => Ok(crate::utils::path::user_home_dir()
            .join(".config")
            .join("opencode")
            .join(crate::constants::OPENCODE_CONFIG_FILE)),
    }
}

fn parse_imported_mcp_payload(
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

            let transport = match explicit_type.unwrap_or_default() {
                "" => {
                    if command.is_some() {
                        crate::constants::TRANSPORT_STDIO.to_string()
                    } else if final_url.is_some() {
                        crate::constants::TRANSPORT_HTTP.to_string()
                    } else {
                        return Err(format!(
                            "MCP server '{server_name}' must include either 'command' or 'url'."
                        ));
                    }
                }
                crate::constants::TRANSPORT_STDIO | "local" => crate::constants::TRANSPORT_STDIO.to_string(),
                crate::constants::TRANSPORT_HTTP => crate::constants::TRANSPORT_HTTP.to_string(),
                crate::constants::TRANSPORT_SSE | crate::constants::TRANSPORT_REMOTE => {
                    crate::constants::TRANSPORT_SSE.to_string()
                }
                other => {
                    return Err(format!(
                        "MCP server '{server_name}' has unsupported type '{other}'."
                    ))
                }
            };

            Ok((server_name.clone(), ImportedMcpServer {
                transport,
                command,
                args,
                env,
                url: final_url,
                headers,
            }))
        })
        .collect()
}

fn validate_editable_mcp_server(
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

    let mut grouped = BTreeMap::<String, LocalMcpServerDto>::new();

    for target in scan_targets {
        let items = match target.agent_type.parse::<crate::constants::AgentType>() {
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
        .map_err(|e| e.to_string())
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
        .map_err(|e| e.to_string())
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

    let agent_type_enum = agent_type.parse::<crate::constants::AgentType>()?;
    let handler = create_mcp_handler(agent_type_enum);
    handler.read_server(&path, &server_name, &scope, project_path.as_deref())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
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
    validate_editable_mcp_server(&next_server_name, &next_server)?;

    let agent_type_enum = agent_type.parse::<crate::constants::AgentType>()?;
    let handler = create_mcp_handler(agent_type_enum);
    handler.write_server(
        &path,
        &server_name,
        &next_server_name,
        &next_server,
        &scope,
        project_path.as_deref(),
    )
}

#[tauri::command]
pub fn delete_local_mcp(
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<(), String> {
    let path = resolve_path(&config_path);
    if !path.exists() || !path.is_file() {
        return Err(format!("MCP config file not found: {config_path}"));
    }

    // Validate scope
    if scope != "user" && scope != "local" {
        return Err(format!("Invalid scope: {scope}, must be 'user' or 'local'"));
    }

    // local scope requires project_path
    if scope == "local" && project_path.as_deref().unwrap_or_default().is_empty() {
        return Err("Local scope requires project_path".to_string());
    }

    let agent_type_enum = agent_type.parse::<crate::constants::AgentType>()?;
    let handler = create_mcp_handler(agent_type_enum);
    handler.delete_server(&path, &server_name, &scope, project_path.as_deref())
}

#[tauri::command]
pub fn import_local_mcp_json(
    agent_type: String,
    root_path: String,
    json_payload: String,
    conflict_strategy: String,
) -> Result<ImportLocalMcpResultDto, String> {
    let strategy = conflict_strategy.parse::<McpImportConflictStrategy>()?;
    let servers = parse_imported_mcp_payload(&json_payload)?;
    let config_path = resolve_mcp_config_path(&agent_type, &root_path)?;

    let agent_type_enum = agent_type.parse::<crate::constants::AgentType>()?;
    let handler = create_mcp_handler(agent_type_enum);
    handler.import_servers(&config_path, &servers, strategy)
}

#[tauri::command]
pub async fn launch_mcp_inspector(
    app: tauri::AppHandle,
    config: EditableLocalMcpDto,
) -> Result<(), String> {
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

    let check_npx = app
        .shell()
        .command(npx_cmd)
        .args(["--version"])
        .output()
        .await
        .map_err(|e| format!("Failed to find npx command: {}", e))?;

    if !check_npx.status.success() {
        return Err(McpInspectorError::NodeNotInstalled(
            "Node.js is not installed. Please install Node.js first:\n\nhttps://nodejs.org/".to_string()
        ).to_string());
    }

    // Build inspector arguments - use npx to run directly without global installation
    // Add -y to auto confirm package installation without user prompt
    // Don't fix port, let Inspector auto select available port, extract actual URL from output later
    let mut args = vec!["-y".to_string(), "@modelcontextprotocol/inspector@latest".to_string()];

    match config.transport.as_str() {
        crate::constants::TRANSPORT_STDIO => {
            if let Some(cmd) = &config.command {
                let mut cmd_parts = vec![cmd.clone()];
                cmd_parts.extend(config.args.clone());
                // Use -- separator to pass command and args directly without shell escaping
                // This avoids cross-platform quoting issues and injection risks
                args.push("--".to_string());
                args.extend(cmd_parts);
            } else {
                return Err(McpInspectorError::MissingCommand(
                    "stdio transport requires a command".to_string()
                ).to_string());
            }
        }
        crate::constants::TRANSPORT_HTTP | crate::constants::TRANSPORT_SSE => {
            if let Some(url) = &config.url {
                args.push("--url".to_string());
                args.push(url.clone());
            } else {
                return Err(McpInspectorError::MissingUrl(
                    format!("{} transport requires a url", config.transport)
                ).to_string());
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

    // --name parameter is no longer supported in latest MCP Inspector

    // Build npx command with Tauri shell plugin
    let cmd = app.shell().command(npx_cmd).args(args);

    // Spawn process in background
    let (mut rx, child) = cmd
        .spawn()
        .map_err(|e| McpInspectorError::LaunchFailed(
            format!("Failed to launch MCP Inspector: {}", e)
        ).to_string())?;

    let pid = child.pid();

    // 存储进程到全局单例，stop的时候可以找到
    CURRENT_INSPECTOR.lock().await.replace((pid, child));
    eprintln!("[MCP Inspector] ✅ Process started and stored, PID: {}", pid);

    // Listen to process output in background, log all stdout/stderr to console and forward to frontend
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(data) => {
                    if let Ok(output) = String::from_utf8(data) {
                        println!("[MCP Inspector] stdout: {}", output.trim());
                        // Forward output to frontend
                        let _ = app.emit("mcp-inspector-output", serde_json::json!({
                            "pid": pid,
                            "type": "stdout",
                            "data": output
                        }));

                        // Extract inspector access URL and push to frontend
                        if let Some(url_match) = INSPECTOR_URL_REGEX.find(&output) {
                            let access_url = url_match.as_str();
                            println!("[MCP Inspector] Extracted access URL: {}", access_url);
                            let _ = app.emit("mcp-inspector-url", serde_json::json!({
                                "pid": pid,
                                "url": access_url
                            }));
                        }
                    }
                }
                CommandEvent::Stderr(data) => {
                    if let Ok(output) = String::from_utf8(data) {
                        eprintln!("[MCP Inspector] stderr: {}", output.trim());
                        // Forward error output to frontend
                        let _ = app.emit("mcp-inspector-output", serde_json::json!({
                            "pid": pid,
                            "type": "stderr",
                            "data": output
                        }));
                    }
                }
                _ => {} // Ignore other events
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_mcp_inspector(
    _app: tauri::AppHandle,
) -> Result<(), String> {
    // 强制输出到stderr，红色显示，不会被缓冲，肯定能看到
    eprintln!("[MCP Inspector] 🛑 Received stop command from frontend");

    let mut current_lock = CURRENT_INSPECTOR.lock().await;

    // Stop and remove current singleton process if exists
    if let Some((pid, child)) = current_lock.take() {
        eprintln!("[MCP Inspector] Killing process PID: {}", pid);
        match child.kill() {
            Ok(_) => eprintln!("[MCP Inspector] ✅ Successfully stopped process PID: {}", pid),
            Err(e) => {
                eprintln!("[MCP Inspector] ❌ Failed to stop process PID: {}: {}", pid, e);
                return Err(format!("Failed to stop process: {}", e));
            }
        }
    } else {
        eprintln!("[MCP Inspector] ℹ️ No running inspector process to stop");
    }

    Ok(())
}

/// Cleanup inspector process on application exit
/// This function is called from the app exit handler to ensure child processes are terminated
pub async fn cleanup_inspector_on_exit() -> Result<(), String> {
    let mut current_lock = CURRENT_INSPECTOR.lock().await;
    if let Some((pid, child)) = current_lock.take() {
        eprintln!("[MCP Inspector] 🧹 Cleanup on app exit, killing PID: {}", pid);
        match child.kill() {
            Ok(_) => eprintln!("[MCP Inspector] ✅ Cleanup successful for PID: {}", pid),
            Err(e) => eprintln!("[MCP Inspector] ⚠️ Cleanup failed for PID: {}: {}", pid, e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
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
            Some(crate::constants::TRANSPORT_HTTP)
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
            .import_servers(&config_path, &servers, McpImportConflictStrategy::Skip)
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
            .import_servers(&config_path, &servers, McpImportConflictStrategy::Overwrite)
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
