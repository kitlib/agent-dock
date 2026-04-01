# Subagents 开发指南

## 1. 目标与适用范围

本文用于为 AgentDock 后续的 Subagents 相关开发提供统一依据，覆盖以下目标：

- 统一 Subagents 的核心概念与边界认知
- 明确 Claude Code 中 Subagents 的配置模型
- 结合当前项目已有代码结构，确定前后端的实现落点
- 为 Subagent 列表、详情展示和后续编辑能力提供数据模型依据
- 为详情面板的信息架构、展示分区和诊断规则提供设计依据

本文主要服务于以下开发场景：

- 本地 Subagents 发现与扫描
- Subagent 基础信息展示
- Subagent 详情面板展示
- Subagent DTO 与前后端接口设计
- Subagent 配置解析、诊断与容错

---

## 2. 核心概念

### 2.1 什么是 Subagent

Subagent 是主 Agent 用来处理专项任务的子代理。

它的核心特点是：

- 运行在独立上下文中
- 可以配置独立 prompt
- 可以限制工具访问范围
- 可以指定模型与权限模式
- 可以在完成后仅把结果摘要返回给主 Agent

在产品定位上，Subagent 适合承担高噪声、可总结、可并行的子任务，例如：

- 代码库探索
- 文档检索
- 日志分析
- 测试执行
- 配置检查

### 2.2 与相近概念的区别

#### Subagents vs Skills

- Skills 更偏向可复用的提示模板与工作流说明
- Subagents 是真正独立执行子任务的代理单元

可以理解为：

- Skill 更像“做事的方法模板”
- Subagent 更像“单独干活的角色”

#### Subagents vs Slash Commands

- Slash Commands 是触发入口
- Subagents 是执行单元

例如 `/agents` 是管理入口，但不是 subagent 本身。

#### Subagents vs Agent Teams

- Subagents 运行在同一个主会话之下
- Subagents 之间不能直接通信
- Agent Teams 才是多 Agent 协作模式

如果后续需求是多代理互相讨论、分工协作或共享任务列表，应考虑 Agent Teams，而不是单纯扩展 Subagents。

### 2.3 核心限制与展示边界

当前应明确遵循以下认知：

- Subagents 不能继续生成新的 Subagents
- Subagent 更适合聚焦单一职责
- 不适合承担需要频繁用户澄清的任务
- 不适合长期共享同一复杂上下文的任务

这些限制会直接影响：

- 数据模型设计
- 详情页字段解释
- 风险提示与能力边界展示

---

## 3. 配置与协议模型

### 3.1 基本文件结构

Claude Code 中的 Subagent 一般采用 Markdown 文件定义，其结构由两部分组成：

1. YAML frontmatter
2. Markdown 正文（作为 system prompt 内容）

因此，一个 Subagent 文件本质上需要被拆解为：

- 元信息区
- Prompt 正文区

后续后端解析时，必须分别处理这两部分。

### 3.5 关键字段

在当前开发阶段，应按以下字段作为重点支持对象：

- `name`
- `description`
- `tools`
- `disallowedTools`
- `model`
- `permissionMode`
- `maxTurns`
- `skills`
- `mcpServers`
- `hooks`
- `memory`
- `background`
- `effort`
- `isolation`

### 3.6 字段展示重点

这些字段在详情页中建议重点按以下方式理解与展示：

- `name`：作为列表主标题和详情页标题
- `description`：作为摘要说明和适用场景描述
- `tools`：作为已授权能力展示
- `disallowedTools`：作为受限能力展示
- `model`：作为模型标签与性能/成本倾向提示
- `permissionMode`：作为高优先级权限信息展示
- `maxTurns`：作为运行约束信息展示
- `skills`：作为扩展能力说明展示
- `mcpServers`：作为外部能力边界展示
- `hooks`：作为运行时增强点展示
- `memory`：作为 memory scope 展示
- `background`：作为后台运行能力展示
- `effort`：作为行为倾向信息展示
- `isolation`：作为隔离运行级别展示

### 3.2 来源与优先级

后续实现 Subagent 发现与详情展示时，必须考虑来源与优先级。

结合 `src/features/agents/tooling-matrix.json`，当前项目在“多平台资源发现”语境下，Subagent 的本地发现不应只假设 Claude Code 一种目录结构，而应按平台配置矩阵识别。

