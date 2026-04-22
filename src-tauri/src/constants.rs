//! Global constants
use std::fmt;
use std::str::FromStr;

/// Supported agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    OpenCode,
}

impl FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            "opencode" => Ok(Self::OpenCode),
            _ => Err(format!("Unsupported agent type: {}", s)),
        }
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
        };
        write!(f, "{}", s)
    }
}

// MCP configuration filenames
pub const CLAUDE_CONFIG_FILE: &str = ".claude.json";
pub const CODEX_CONFIG_FILE: &str = "config.toml";
pub const GEMINI_CONFIG_FILE: &str = "settings.json";
pub const OPENCODE_CONFIG_FILE: &str = "opencode.json";
pub const OPENCODE_CONFIG_PATH: [&str; 3] = [".config", "opencode", OPENCODE_CONFIG_FILE];

// MCP transport types
pub const TRANSPORT_STDIO: &str = "stdio";
pub const TRANSPORT_HTTP: &str = "http";
pub const TRANSPORT_SSE: &str = "sse";
pub const TRANSPORT_REMOTE: &str = "remote";
pub const TRANSPORT_UNKNOWN: &str = "unknown";


// MCP configuration field names
pub const FIELD_MCP_SERVERS: &str = "mcpServers";
pub const FIELD_TYPE: &str = "type";
pub const FIELD_COMMAND: &str = "command";
pub const FIELD_ARGS: &str = "args";
pub const FIELD_ENV: &str = "env";
pub const FIELD_URL: &str = "url";
pub const FIELD_HTTP_URL: &str = "httpUrl";
pub const FIELD_HEADERS: &str = "headers";
