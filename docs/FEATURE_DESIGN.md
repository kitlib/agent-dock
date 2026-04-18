# AgentDock 多 Agent 资源管理功能设计

## 功能定位

AgentDock 当前的核心目标是成为一个面向多 Agent 的本地资源管理中心。

当前产品重点：

- 管理本地 Agent
- 浏览和维护本地 Skill
- 在统一工作区内承载 Skills、MCP、Subagents 三类资源
- 通过 Marketplace 作为补充导入入口

当前产品决策：

- `Home / Agents` 是当前唯一主工作台
- 不再规划独立 `Resources` 页面
- Marketplace 已对 `skills.sh` 打通真实链路，但仅覆盖 Skill

## 当前代码实现现状（基于代码核对，2026-04-18）

### 1. Agents / Home 工作台

当前已实现：

- 左侧 Agent Rail 支持折叠
- 左侧顶部提供“全部”入口
- 选中“全部”时，可在 Home 内查看所有已管理且未隐藏 Agent 的聚合资源
- 选中单个 Agent 时，可查看该 Agent 的本地资源
- 工作区支持 `browse / adding` 两种模式切换

当前边界：

- “全部”聚合当前主要用于 Skill
- MCP 已具备真实本地发现与管理能力，但还没有统一的跨 Agent 聚合视图
- Subagent 仍主要是占位资源模型

### 2. 本地 Skill 发现与管理

当前已实现：

- 扫描 `skills/` 与 `commands/` 两类来源，并统一映射为 Skill 资源
- 本地 Skill 列表
- 本地 Skill 详情
- 启用 / 停用
- 打开 Skill 目录
- 打开 Skill 入口文件
- 删除单个本地 Skill
- 在 `All Agents` 与单 Agent 视图之间复用同一套 Skill 浏览链路

补充说明：

- `commands` 类型的 Markdown 文件也会被纳入 Skill 资源视图
- 启用 / 停用通过入口文件与 `.disabled` 文件名切换完成
- 删除会同步移除 Marketplace 安装记录

### 3. Skill 解析能力

当前后端已解析：

- `description`
- `name / title`
- `tags`
- `warnings / errors`
- `marketplaceSource / marketplaceSkillId`

当前前端已展示：

- 标题
- 描述
- `SKILL.md` Markdown
- Diagnostics
- Marketplace 更新按钮

结论：

- 后端已经去掉未被消费的冗余结构化解析字段
- Skill 详情当前聚焦真正被使用的可见信息

### 4. Skill 复制链路

当前已实现：

- 批量复制本地 Skill 到其他 Agent
- 单个 Skill 从列表“更多”菜单发起复制
- 复制目标改为平铺 Agent 卡片，而不是下拉框
- 支持多选目标 Agent
- 支持冲突预览
- 支持 `overwrite / skip`
- 复制完成后刷新本地 Skill 列表和 Agent 列表

当前边界：

- 复制完成后的反馈仍较轻量，主要是 toast
- 尚未提供“跳转到目标 Agent / 目标 Skill”的定位反馈

### 5. MCP 本地管理

当前已实现：

- 真实 MCP 本地发现，不再使用前端 mock
- 真实 MCP 数量统计
- Home 工作区展示本地 MCP 列表
- MCP 详情展示真实读取结果
- 打开 MCP 配置目录
- 打开 MCP 配置文件
- 删除本地 MCP 配置项
- 粘贴 JSON 导入 MCP
- 不支持 MCP 的 Agent 在中栏显示 `暂未支持`

当前已支持的 Agent：

- Claude Code
  - 读取 `~/.claude.json`
  - 支持顶层 `mcpServers`
  - 支持 `projects.<path>.mcpServers`
- Codex CLI
  - 读取 `~/.codex/config.toml`
  - 支持 `mcp_servers`
  - 兼容 `mcp.servers`
- Gemini CLI
  - 读取 `~/.gemini/settings.json`
  - 解析 `mcpServers`
- OpenCode
  - 读取 `~/.config/opencode/opencode.json`
  - 使用 `json5` 解析
  - 支持 `local / remote` 到内部模型的转换

当前详情右栏已展示：

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

当前边界：

- 尚未支持 MCP 编辑
- 尚未支持 MCP 启用 / 停用
- 尚未支持 MCP 运行时握手与探测
- 尚未支持 Claude 项目根 `.mcp.json` 扫描
- 尚未支持 Cursor / Cline / Goose 等更多 Agent
- 尚未支持 MCP Marketplace