以 Claude Code 为例：

- 平台目录：`.claude/`
- Agents 目录：`.claude/agents/`
- 参考配置：`src/features/agents/tooling-matrix.json:43`

同一矩阵中，多个平台也提供 agents 目录，例如：

- Cursor：`.cursor/agents/`
- Claude Code Plugin：`.claude-plugin/agents/`
- OpenCode：`.opencode/agents/`
- Qoder：`.qoder/agents/`
- GitHub Copilot：`.github/agents/`

也有平台使用不同命名的 agent 目录，例如：

- Factory AI：`.factory/droids/`

典型来源包括：

- 项目级平台目录中的 agents 子目录
- 用户级平台目录中的 agents 子目录
- 插件目录中的 agents 子目录
- CLI 传入的 agents

这意味着详情页至少应支持展示：

- 平台类型
- 平台根目录与 agents 目录
- 文件路径
- 是否为当前生效版本
- 是否存在重名覆盖关系
- 是否可编辑

如果同名 Subagent 存在多个来源，系统后续应明确“最终生效项”和“被覆盖项”的展示策略。

### 3.3 基于 tooling-matrix 的发现策略

仅仅在文档里列举几个目录示例还不够，真正实现 Subagent 发现时，建议把 `src/features/agents/tooling-matrix.json` 视为平台规则表，而不是把路径硬编码在 scanner 中。

推荐发现流程如下：

1. 读取 `tooling-matrix.json` 中的所有平台项
2. 对每个平台取 `directory` 作为平台根目录候选
3. 读取该平台的 `agents` 字段
4. 如果 `agents` 为空，则说明该平台当前不提供 Subagent/Agent 目录，直接跳过
5. 如果 `agents` 非空，则将 `directory + agents` 组合为候选目录
6. 分别按项目级、用户级、插件级等 source scope 生成实际扫描路径
7. 扫描候选目录下的 Markdown agent 文件，并记录平台、来源、文件路径与优先级信息
8. 按名称或标识符归并结果，标记最终生效项、被覆盖项与不可解析项

结合当前矩阵，发现逻辑应至少覆盖这些典型情况：

- 标准 `agents/` 目录，如 Claude Code、Cursor、OpenCode、Qoder、GitHub Copilot
- 变体目录名，如 Factory AI 使用 `droids/`
- 不支持 agents 的平台，应通过 `agents: null` 明确跳过，而不是误报扫描失败

这意味着后端 scanner 更适合产出“候选发现项”，后续再由 parser / normalizer / resolver 继续处理，而不是在第一步就把所有平台写死成条件分支。

### 3.4 解析与归一化

Subagent 文件通常由 frontmatter 与 prompt 正文两部分组成，后端不应把原始 Markdown 直接传给前端组件。

开发者需要理解为什么必须做解析与归一化：

- frontmatter 字段可能缺失或类型不一致
- 同一字段可能存在单值与数组两种写法
- 不同来源的 Subagent 需要统一为稳定结构
- UI 需要稳定、可显示、可降级的字段结构
- 详情页需要区分结构化配置与 prompt 正文

推荐的解析阶段：

1. 读取 Markdown 文件并拆分 frontmatter 与正文
2. 解析 YAML 为结构化配置
3. 规范化字段值与默认值
4. 生成面向 UI 的详情模型与能力摘要
5. 保留必要的原始信息用于高级诊断展示


---

## 4. 当前项目的实现落点

当前仓库已经具备 Agents 原型和 Tauri 后端骨架，因此 Subagents 文档不应脱离当前代码结构重新定义，而应建立在现有模块之上。

### 4.1 前端落点

当前与 Agents 相关的前端模块主要集中在：

- `src/features/agents/types.ts`
- `src/features/agents/api.ts`
- `src/features/agents/discovery.ts`
- `src/features/agents/hooks.ts`
- `src/features/agents/use-agent-discovery.ts`
- `src/features/agents/use-agent-management.ts`
- `src/features/agents/use-agent-workspace.ts`

其中可以看到：

#### `src/features/agents/types.ts`
已包含：

- `ResourceKind = "subagent"`
- `SubagentResource`
- `AgentResource`
- `AgentResourceView`

