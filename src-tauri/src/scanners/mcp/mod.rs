//! MCP配置处理模块入口
pub mod traits;
pub mod factory;
pub mod common;
pub mod claude;
pub mod codex;
pub mod gemini;
pub mod opencode;

// 公共导出
pub use traits::McpConfigHandler;
pub use factory::create_mcp_handler;
