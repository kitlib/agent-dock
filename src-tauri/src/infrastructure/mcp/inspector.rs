use std::fmt;
use std::sync::LazyLock;
use regex::Regex;
use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;

use crate::dto::mcp::EditableLocalMcpDto;

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

impl fmt::Display for McpInspectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_else(|_| r#"{"code":"UNKNOWN","message":"Invalid error format"}"#.to_string());
        write!(f, "{}", s)
    }
}

// Regex to match MCP Inspector access URL from output
static INSPECTOR_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"http://localhost:\d+/\?MCP_PROXY_AUTH_TOKEN=[a-f0-9]+").unwrap()
});

// Global singleton store for running MCP Inspector process (only one allowed at a time)
static CURRENT_INSPECTOR: LazyLock<Mutex<Option<(u32, CommandChild)>>> =
    LazyLock::new(|| Mutex::new(None));

/// MCP Inspector process manager - handles inspector lifecycle
pub struct McpInspectorManager;

impl McpInspectorManager {
    /// Launch MCP Inspector process
    pub async fn launch(
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

        // Build inspector arguments
        let args = Self::build_inspector_args(&config)?;

        // Build npx command with Tauri shell plugin
        let cmd = app.shell().command(npx_cmd).args(args);

        // Spawn process in background
        let (mut rx, child) = cmd
            .spawn()
            .map_err(|e| McpInspectorError::LaunchFailed(
                format!("Failed to launch MCP Inspector: {}", e)
            ).to_string())?;

        let pid = child.pid();

        // Store process to global singleton
        CURRENT_INSPECTOR.lock().await.replace((pid, child));
        eprintln!("[MCP Inspector] Process started and stored, PID: {}", pid);

        // Listen to process output in background
        tauri::async_runtime::spawn(async move {
            use tauri_plugin_shell::process::CommandEvent;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(data) => {
                        if let Ok(output) = String::from_utf8(data) {
                            println!("[MCP Inspector] stdout: {}", output.trim());
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
                            let _ = app.emit("mcp-inspector-output", serde_json::json!({
                                "pid": pid,
                                "type": "stderr",
                                "data": output
                            }));
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Stop MCP Inspector process
    pub async fn stop() -> Result<(), String> {
        eprintln!("[MCP Inspector] Received stop command from frontend");

        let mut current_lock = CURRENT_INSPECTOR.lock().await;

        if let Some((pid, child)) = current_lock.take() {
            eprintln!("[MCP Inspector] Killing process PID: {}", pid);
            match child.kill() {
                Ok(_) => eprintln!("[MCP Inspector] Successfully stopped process PID: {}", pid),
                Err(e) => {
                    eprintln!("[MCP Inspector] Failed to stop process PID: {}: {}", pid, e);
                    return Err(format!("Failed to stop process: {}", e));
                }
            }
        } else {
            eprintln!("[MCP Inspector] No running inspector process to stop");
        }

        Ok(())
    }

    /// Cleanup inspector process on application exit
    pub async fn cleanup_on_exit() -> Result<(), String> {
        let mut current_lock = CURRENT_INSPECTOR.lock().await;
        if let Some((pid, child)) = current_lock.take() {
            eprintln!("[MCP Inspector] Cleanup on app exit, killing PID: {}", pid);
            match child.kill() {
                Ok(_) => eprintln!("[MCP Inspector] Cleanup successful for PID: {}", pid),
                Err(e) => eprintln!("[MCP Inspector] Cleanup failed for PID: {}: {}", pid, e),
            }
        }
        Ok(())
    }

    fn build_inspector_args(config: &EditableLocalMcpDto) -> Result<Vec<String>, String> {
        let mut args = vec!["-y".to_string(), "@modelcontextprotocol/inspector@latest".to_string()];

        match config.transport.as_str() {
            crate::constants::TRANSPORT_STDIO => {
                if let Some(cmd) = &config.command {
                    let mut cmd_parts = vec![cmd.clone()];
                    cmd_parts.extend(config.args.clone());
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

        Ok(args)
    }
}