说明当前前端已经把 Subagent 作为统一资源类型之一进行建模，但目前 `SubagentResource` 仍然是偏简化的原型结构，仅包含：

- `id`
- `kind`
- `name`
- `summary`
- `enabled`
- `model`
- `usageCount`
- `updatedAt`
- `prompt`
- `capabilities`

这套结构可以作为后续 Subagent 详情模型的起点，但不足以承载完整 frontmatter、诊断信息与来源信息。

#### `src/features/agents/api.ts`
当前 API 层已采用统一的 Tauri `invoke()` 调用模式，适合作为后续新增 Subagent 详情接口的延伸基础。

#### `src/features/agents/discovery.ts`
当前承担资源发现项转换、搜索和排序逻辑。后续 Subagent 详情能力应与 discovery 列表能力衔接，而不是另起一套完全独立的资源流。

#### `src/features/agents/use-agent-discovery.ts`
负责加载 discovered / managed / resolved agents 状态，是后续加入“Subagent 详情读取状态”的自然接入点之一。

#### `src/features/agents/use-agent-management.ts`
负责导入、刷新、启停等管理流程。若后续支持 Subagent 启用状态切换或绑定管理，可延续同样的状态同步策略。

#### `src/features/agents/use-agent-workspace.ts`
当前是 Agents 主工作台的数据编排核心。Subagent 的详情展示应被视为当前 workspace/detail panel 原型的纵向扩展。

### 4.2 后端落点

当前 Tauri 后端相关模块包括：

- `src-tauri/src/commands/agents.rs`
- `src-tauri/src/dto/agents.rs`
- `src-tauri/src/services/agent_discovery_service.rs`
- `src-tauri/src/scanners/agent_type_scanner.rs`
- `src-tauri/src/persistence/managed_agents_store.rs`

#### `src-tauri/src/commands/agents.rs`
当前已经形成 command 入口，后续新增 Subagent 详情读取接口时，应延续同样的 Tauri command 暴露方式。

#### `src-tauri/src/dto/agents.rs`
当前已有 DiscoveredAgent、ManagedAgent、ResolvedAgent 等 DTO，说明后端已采用 DTO 输出给前端的模式。后续 Subagent 详情信息也应以 DTO 形式返回，而不是直接透出原始解析结构。

#### `src-tauri/src/services/agent_discovery_service.rs`
当前 service 层主要负责 discovery 结果组织。后续 Subagent 文件解析、字段规范化与详情组装，建议继续归于 service 层。

### 4.3 当前状态判断

从现有代码看，当前项目已经具备：

- Agent discovery 列表基础
- Agent 管理原型
- Tauri command / dto / service 基础链路

但尚未完整具备：

- Subagent 文件发现与 frontmatter 解析
- Subagent 详情 DTO
- Subagent 配置诊断模型
- 面向详情面板的完整展示字段集合

因此，本文档的作用是为这部分增量能力提供统一依据。

---

## 5. 推荐数据模型

为支持 Subagents 列表与详情展示，建议将数据模型分为三层。

### 5.1 列表模型

目的：

- 为资源列表、侧栏、搜索结果和概览展示提供稳定字段

典型字段：

- `id`
- `name`
- `description`
- `source`
- `filePath`
- `model`
- `permissionMode`
- `enabled`
- `isActive`
- `isOverridden`
- `editable`
- `updatedAt`
- `warningCount`
- `errorCount`

如果继续复用现有 `SubagentResource`，建议后续按增量方式补全，而不是直接推翻当前类型层次。

### 5.2 详情模型

目的：

- 为右侧详情面板、资源详情页和高级查看模式提供完整字段结构

典型字段：

#### 基础信息

- `id`
- `name`
- `description`
- `source`
- `filePath`
- `priority`
- `editable`
- `isActive`
- `isOverridden`

#### 配置字段

- `tools`
- `disallowedTools`
- `model`
- `permissionMode`
- `maxTurns`
- `skills`
- `mcpServers`
- `hooks`
- `memory`
- `background`
- `effort`
- `isolation`

#### Prompt 相关

- `promptMarkdown`
- `promptPreview`

#### 原始数据

- `frontmatterRaw`
- `rawDocument`

#### 衍生信息

