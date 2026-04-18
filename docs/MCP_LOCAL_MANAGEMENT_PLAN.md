# MCP 本地管理实现现状

## 目标

AgentDock 当前的 MCP 方案不是完整 MCP client，而是优先完成本地配置发现和只读管理，把 Home 工作区里的 MCP 资源从前端 mock 替换为真实本地数据。

当前已落地的支持范围：

- Claude Code
- Codex CLI
- Gemini CLI
- OpenCode

## 已实现能力

### 1. 真实本地发现

后端已经具备 MCP 本地扫描链路：

- `src-tauri/src/scanners/mcp_scanner.rs`
- `src-tauri/src/services/mcp_discovery_service.rs`
- `src-tauri/src/commands/mcp.rs`

前端已经接入真实查询链路，Home 工作区中的 MCP 列表不再依赖 mock 数据。

### 2. 已支持的配置文件

#### Claude Code

- 用户级配置：`~/.claude.json`
- 已解析顶层 `mcpServers`
- 已解析 `projects.<path>.mcpServers`
- 支持区分 `user` 和 `project-local` scope

说明：
- 当前没有做任意项目根目录 `.mcp.json` 的全盘扫描
- 只读取当前用户配置文件中能发现的 Claude MCP 定义

#### Codex CLI

- 配置文件：`~/.codex/config.toml`
- 支持官方格式：`[mcp_servers.<name>]`
- 兼容旧格式：`[mcp.servers.<name>]`
- 支持读取 `http_headers`

#### Gemini CLI

- 配置文件：`~/.gemini/settings.json`
- 读取 `mcpServers`
- transport 推断规则：
  - `command` -> `stdio`
  - `httpUrl` -> `http`
  - `url` -> `sse`

#### OpenCode

- 配置文件：`~/.config/opencode/opencode.json`
- 使用 `json5` 解析，兼容注释和尾逗号
- 支持格式归一化：
  - `local` -> `stdio`
  - `remote` -> `sse`

## 当前 UI 能力

### 1. MCP 列表

Home 工作区已经支持：

- 展示真实 MCP 数量
- 按 Agent 查看本地 MCP
- 在不支持 MCP 的 Agent 上显示 `暂未支持`
- 列表中打开配置目录
- 列表中打开配置文件
- 列表中删除本地 MCP
- 粘贴 JSON 导入 MCP

### 2. MCP 详情

右栏详情已经改为优先展示实际读取结果，而不是仅显示说明文案。

当前详情内容包括：

- Summary
- Read Result
  - Transport
  - Source Agent
  - Scope
  - Updated
  - Endpoint
  - Project Path
  - Config Path
- Notes
- Masked Config
- Diagnostics

### 3. 敏感信息处理

默认会对敏感字段做脱敏展示，当前覆盖：

- `env`
- `headers`
- `http_headers`

## 已实现写操作

当前 MCP 管理不是完全只读，已经支持两类写操作：

- 删除本地 MCP 配置项
- 粘贴 JSON 导入 MCP

已支持删除的 Agent：

- Claude Code
- Codex CLI
- Gemini CLI
- OpenCode

删除行为说明：

- Claude / Gemini：删除 `mcpServers.<name>`
- Codex：同时清理 `mcp_servers` 和兼容格式 `mcp.servers`
- OpenCode：删除对应 MCP 条目后写回 `opencode.json`

删除入口已接入：

- 列表更多菜单
- 右侧详情面板

删除后会自动刷新 MCP 列表。

## 当前边界

以下能力尚未实现：

- MCP 编辑
- MCP 启用 / 停用
- MCP 运行时握手与探测
- tools / resources / prompts 运行时枚举
- Claude 项目根 `.mcp.json` 工作区扫描
- Cursor / Cline / Goose 等更多 Agent 的 MCP 发现
- MCP Marketplace

## 当前实现判断

现在对 MCP 功能的准确描述应为：

- 已完成第一版真实本地 MCP 管理
- 范围是配置发现、列表展示、详情查看、打开配置、删除本地配置、JSON 导入
- 尚未进入运行时探测、编辑写回和 Marketplace 阶段

## 相关代码

- 后端：
  - `src-tauri/src/dto/mcp.rs`
  - `src-tauri/src/scanners/mcp_scanner.rs`
  - `src-tauri/src/services/mcp_discovery_service.rs`
  - `src-tauri/src/commands/mcp.rs`
- 前端：
  - `src/features/agents/agent-meta.ts`
  - `src/features/agents/api.ts`
  - `src/features/home/queries.ts`
  - `src/features/home/use-home-workspace.ts`
  - `src/features/resources/core/components/resource-list.tsx`
  - `src/features/resources/core/components/resource-detail.tsx`
  - `src/features/home/components/detail-panel.tsx`

## 后续优先级建议

建议按下面顺序继续推进：

1. Cursor MCP 本地发现
2. Claude 项目级 `.mcp.json` 扫描
3. MCP 编辑与启停
4. MCP 运行时探测
