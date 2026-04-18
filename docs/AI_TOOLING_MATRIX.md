# AI 工具配置目录速查表

本文档整理常见 AI 编码工具的配置目录、根文件以及规则、命令、代理、技能和 MCP 配置位置，便于快速查阅和对比。

## 配置矩阵

| Platform | Directory | Root file | Rules | Commands | Agents | Skills | MCP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AdaL | `.adal/` |  |  |  |  | `skills/` | `settings.json` |
| Amp | `.agents/` |  | `checks/` |  |  | `skills/` | `.amp/settings.json` |
| Antigravity | `.agent/` |  | `rules/` | `workflows/` |  |  |  |
| Augment Code | `.augment/` |  | `rules/` | `commands/` |  | `skills/` |  |
| Claude Code | `.claude/` | `CLAUDE.md` | `rules/` | `commands/` | `agents/` | `skills/` | `~/.claude.json`（user）, `.mcp.json`（project root） |
| Claude Code Plugin | `.claude-plugin/` |  | `rules/` | `commands/` | `agents/` | `skills/` | `.mcp.json`（project root） |
| Cline | `.cline/` |  |  |  |  | `skills/` | `cline_mcp_settings.json` |
| CodeBuddy | `.codebuddy/` |  | `rules/` |  |  | `skills/` |  |
| Codex CLI | `.codex/` |  |  | `prompts/` |  | `skills/` | `~/.codex/config.toml` |
| Command Code | `.commandcode/` |  |  | `commands/` | `agents/` | `skills/` |  |
| Continue | `.continue/` |  | `rules/` | `prompts/` |  | `skills/` |  |
| Crush | `.config/crush/` |  |  |  |  | `skills/` | `crush.json` |
| Cursor | `.cursor/` |  | `rules/` | `commands/` | `agents/` | `skills/` | `~/.cursor/mcp.json` |
| Factory AI | `.factory/` |  |  | `commands/` | `droids/` | `skills/` | `settings/mcp.json` |
| Gemini CLI | `.gemini/` | `GEMINI.md` |  |  |  | `skills/` | `~/.gemini/settings.json` |
| GitHub Copilot | `.github/` |  |  |  | `agents/` | `skills/` |  |
| Goose | `.goose/` |  |  |  |  | `skills/` | `config.yaml` |
| iFlow CLI | `.iflow/` | `IFLOW.md` |  | `commands/` | `agents/` | `skills/` | `settings.json` |
| Junie | `.junie/` |  |  |  |  | `skills/` |  |
| Kilo Code | `.kilocode/` |  | `rules/` | `workflows/` |  | `skills/` | `mcp.json` |
| Kimi Code CLI | `.agents/` |  |  |  |  | `skills/` | `.kimi/mcp.json` |
| Kiro | `.kiro/` |  | `steering/` |  |  | `skills/` | `settings/mcp.json` |
| Kode | `.kode/` |  |  |  |  | `skills/` |  |
| MCPJam | `.mcpjam/` |  |  |  |  | `skills/` |  |
| Mistral Vibe | `.vibe/` |  |  |  |  | `skills/` |  |
| Mux | `.mux/` |  |  |  |  | `skills/` |  |
| Neovate | `.neovate/` |  |  |  |  | `skills/` | `mcp.json` |
| OpenClaw | `.openclaw/` |  |  |  |  | `skills/` |  |
| OpenCode | `.opencode/` |  |  | `commands/` | `agents/` | `skills/` | `~/.config/opencode/opencode.json` |
| OpenHands | `.openhands/` |  |  |  |  | `skills/` | `mcp.json` |
| Pi-Mono | `.pi/` |  |  | `agent/prompts/` |  | `agent/skills/` |  |
| Pochi | `.pochi/` |  |  |  |  | `skills/` |  |
| Qoder | `.qoder/` |  | `rules/` | `commands/` | `agents/` | `skills/` |  |
| Qwen Code | `.qwen/` | `QWEN.md` |  |  | `agents/` | `skills/` | `settings.json` |
| Replit | `.agents/` | `replit.md` |  |  |  | `skills/` |  |
| Roo Code | `.roo/` |  |  | `commands/` |  | `skills/` | `mcp.json` |
| Trae | `.trae/` |  | `rules/` |  |  | `skills/` |  |
| Trae CN | `.trae-cn/` |  | `rules/` |  |  | `skills/` |  |
| Warp | `.warp/` | `WARP.md` |  |  |  |  |  |
| Windsurf | `.windsurf/` |  | `rules/` |  |  | `skills/` |  |
| Zencoder | `.zencoder/` |  |  |  |  | `skills/` |  |

## 与当前代码实现直接相关的 MCP 路径

以下路径已经被 AgentDock 当前代码真实支持：

- Claude Code：`~/.claude.json`
- Codex CLI：`~/.codex/config.toml`
- Gemini CLI：`~/.gemini/settings.json`
- OpenCode：`~/.config/opencode/opencode.json`

说明：

- Claude Code 的 `.mcp.json` 是项目级 MCP 配置位置，但 AgentDock 当前尚未实现项目根扫描。
- Cursor 在官方文档中使用 `~/.cursor/mcp.json`，但 AgentDock 当前还未实现本地发现。