- `capabilities`
- `effectiveTools`
- `riskLevel`
- `statusSummary`

### 5.3 视图模型

目的：

- 为 UI 提供稳定、易展示的字段组合与派生摘要

典型字段：

- `promptPreview`
- `capabilities`
- `effectiveTools`
- `riskLevel`
- `statusSummary`
- `validationSummary`

### 5.4 诊断模型

目的：

- 为 diagnostics 区块提供结构化错误、警告和降级信息

典型字段：

- `errors`
- `warnings`
- `unknownFields`
- `missingRequiredFields`
- `invalidToolRefs`
- `invalidMcpRefs`

设计原则：

- 即使 frontmatter 存在错误，也尽量返回可展示的信息
- 诊断结果应服务于“展示 + 后续编辑”，而不只是简单报错

---

## 6. 详情展示的信息架构

当前项目已有 Agents 页面三栏原型，Subagent 详情应优先适配右侧详情面板，而不是单独设计孤立页面。

### 6.1 推荐分区

建议右侧详情结构按以下分区组织：

1. 基本信息
2. 运行配置
3. 工具权限
4. 扩展能力
5. Prompt
6. Diagnostics
7. 原始配置（高级模式）

### 6.2 基本信息区

展示内容建议包括：

- 名称
- 简述
- 类型（Subagent）
- 来源
- 文件路径
- 是否启用
- 是否为当前生效版本
- 是否可编辑
- 最后更新时间

### 6.3 运行配置区

展示内容建议包括：

- model
- permissionMode
- background
- maxTurns
- effort
- isolation

推荐以 badge + definition list 的形式展示，便于快速浏览。

### 6.4 工具权限区

展示内容建议包括：

- allowed tools
- disallowed tools
- 关键能力摘要
- 是否涉及 MCP

这一区域是能力边界展示的重点，建议配合状态色或说明文案。

### 6.5 扩展能力区

展示内容建议包括：

- skills
- mcpServers
- hooks
- memory

如果某些字段未配置，应显示明确的 empty state，而不是直接不渲染。

### 6.6 Prompt 区

展示内容建议包括：

- prompt 摘要
- 原始 markdown 内容
- 必要时支持折叠/展开

Prompt 区应与配置字段区明确区分，避免把元信息和正文混在一起。

### 6.7 Diagnostics 区

展示内容建议包括：

- warnings
- errors
- 未识别字段
- 缺失关键字段
- 解析异常说明

这是详情页面向开发者的重要部分，应尽量结构化展示，而不是只拼接为一段错误文本。

### 6.8 原始配置区

在高级模式下，可展示：

- 原始 YAML frontmatter
- 原始 Markdown 文件内容

该区块主要用于开发调试，不建议默认展开。

---

## 7. 后端职责建议

后续若在 Tauri 后端正式支持 Subagent 详情能力，建议职责拆分如下。

### 7.1 扫描层

负责：

- 扫描 Subagent 目录
- 收集文件路径
- 标记来源类型
- 处理重复项和覆盖关系基础信息

可落在 scanner/service 协作层中。

### 7.2 解析层

负责：

- 读取 Markdown 文件
- 提取 frontmatter
- 提取 prompt body
- 解析 YAML 为结构化配置

### 7.3 规范化层

负责：

- 默认值补全
- 字段类型标准化
- 单值/数组统一处理
- 未知字段保留

### 7.4 诊断层

负责：

- 记录解析错误
- 记录字段异常
- 记录引用无效项
- 生成前端可直接显示的 warning/error 结构

### 7.5 DTO 输出层

负责：

- 向前端输出列表 DTO
- 向前端输出详情 DTO
- 避免前端依赖原始内部解析结构

建议沿用现有：

- command → service → dto

这一结构，不额外引入新的链路风格。

---

## 8. 前端职责建议

### 8.1 列表与详情的关系

前端应把 Subagent 视为现有资源体系中的一种，而不是特殊页里的孤立对象。

因此建议：

- 列表继续沿用当前 `ResourceKind = "subagent"` 的分类方式
- 详情展示在现有 workspace/detail panel 结构中扩展
- 搜索、筛选、排序规则尽量与 Skills / MCP 保持一致

### 8.2 展示组件建议

Subagent 详情页/详情面板建议使用以下 UI 组织方式：

