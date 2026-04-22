//! MCP配置处理模块入口
use std::collections::BTreeMap;
use std::path::Path;

use crate::constants::AgentType;
use crate::dto::mcp::{
    EditableLocalMcpDto, ImportLocalMcpResultDto, ImportedMcpServer, LocalMcpServerDto,
    McpImportConflictStrategy, McpScanTargetDto, UpdateLocalMcpResultDto,
};

pub mod claude;
pub mod codex;
pub mod common;
pub mod gemini;
pub mod opencode;

pub trait McpConfigHandler: Send + Sync {
    /// 扫描该Agent配置下的所有MCP服务器
    fn scan_servers(&self, target: &McpScanTargetDto) -> Vec<LocalMcpServerDto>;

    /// 读取单个MCP服务器编辑数据
    fn read_server(
        &self,
        config_path: &Path,
        server_name: &str,
        scope: &str,
        project_path: Option<&str>,
    ) -> Result<EditableLocalMcpDto, String>;

    /// 写入/更新MCP服务器配置
    fn write_server(
        &self,
        config_path: &Path,
        old_name: &str,
        new_name: &str,
        server: &ImportedMcpServer,
        scope: &str,
        project_path: Option<&str>,
    ) -> Result<UpdateLocalMcpResultDto, String>;

    /// 删除MCP服务器配置
    fn delete_server(
        &self,
        config_path: &Path,
        server_name: &str,
        scope: &str,
        project_path: Option<&str>,
    ) -> Result<(), String>;

    /// 批量导入MCP服务器配置
    fn import_servers(
        &self,
        config_path: &Path,
        servers: &BTreeMap<String, ImportedMcpServer>,
        conflict_strategy: McpImportConflictStrategy,
    ) -> Result<ImportLocalMcpResultDto, String>;
}

/// 根据Agent类型创建对应的MCP配置处理器
pub fn create_mcp_handler(agent_type: AgentType) -> Box<dyn McpConfigHandler> {
    match agent_type {
        AgentType::Claude => Box::new(claude::ClaudeHandler),
        AgentType::Codex => Box::new(codex::CodexHandler),
        AgentType::Gemini => Box::new(gemini::GeminiHandler),
        AgentType::OpenCode => Box::new(opencode::OpenCodeHandler),
    }
}
