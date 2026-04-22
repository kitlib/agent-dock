//! MCP处理公共辅助函数
use std::path::Path;

use serde_json::Value;

use crate::constants::*;
use crate::dto::mcp::{LocalMcpServerDto, McpScanTargetDto};
use crate::infrastructure::utils::path::normalize_path;

pub fn transport_from_config(
    explicit_type: Option<&str>,
    command: Option<&str>,
    url: Option<&str>,
) -> String {
    if let Some(transport) = explicit_type.filter(|value| !value.trim().is_empty()) {
        return transport.to_string();
    }
    if command.is_some() {
        return TRANSPORT_STDIO.into();
    }
    if let Some(endpoint) = url {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            return TRANSPORT_HTTP.into();
        }
        return TRANSPORT_REMOTE.into();
    }
    TRANSPORT_UNKNOWN.into()
}

pub fn short_summary(transport: &str, endpoint: &str, scope: &str) -> String {
    match transport {
        TRANSPORT_STDIO => format!("Configured as a {scope} stdio MCP server."),
        TRANSPORT_HTTP | TRANSPORT_SSE => format!("Configured as a {scope} remote MCP server."),
        _ => {
            if endpoint.is_empty() {
                format!("Configured as a {scope} MCP server.")
            } else {
                format!("Configured as a {scope} MCP server via {transport}.")
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn markdown_document(
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

pub fn mask_object_values(object: &serde_json::Map<String, Value>) -> Value {
    let masked = object
        .keys()
        .map(|key| (key.clone(), Value::String("***".into())))
        .collect::<serde_json::Map<_, _>>();
    Value::Object(masked)
}

pub fn masked_json_config(server: &serde_json::Map<String, Value>) -> String {
    let mut masked = server.clone();
    if let Some(Value::Object(env)) = masked.get(FIELD_ENV) {
        masked.insert(FIELD_ENV.into(), mask_object_values(env));
    }
    if let Some(Value::Object(headers)) = masked.get(FIELD_HEADERS) {
        masked.insert(FIELD_HEADERS.into(), mask_object_values(headers));
    }

    serde_json::to_string_pretty(&Value::Object(masked)).unwrap_or_else(|_| "{}".into())
}

pub fn masked_toml_config(server: &toml::value::Table) -> String {
    let mut masked = server.clone();
    if let Some(value) = masked.get_mut(FIELD_ENV).and_then(toml::Value::as_table_mut) {
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
        .get_mut(FIELD_HEADERS)
        .and_then(toml::Value::as_table_mut)
    {
        for (_, entry) in value.iter_mut() {
            *entry = toml::Value::String("***".into());
        }
    }

    toml::to_string_pretty(&masked).unwrap_or_else(|_| String::new())
}

#[allow(clippy::too_many_arguments)]
pub fn build_local_mcp(
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

fn updated_at(path: &Path) -> String {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .map(chrono::DateTime::<chrono::Utc>::from)
        .map(|datetime| datetime.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string())
}
