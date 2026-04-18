<div align="center">

# AgentDock

[English](./README.md) | 简体中文

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)

一个用于浏览、管理和编排本地 AI 工具资源的桌面应用，面向多种 Agent 生态。

</div>

## 预览

![应用截图](./screenshots/app.png)

## 特点

- 🤖 **本地 Agent 管理** - 发现、导入、创建、移除并查看不同 AI 工具体系下的本地 Agent
- 🧩 **Skill 发现与维护** - 扫描 `skills/` 与 `commands/`，查看详情，启停 Skill，打开文件，删除并跨 Agent 复制
- 🛒 **Skill Marketplace 集成** - 通过 `skills.sh` 浏览、搜索、查看详情、安装和更新 Marketplace Skill
- 🔌 **本地 MCP 管理** - 支持 `Claude Code`、`Codex CLI`、`Gemini CLI`、`OpenCode` 的本地 MCP 发现与管理
- 📥 **MCP JSON 导入** - 粘贴 MCP JSON，预览冲突后导入到当前支持的本地 Agent 配置
- 🏠 **统一工作区体验** - 以单一 `Home / Agents` 工作区承载 Agent、Skill、MCP 等资源操作
- 🔔 **桌面壳能力** - 系统托盘、全局快捷键、单实例行为以及更新相关桌面流程
- 🌍 **国际化支持** - 基于 i18next 提供中英文界面

## 技术栈

- **桌面框架**: [Tauri v2](https://tauri.app/)
- **前端框架**: [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/)
- **构建工具**: [Vite](https://vite.dev/)
- **UI 组件**: [shadcn/ui](https://ui.shadcn.com/)
- **样式方案**: [Tailwind CSS v4](https://tailwindcss.com/)
- **代码格式化**: [Prettier](https://prettier.io/)

## 开始使用

### 环境要求

- Node.js >= 18
- pnpm >= 9
- Rust >= 1.70

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建发布

```bash
pnpm tauri build
```

### 版本管理

`pnpm release:version` 是版本发布的唯一入口。

```bash
pnpm release:version
pnpm release:version --lang zh
pnpm release:version --lang en
```

它会交互式完成发布前检查和版本更新流程：

- 确保工作区干净
- 强制要求当前分支为 `main`
- 校验 `package.json`、`src-tauri/tauri.conf.json` 和 `src-tauri/Cargo.toml` 的版本一致
- 检查目标 tag 是否已在本地或远端 `origin` 存在
- 同步更新这三个版本文件
- 创建发布提交和 `vX.Y.Z` tag
- 可选地推送分支和 tag

## 添加 shadcn/ui 组件

```bash
pnpm dlx shadcn@latest add <component-name>
```

示例：

```bash
pnpm dlx shadcn@latest add button
pnpm dlx shadcn@latest add input
pnpm dlx shadcn@latest add dialog
```

## 代码格式化

```bash
pnpm format        # 格式化代码
pnpm format:check  # 检查代码格式
```

## 质量检查

```bash
pnpm lint
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## 项目结构

```
.
├── src/                           # 前端源码
│   ├── components/                # 共享 React 组件
│   │   └── ui/                    # shadcn/ui 封装
│   ├── features/                  # 领域功能模块
│   │   ├── agents/                # Agent 发现与管理
│   │   ├── home/                  # 工作区与资源浏览
│   │   ├── marketplace/           # skills.sh 集成
│   │   └── resources/             # 共享资源渲染
│   ├── i18n/                      # 国际化
│   ├── pages/                     # 窗口页面
│   └── main.tsx                   # 前端入口与基于 pathname 的页面选择器
├── src-tauri/                     # Tauri/Rust 后端
│   ├── src/commands/              # Tauri 命令层
│   ├── src/scanners/              # 本地发现扫描器
│   ├── src/services/              # 领域编排逻辑
│   ├── src/persistence/           # 本地持久化状态
│   └── tauri.conf.json            # Tauri 配置
├── docs/                          # 产品与实现文档
├── components.json                # shadcn/ui 配置
└── package.json
```

## CI/CD

本项目使用 GitHub Actions 实现自动化构建和发布。

### 自动化发布

工作流会在推送符合 `v*` 格式的标签时触发，例如 `v0.1.0`。
推荐通过 `pnpm release:version` 发版，它会自动创建匹配的 `vX.Y.Z` tag。

**手动创建并推送标签示例：**

```bash
git tag v0.1.0
git push origin v0.1.0
```

### 构建产物

工作流会生成：

- **NSIS 安装包** - Windows 安装程序
- **更新文件** - `latest.json` 用于自动更新支持

### 自动更新配置

要启用自动更新功能，需要：

1. 生成签名密钥：`pnpm tauri signer generate -w ~/.tauri/myapp.key`
2. 添加 GitHub Secrets：`TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

**注意：** `src-tauri/tauri.conf.json` 中的公钥和更新端点占位符会在发布构建期间由 GitHub Actions 自动替换。自动更新依赖已发布的 GitHub Release 对外提供最新版本的 `latest.json` 资源。

详细配置说明请查看 [自动更新配置文档](./docs/AUTO_UPDATE.zh-CN.md)。

### 代码签名（可选）

如需启用代码签名，在 GitHub 仓库设置中添加以下 Secrets：

- `TAURI_SIGNING_PRIVATE_KEY` - 私钥内容
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - 私钥密码

不配置这些 Secrets 也能正常构建，只是安装包不会被签名。

### 多平台支持

如需启用 macOS 和 Linux 构建，取消 `.github/workflows/release.yml` 中对应平台配置的注释即可。

## IDE 推荐

- [VS Code](https://code.visualstudio.com/)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=kitlib/agent-dock&type=Date)](https://star-history.com/#kitlib/agent-dock&Date)
