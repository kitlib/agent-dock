use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};

use crate::dto::mcp::{LocalMcpServerDto, McpScanTargetDto};

const CLAUDE_CONFIG_FILE: &str = ".claude.json";
const CODEX_CONFIG_FILE: &str = "config.toml";
const GEMINI_CONFIG_FILE: &str = "settings.json";
const OPENCODE_CONFIG_PATH: [&str; 3] = [".config", "opencode", "opencode.json"];

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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

fn resolve_agent_root(root_path: &str) -> PathBuf {
    if let Some(relative_path) = root_path
        .strip_prefix("~/")
        .or_else(|| root_path.strip_prefix("~\\"))
    {
        return user_home_dir().join(relative_path);
    }

    let path = PathBuf::from(root_path);
    if path.is_absolute() {
        return path;
    }

    user_home_dir().join(path)
}

fn updated_at(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|datetime| datetime.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
}

fn sanitize_id_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "mcp".into()
    } else {
        trimmed.into()
    }
}

fn transport_from_config(
    explicit_type: Option<&str>,
    command: Option<&str>,
    url: Option<&str>,
) -> String {
    if let Some(transport) = explicit_type.filter(|value| !value.trim().is_empty()) {
        return transport.to_string();
    }
    if command.is_some() {
        return "stdio".into();
    }
    if let Some(endpoint) = url {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            return "http".into();
        }
        return "remote".into();
    }
    "unknown".into()
}

fn short_summary(transport: &str, endpoint: &str, scope: &str) -> String {
    match transport {
        "stdio" => format!("Configured as a {scope} stdio MCP server."),
        "http" | "sse" => format!("Configured as a {scope} remote MCP server."),
        _ => {
            if endpoint.is_empty() {
                format!("Configured as a {scope} MCP server.")
            } else {
                format!("Configured as a {scope} MCP server via {transport}.")
            }
        }
    }
}

fn markdown_document(
    name: &str,
    scope: &str,
    transport: &str,
    endpoint: &str,
    config_path: &str,
    project_path: Option<&str>,
    warnings: &[String],
    errors: &[String],
) -> String {
    let mut lines = vec![
        format!("# {name}"),
        String::new(),
        format!("- Scope: {scope}"),
        format!("- Transport: {transport}"),
        format!("- Config path: {config_path}"),
    ];

    if let Some(project_path) = project_path.filter(|value| !value.is_empty()) {
        lines.push(format!("- Project path: {project_path}"));
    }
    if !endpoint.is_empty() {
        lines.push(format!("- Endpoint: {endpoint}"));
    }
    if !warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Warnings".into());
        for warning in warnings {
            lines.push(format!("- {warning}"));
        }
    }
    if !errors.is_empty() {
        lines.push(String::new());
        lines.push("## Errors".into());
        for error in errors {
            lines.push(format!("- {error}"));
        }
    }

    lines.join("\n")
}

fn mask_object_values(object: &Map<String, Value>) -> Value {
    let masked = object
        .keys()
        .map(|key| (key.clone(), Value::String("***".into())))
        .collect::<Map<_, _>>();
    Value::Object(masked)
}

fn masked_json_config(server: &Map<String, Value>) -> String {
    let mut masked = server.clone();
    if let Some(Value::Object(env)) = masked.get("env") {
        masked.insert("env".into(), mask_object_values(env));
    }
    if let Some(Value::Object(headers)) = masked.get("headers") {
        masked.insert("headers".into(), mask_object_values(headers));
    }

    serde_json::to_string_pretty(&Value::Object(masked)).unwrap_or_else(|_| "{}".into())
}

fn masked_toml_config(server: &toml::value::Table) -> String {
    let mut masked = server.clone();
    if let Some(value) = masked.get_mut("env").and_then(toml::Value::as_table_mut) {
        for (_, entry) in value.iter_mut() {
            *entry = toml::Value::String("***".into());
        }
    }
    if let Some(value) = masked
        .get_mut("http_headers")
        .and_then(toml::Value::as_table_mut)
    {
        for (_, entry) in value.iter_mut() {
            *entry = toml::Value::String("***".into());
        }
    }
    if let Some(value) = masked
        .get_mut("headers")
        .and_then(toml::Value::as_table_mut)
    {
        for (_, entry) in value.iter_mut() {
            *entry = toml::Value::String("***".into());
        }
    }

    toml::to_string_pretty(&masked).unwrap_or_else(|_| String::new())
}

