use crate::dto::mcp::{LocalMcpServerDto, McpScanTargetDto};
use crate::scanners::mcp_scanner;

pub fn list_local_mcps(scan_targets: Vec<McpScanTargetDto>) -> Vec<LocalMcpServerDto> {
    mcp_scanner::scan_local_mcps(scan_targets)
}