- badge：展示 model、source、permissionMode、background、isolation
- alert：展示警告和风险提示
- tabs 或 section 分块：区分 overview / prompt / diagnostics
- code block：展示 YAML 或 prompt 原文
- definition list / table：展示配置键值
- empty state：显示未配置字段

### 8.3 i18n 文案策略

考虑当前项目已使用 i18n，详情展示建议采用以下文案策略：

- 字段标题使用中文翻译
- 字段原值尽量保留官方英文
- 对关键概念提供简短中文说明

例如：

- 权限模式：`dontAsk`
- 说明：自动接受已允许的操作，不主动询问用户

这样既利于学习，又利于后续和官方文档对照。

### 8.4 详情展示原则

前端展示应遵循：

- 先显示开发者最关心的有效信息
- 原始配置放到高级区
- 错误与警告要显式可见
- 缺失配置要明确说明，而不是隐藏
- 不把 frontmatter 与 prompt 混成一个长文本块

---

## 9. 开发检查清单

后续实现 Subagents 列表与详情功能时，可按以下清单推进。

### 9.1 基础能力

- [ ] 扫描 Subagent 文件
- [ ] 区分来源类型
- [ ] 解析 YAML frontmatter
- [ ] 提取 Markdown 正文
- [ ] 生成统一结构化模型

### 9.2 列表能力

- [ ] 输出列表 DTO
- [ ] 提供名称、摘要、来源、更新时间等基础字段
- [ ] 提供搜索与排序所需字段
- [ ] 标记启用状态和覆盖状态

### 9.3 详情能力

- [ ] 输出详情 DTO
- [ ] 返回 frontmatter 关键字段
- [ ] 返回 prompt 正文
- [ ] 返回来源路径和可编辑性信息
- [ ] 返回衍生能力摘要

### 9.4 诊断能力

- [ ] 返回 warnings
- [ ] 返回 errors
- [ ] 返回 unknown fields
- [ ] 返回缺失关键字段提示
- [ ] 遇到坏配置时仍尽量可展示

### 9.5 前端展示能力

- [ ] 右栏详情区分组渲染
- [ ] 风险字段显示 badge 或 alert
- [ ] prompt 支持预览与展开
- [ ] diagnostics 支持结构化展示
- [ ] 高级模式支持查看原始配置

### 9.6 后续可扩展能力

- [ ] Subagent 编辑
- [ ] 配置保存
- [ ] 校验前置提示
- [ ] 覆盖关系可视化
- [ ] 从详情页跳转到本地文件

---

## 10. 推荐实现顺序

后续开发可以按下面顺序推进：

1. 定义 Subagent 列表与详情 DTO
2. 定义 frontmatter 解析与归一化规则
3. 实现后端文件发现、解析与诊断 commands
4. 实现前端 detail view model 映射
5. 实现概览与运行配置区块
6. 实现工具权限、扩展能力与 prompt 区块
7. 实现 diagnostics 与原始配置查看
8. 完善来源优先级、覆盖关系与编辑能力预留

---

## 11. 对当前项目的直接指导结论

结合当前仓库现状，后续开发应遵循以下原则：

1. 不要为 Subagent 详情单独建立完全独立的数据流，应建立在现有 agents feature 基础上扩展。
2. 前端以 `src/features/agents/types.ts` 中现有 `SubagentResource` 为起点增量扩展详情模型。
3. 后端以 `src-tauri/src/commands/agents.rs`、`src-tauri/src/services/agent_discovery_service.rs`、`src-tauri/src/dto/agents.rs` 现有链路为扩展基础。
4. 详情展示优先服务开发与调试，因此必须把 diagnostics 作为正式区块，而不是补充信息。
5. 文档与实现都应明确区分“结构化配置字段”和“prompt 正文内容”。
6. 来源、优先级、是否被覆盖、是否可编辑，是详情页必须具备的上下文信息。

---

## 12. 参考资料

官方资料建议重点参考：

- Claude Code Subagents
- Claude Code Skills
- Claude Code Permissions
- Claude Code Hooks
- Claude Code MCP
- Claude Agent SDK Subagents

在后续实现中，如需与官方术语保持一致，应优先保留字段英文原值，并在 UI 中补充中文解释。