fn build_local_mcp(
    target: &McpScanTargetDto,
    server_name: &str,
    scope: &str,
    config_path: &Path,
    project_path: Option<&str>,
    transport: String,
    endpoint: String,
    config: String,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> LocalMcpServerDto {
    let normalized_config_path = normalize_path(config_path);
    LocalMcpServerDto {
        id: format!(
            "{}::mcp::{}::{}",
            target.agent_id,
            sanitize_id_segment(scope),
            sanitize_id_segment(server_name)
        ),
        kind: "mcp".into(),
        name: server_name.into(),
        summary: short_summary(&transport, &endpoint, scope),
        enabled: true,
        endpoint: endpoint.clone(),
        transport: transport.clone(),
        usage_count: 0,
        updated_at: updated_at(config_path),
        document: markdown_document(
            server_name,
            scope,
            &transport,
            &endpoint,
            &normalized_config_path,
            project_path,
            &warnings,
            &errors,
        ),
        config,
        owner_agent_id: target.agent_id.clone(),
        source_label: format!("{} local", target.display_name),
        agent_type: target.agent_type.clone(),
        agent_name: target.display_name.clone(),
        config_path: normalized_config_path,
        scope: scope.into(),
        project_path: project_path.map(str::to_string),
        warnings,
        errors,
    }
}

fn scan_claude_servers(target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
    let config_path = user_home_dir().join(CLAUDE_CONFIG_FILE);
    if !config_path.exists() || !config_path.is_file() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(_) => return Vec::new(),
    };
    let value = match serde_json::from_str::<Value>(&contents) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    if let Some(servers) = value.get("mcpServers").and_then(Value::as_object) {
        for (server_name, server_value) in servers {
            let Some(server) = server_value.as_object() else {
                continue;
            };
            let transport = transport_from_config(
                server.get("type").and_then(Value::as_str),
                server.get("command").and_then(Value::as_str),
                server.get("url").and_then(Value::as_str),
            );
            let endpoint = server
                .get("url")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| server.get("command").and_then(Value::as_str).map(str::to_string))
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

    if let Some(projects) = value.get("projects").and_then(Value::as_object) {
        for (project_path, project_value) in projects {
            let Some(project) = project_value.as_object() else {
                continue;
            };
            let Some(servers) = project.get("mcpServers").and_then(Value::as_object) else {
                continue;
            };
            for (server_name, server_value) in servers {
                let Some(server) = server_value.as_object() else {
                    continue;
                };
                let transport = transport_from_config(
                    server.get("type").and_then(Value::as_str),
                    server.get("command").and_then(Value::as_str),
                    server.get("url").and_then(Value::as_str),
                );
                let endpoint = server
                    .get("url")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| server.get("command").and_then(Value::as_str).map(str::to_string))
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
            server.get("type").and_then(toml::Value::as_str),
            server.get("command").and_then(toml::Value::as_str),
            server.get("url").and_then(toml::Value::as_str),
        );
        let endpoint = server
            .get("url")
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .or_else(|| server.get("command").and_then(toml::Value::as_str).map(str::to_string))
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

fn scan_codex_servers(target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
    let root_path = resolve_agent_root(&target.root_path);
    let config_path = root_path.join(CODEX_CONFIG_FILE);
    if !config_path.exists() || !config_path.is_file() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(_) => return Vec::new(),
    };
    let value = match toml::from_str::<toml::Value>(&contents) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
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

fn scan_gemini_servers(target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
    let root_path = resolve_agent_root(&target.root_path);
    let config_path = root_path.join(GEMINI_CONFIG_FILE);
    if !config_path.exists() || !config_path.is_file() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(_) => return Vec::new(),
    };
    let value = match serde_json::from_str::<Value>(&contents) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let Some(servers) = value.get("mcpServers").and_then(Value::as_object) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for (server_name, server_value) in servers {
        let Some(server) = server_value.as_object() else {
            continue;
        };

        let explicit_type = server.get("type").and_then(Value::as_str);
        let command = server.get("command").and_then(Value::as_str);
        let http_url = server.get("httpUrl").and_then(Value::as_str);
        let url = server.get("url").and_then(Value::as_str);
        let transport = if http_url.is_some() {
            "http".to_string()
        } else if explicit_type.is_some() {
            transport_from_config(explicit_type, command, url)
        } else if command.is_some() {
            "stdio".to_string()
        } else if url.is_some() {
            "sse".to_string()
        } else {
            "unknown".to_string()
        };
        let endpoint = http_url
            .map(str::to_string)
            .or_else(|| url.map(str::to_string))
            .or_else(|| command.map(str::to_string))
            .unwrap_or_default();

        let mut normalized_server = server.clone();
        if let Some(http_url_value) = normalized_server.remove("httpUrl") {
            normalized_server.insert("url".into(), http_url_value);
        }
        if !normalized_server.contains_key("type") {
            normalized_server.insert("type".into(), Value::String(transport.clone()));
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

fn scan_opencode_servers(target: &McpScanTargetDto) -> Vec<LocalMcpServerDto> {
    let config_path = OPENCODE_CONFIG_PATH
        .iter()
        .fold(user_home_dir(), |path, segment| path.join(segment));
    if !config_path.exists() || !config_path.is_file() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&config_path) {
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
        let open_code_type = server.get("type").and_then(Value::as_str).unwrap_or("local");
        let mut normalized_server = serde_json::Map::new();
        let (transport, endpoint) = match open_code_type {
            "local" => {
                normalized_server.insert("type".into(), Value::String("stdio".into()));
                let command_parts = server.get("command").and_then(Value::as_array);
                let command = command_parts
                    .and_then(|parts| parts.first())
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if !command.is_empty() {
                    normalized_server.insert("command".into(), Value::String(command.clone()));
                }
                if let Some(args) = command_parts {
                    let normalized_args = args
                        .iter()
                        .skip(1)
                        .filter_map(Value::as_str)
                        .map(|value| Value::String(value.to_string()))
                        .collect::<Vec<_>>();
                    if !normalized_args.is_empty() {
                        normalized_server.insert("args".into(), Value::Array(normalized_args));
                    }
                }
                if let Some(environment) = server.get("environment").and_then(Value::as_object) {
                    normalized_server.insert("env".into(), Value::Object(environment.clone()));
                }
                ("stdio".to_string(), command)
            }
            "remote" => {
                normalized_server.insert("type".into(), Value::String("sse".into()));
                if let Some(url) = server.get("url").and_then(Value::as_str) {
                    normalized_server.insert("url".into(), Value::String(url.to_string()));
                }
                if let Some(headers) = server.get("headers").and_then(Value::as_object) {
                    normalized_server.insert("headers".into(), Value::Object(headers.clone()));
                }
                (
                    "sse".to_string(),
                    server
                        .get("url")
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

pub fn scan_local_mcps(scan_targets: Vec<McpScanTargetDto>) -> Vec<LocalMcpServerDto> {
    let mut grouped = BTreeMap::<String, LocalMcpServerDto>::new();

    for target in scan_targets {
        let items = match target.agent_type.as_str() {
            "claude" => scan_claude_servers(&target),
            "codex" => scan_codex_servers(&target),
            "gemini" => scan_gemini_servers(&target),
            "opencode" => scan_opencode_servers(&target),
            _ => Vec::new(),
        };

        for item in items {
            grouped.insert(item.id.clone(), item);
        }
    }

    grouped.into_values().collect()
}

pub fn count_local_mcps(agent_type: &str, root_path: &Path) -> u32 {
    let target = McpScanTargetDto {
        agent_id: "count".into(),
        agent_type: agent_type.into(),
        root_path: normalize_path(root_path),
        display_name: agent_type.into(),
    };

    match agent_type {
        "claude" => scan_claude_servers(&target).len() as u32,
        "codex" => scan_codex_servers(&target).len() as u32,
        "gemini" => scan_gemini_servers(&target).len() as u32,
        "opencode" => scan_opencode_servers(&target).len() as u32,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{count_local_mcps, scan_local_mcps};
    use crate::dto::mcp::McpScanTargetDto;
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
        std::env::temp_dir().join(format!("agent-dock-mcp-{name}-{unique}"))
    }

    #[test]
    fn scan_local_mcps_reads_claude_user_and_project_scopes() {
        let root = temp_dir("claude");
        fs::create_dir_all(root.join(".claude")).expect("create claude dir");
        fs::write(
            root.join(".claude.json"),
            r#"{
  "mcpServers": {
    "docs": {
      "type": "http",
      "url": "https://example.com/mcp",
      "headers": {
        "Authorization": "secret"
      }
    }
  },
  "projects": {
    "/workspace/demo": {
      "mcpServers": {
        "filesystem": {
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
        .expect("write claude config");
        let previous_test_home = std::env::var_os("AGENT_DOCK_TEST_HOME");
        unsafe {
            std::env::set_var("AGENT_DOCK_TEST_HOME", &root);
        }

        let items = scan_local_mcps(vec![McpScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: root.join(".claude").to_string_lossy().replace('\\', "/"),
            display_name: "Claude Code".into(),
        }]);

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.scope == "user"));
        assert!(items.iter().any(|item| item.scope == "local"));
        assert!(items.iter().all(|item| !item.config.contains("secret")));

        match previous_test_home {
            Some(value) => unsafe { std::env::set_var("AGENT_DOCK_TEST_HOME", value) },
            None => unsafe { std::env::remove_var("AGENT_DOCK_TEST_HOME") },
        }
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn scan_local_mcps_reads_codex_server_tables() {
        let root = temp_dir("codex");
        fs::create_dir_all(root.join(".codex")).expect("create codex dir");
        fs::write(
            root.join(".codex/config.toml"),
            r#"[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"

[mcp_servers.filesystem]
command = "docker"
args = ["mcp", "gateway", "run"]
[mcp_servers.filesystem.env]
API_KEY = "secret"

[mcp_servers.remote]
type = "http"
url = "https://example.com/mcp"
[mcp_servers.remote.http_headers]
Authorization = "secret"
"#,
        )
        .expect("write codex config");

        let items = scan_local_mcps(vec![McpScanTargetDto {
            agent_id: "agent-codex".into(),
            agent_type: "codex".into(),
            root_path: root.join(".codex").to_string_lossy().replace('\\', "/"),
            display_name: "Codex CLI".into(),
        }]);

        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|item| item.transport == "http"));
        assert!(items.iter().any(|item| item.transport == "stdio"));
        assert!(items.iter().all(|item| !item.config.contains("secret")));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn count_local_mcps_returns_scanned_count_for_supported_agent() {
        let root = temp_dir("count-codex");
        let config_root = root.join(".codex");
        fs::create_dir_all(&config_root).expect("create codex dir");
        fs::write(
            config_root.join("config.toml"),
            r#"[mcp_servers.docs]
url = "https://developers.openai.com/mcp"
"#,
        )
        .expect("write codex config");

        assert_eq!(count_local_mcps("codex", &config_root), 1);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn scan_local_mcps_reads_gemini_servers_with_transport_inference() {
        let root = temp_dir("gemini");
        fs::create_dir_all(root.join(".gemini")).expect("create gemini dir");
        fs::write(
            root.join(".gemini/settings.json"),
            r#"{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem"],
      "env": {
        "TOKEN": "secret"
      }
    },
    "remote-http": {
      "httpUrl": "https://example.com/http-mcp",
      "headers": {
        "Authorization": "secret"
      }
    },
    "remote-sse": {
      "url": "https://example.com/sse"
    }
  }
}"#,
        )
        .expect("write gemini config");

        let items = scan_local_mcps(vec![McpScanTargetDto {
            agent_id: "agent-gemini".into(),
            agent_type: "gemini".into(),
            root_path: root.join(".gemini").to_string_lossy().replace('\\', "/"),
            display_name: "Gemini CLI".into(),
        }]);

        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|item| item.transport == "stdio"));
        assert!(items.iter().any(|item| item.transport == "http"));
        assert!(items.iter().any(|item| item.transport == "sse"));
        assert!(items.iter().all(|item| !item.config.contains("secret")));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn scan_local_mcps_reads_opencode_servers_from_json5_config() {
        let root = temp_dir("opencode");
        let config_root = root.join(".config/opencode");
        fs::create_dir_all(&config_root).expect("create opencode dir");
        fs::write(
            config_root.join("opencode.json"),
            r#"{
  // OpenCode MCP config
  mcp: {
    filesystem: {
      type: "local",
      command: ["npx", "-y", "@modelcontextprotocol/server-filesystem"],
      environment: {
        TOKEN: "secret",
      },
    },
    remoteDocs: {
      type: "remote",
      url: "https://example.com/mcp",
      headers: {
        Authorization: "secret",
      },
    },
  },
}"#,
        )
        .expect("write opencode config");
        let previous_test_home = std::env::var_os("AGENT_DOCK_TEST_HOME");
        unsafe {
            std::env::set_var("AGENT_DOCK_TEST_HOME", &root);
        }

        let items = scan_local_mcps(vec![McpScanTargetDto {
            agent_id: "agent-opencode".into(),
            agent_type: "opencode".into(),
            root_path: root.join(".opencode").to_string_lossy().replace('\\', "/"),
            display_name: "OpenCode".into(),
        }]);

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.transport == "stdio"));
        assert!(items.iter().any(|item| item.transport == "sse"));
        assert!(items.iter().all(|item| !item.config.contains("secret")));

        match previous_test_home {
            Some(value) => unsafe { std::env::set_var("AGENT_DOCK_TEST_HOME", value) },
            None => unsafe { std::env::remove_var("AGENT_DOCK_TEST_HOME") },
        }
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