### 6. Marketplace（Skill）

当前已实现：

- `skills.sh` 排行榜拉取
- `skills.sh` 搜索
- Marketplace Skill 详情拉取
- Marketplace Skill 安装预览
- Marketplace Skill 安装到指定 Agent
- Marketplace Skill 覆盖安装
- Marketplace 安装完成后刷新本地 Skill 池
- 本地 Marketplace Skill 更新检查
- 本地 Skill 详情页直接执行更新
- Marketplace 详情缓存
- Marketplace 安装记录持久化

当前边界：

- 真实 Marketplace 仅覆盖 Skill
- `subagent` 仍使用前端 mock 数据
- MCP 尚未接入 Marketplace
- Source 管理、健康检查、配置页尚未完成

结论：

- “Marketplace 真接入未完成”这个旧判断不再准确
- 更准确的描述应是：“Skill Marketplace 已接入，统一 Marketplace 仍未完成”

### 7. 列表与详情操作顺序

当前已实现：

- 列表“更多”菜单顺序统一为：打开、编辑、复制、停用 / 启用、删除
- 右侧详情操作顺序统一为：访问 / 安装、更新、打开、编辑、停用 / 启用、删除
- 详情侧删除已有确认弹窗
- MCP 删除在列表和详情都可触发

当前边界：

- 某些资源类型在列表中的删除确认仍不完全一致

## 核心功能结构

### 1. 多 Agent 管理

支持：

- Agent 列表展示
- 扫描发现 Agent
- 导入 Agent
- 手动创建 Agent
- 删除 Agent
- 移除已管理 Agent
- 切换当前 Agent
- 查看每个 Agent 的资源概览

### 2. 本地资源池管理

统一目标模型：

- Skills
- MCP
- Subagents

当前真实实现：

- Skill 已有真实扫描、详情、管理、Marketplace 安装与更新链路
- MCP 已有真实本地发现、详情、打开配置与删除链路
- Subagent 仍主要是占位模型与展示壳

### 3. Agent 资源绑定管理

当前已实现：

- 通过复制将 Skill 放入目标 Agent
- 对复制冲突执行预览与决策

当前未实现：

- 显式“绑定关系”模型
- 从 Agent 解绑资源
- 绑定顺序调整
- “被哪些 Agent 使用”的关系视图

### 4. 分组管理

规划支持但尚未实现：

- 创建分组
- 重命名分组
- 删除分组
- 折叠 / 展开分组
- 分组排序

### 5. 拖拽编排

当前现状：

- 列表项已有 `draggable`

尚未实现：

- drop 目标
- 实际排序逻辑
- 拖入分组
- 跨组移动

## 页面结构

### Agents / Home

当前唯一主工作台，用于：

- 切换 Agent
- 查看当前 Agent 的资源
- 查看“全部 Skills”聚合视图
- 执行启停、复制、删除、Marketplace 安装与更新
- 查看、导入和删除本地 MCP

### Marketplace

当前仍嵌入 Home 资源浏览器中，不是独立真实编排页。

已具备：

- 浏览 Skill Marketplace
- 搜索 Skill Marketplace
- 查看 Skill 详情
- 安装到本地 Agent

未具备：

- 独立的 Marketplace Source 管理
- 统一三类资源的真实 Marketplace

## 当前阶段结论

已落地：

- 多 Agent 基础管理
- 本地 Skill 扫描与展示
- Skill 启停、打开、删除、复制
- Home 内“全部 Skills”聚合视图
- `skills.sh` Marketplace 的列表、搜索、详情、安装、更新检查
- MCP 的真实本地发现、详情、打开配置、删除

未完成：

- Skill 使用关系视图
- 批量删除
- 复制结果反馈增强
- 分组管理
- 拖拽编排
- Subagent 的真实本地与 Marketplace 管理链路
- MCP 的编辑、启停、运行时探测、Marketplace，以及更多 Agent 支持

## 使用原则

- 日常主操作优先在 `Home / Agents` 中完成
- “全部”视图当前优先服务 Skill 聚合
- Marketplace 当前是 Skill 的真实获取入口，不承担完整资源编排职责
- 删除绑定与删除资源本体必须明确区分
- 危险删除操作应统一具备确认
