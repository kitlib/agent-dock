//! MCP Handler工厂类
use crate::constants::AgentType;
use super::McpConfigHandler;
use super::claude::ClaudeHandler;
use super::codex::CodexHandler;
use super::gemini::GeminiHandler;
use super::opencode::OpenCodeHandler;

/// 根据Agent类型创建对应的MCP配置处理器
pub fn create_mcp_handler(agent_type: AgentType) -> Box<dyn McpConfigHandler> {
    match agent_type {
        AgentType::Claude => Box::new(ClaudeHandler),
        AgentType::Codex => Box::new(CodexHandler),
        AgentType::Gemini => Box::new(GeminiHandler),
        AgentType::OpenCode => Box::new(OpenCodeHandler),
    }
}
