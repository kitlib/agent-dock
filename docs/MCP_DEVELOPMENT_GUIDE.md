# MCP 开发指南

## 1. 目标与适用范围

本文用于为 AgentDock 后续的 MCP 相关开发提供统一依据，覆盖以下目标：

- 统一 MCP 的核心概念与协议分层认知
- 明确 MCP 发现与详情展示所需的数据层次
- 结合当前产品定位，约束解析、归一化与展示策略
- 为 MCP 列表、详情展示和运行时诊断提供实现依据
- 为后端 DTO、前端 ViewModel 与详情区块设计提供统一参考

本文主要服务于以下开发场景：

- MCP 配置发现与注册信息读取
- MCP 连接信息与能力概览展示
- MCP 详情面板展示
- MCP DTO 与前后端接口设计
- MCP 运行时探测、诊断与敏感信息脱敏

---

## 2. 核心概念

### 2.1 本项目里的 MCP 是什么

MCP 通常指 Model Context Protocol。

在本项目中，MCP 应被理解为一种用于描述并连接 AI 客户端与外部能力的标准化协议。这里的外部能力包括：

- tools
- resources
- resource templates
- prompts
- 可选的协议能力声明

对 AgentDock 来说，MCP 不只是一个原始协议对象，它还是一种面向产品的资源类型，因此需要支持：

- 发现
- 注册
- 持久化
- 查看
- 状态上报
- 安全展示敏感配置

### 2.2 MCP 核心概念

在实现发现或详情展示之前，开发者需要先理解这些 MCP 基本概念：

- host：管理 MCP 连接的宿主应用
- client：与 MCP server 通信的客户端组件
- server：提供 tools、resources、prompts 的能力提供方
- transport：客户端与 server 建立连接的方式
- capabilities：server 声明自己支持的能力
- initialize：协议初始化与握手阶段
- tools/list：列出可调用工具
- resources/list：列出可读取资源
- resources/templates/list：列出参数化资源模板
- prompts/list：列出可复用的提示模板

之所以必须掌握这些概念，是因为详情页应该反映协议结构，而不是只展示一些零散字段。

### 2.3 核心限制与展示边界

当前应明确遵循以下认知：

- MCP 详情展示不等于完整 MCP 客户端实现
- 静态配置、运行时握手结果和运行时列表结果不应混为一层
- 不同 transport 的字段差异会直接影响数据模型与 UI 展示
- 敏感配置默认不应明文展示
- 详情页必须支持部分成功，而不是只有成功或失败两态

这些限制会直接影响：

- DTO 分层
- 详情页区块设计
- 状态建模
- 脱敏规则

---

## 3. 配置与协议模型

### 3.1 Transport 相关知识

MCP 的发现和详情展示都依赖对 transport 的理解。

在 MCP 相关系统中，常见 transport 包括：

- stdio
- streamable HTTP
- 一些旧实现或过渡实现中会出现 SSE

开发者需要知道不同 transport 会如何影响数据模型与展示方式。

#### stdio

通常需要这些信息：

- command
- args
- env

展示时应关注：

- 可执行命令名需要清晰可见
- args 可能很长，建议支持折叠/展开
- env 的值可能包含敏感信息，默认应脱敏

#### HTTP / SSE

通常需要这些信息：

- URL
- headers
- 身份认证相关提示

展示时应关注：

- URL 要清楚展示
- headers 可能包含 token，默认应脱敏
- transport 类型应明确显示，而不是仅靠 URL 推断

### 3.2 发现相关知识

MCP 发现并不只是读取一个配置文件，通常需要组合多个来源。

结合 `src/features/agents/tooling-matrix.json`，当前项目在“多平台资源发现”语境下，MCP 的本地发现也不应只假设 Claude Code 一种配置位置，而应按平台配置矩阵识别。

以 Claude Code 为例：

- 平台目录：`.claude/`
- MCP 配置：项目根目录 `.mcp.json`
- 参考配置：`src/features/agents/tooling-matrix.json:43`

同一矩阵中，不同平台的 MCP 配置位置并不统一，例如：

- Cursor：`.cursor/mcp.json`
- Codex CLI：`.codex/config.toml`
- Goose：`.goose/config.yaml`
- Cline：`cline_mcp_settings.json`
- Crush：项目根目录 `crush.json`

这说明 MCP 发现的关键不是扫描某个固定子目录，而是根据平台规则定位对应配置文件。

推荐的数据来源包括：

1. 本地静态配置文件
2. registry 或 marketplace 元数据
3. 运行时握手结果
4. 运行时 tools/resources/prompts 的列表结果

开发者需要区分这些层，因为它们回答的问题并不一样。

#### 静态配置回答的问题

- 应该如何连接
- 这个 server 是谁配置的
- 本地 id 和本地元数据是什么
- 该平台的 MCP 配置文件位于哪里

#### 运行时握手回答的问题

- 真实的 server 名称是什么
- 使用的协议版本是什么
- 支持哪些 capabilities

#### 运行时列表回答的问题

- 实际暴露了哪些 tools
- 实际暴露了哪些 resources
- 实际暴露了哪些 prompts

如果在过早阶段把这些层混在一起，后续排查问题会变得很困难。

### 3.3 基于 tooling-matrix 的发现策略

MCP 的发现不应写成“扫描某个固定目录”的逻辑，而应把 `src/features/agents/tooling-matrix.json` 作为平台配置规则表使用。

推荐发现流程如下：

1. 读取 `tooling-matrix.json` 中的所有平台项
2. 对每个平台取 `directory` 作为平台根目录候选
3. 读取该平台的 `mcp` 字段
4. 如果 `mcp` 为空，则说明该平台当前没有可识别的 MCP 配置入口，直接跳过
5. 如果 `mcp` 标记为某个平台目录内文件，则组合 `directory + mcp`
6. 如果 `mcp` 标记为 `(...root)`，则应理解为项目根目录文件，而不是平台子目录内文件
7. 对候选配置路径执行存在性检查，并记录平台、来源、配置路径与 transport 线索
8. 将读取成功的配置文件解析为静态 config layer，再进入后续 runtime probe 与 view model 组装流程

结合当前矩阵，发现逻辑应至少覆盖这些差异：

- 平台目录内配置文件，如 Cursor 的 `.cursor/mcp.json`、Goose 的 `.goose/config.yaml`
- 项目根目录配置文件，如 Claude Code 的 `.mcp.json`、Crush 的 `crush.json`
- 非 JSON 配置格式，如 Codex CLI 的 `config.toml`
- 不支持 MCP 的平台，应通过 `mcp: null` 明确跳过

这说明 MCP scanner 的职责重点是“根据平台规则定位配置入口”，而不是预设单一文件名或单一目录结构。

### 3.4 解析与归一化相关知识

原始 MCP payload 不应该直接传给 UI 组件。

开发者需要理解为什么必须做归一化：

- 运行时响应可能不完整
- 字段很多是可选的
- 不同 server 可能只填充部分字段
- 后续协议版本可能增加新字段
- UI 需要稳定、可显示、可降级的字段结构

推荐的解析阶段：

1. 把配置解析成带 transport 语义的内部模型
2. 把 initialize 结果解析成运行时元信息
3. 把各类 list 结果解析成结构化集合
4. 把 schema 展平成适合人阅读的展示形式
5. 最后组装成 UI 直接使用的 ViewModel

### 3.5 Tool 展示所需的 JSON Schema 知识

MCP 的 tool 详情通常依赖 `inputSchema`。

开发者需要知道这些 JSON Schema 字段对展示最重要：

- type
- properties
- required
- description
- default
- enum
- items
- oneOf / anyOf / allOf
- format
- additionalProperties

首版 UI 不需要完整实现一个 schema 引擎，但至少要支持：

- 顶层 object properties
- required 字段
- 基础类型显示
- 默认值显示
- 枚举值显示
- 原始 schema 的兜底展示

如果不了解这些，tool 详情要么过于模糊，要么强依赖某个具体实现。

---

## 4. 当前项目的实现落点

当前项目将 MCP 作为统一资源类型的一部分，因此后续 MCP 文档与实现应建立在现有 agents/resource 原型之上，而不是脱离现有结构单独设计。

### 4.1 前端落点

后续 MCP 详情与发现能力，应优先落在现有资源体系与工作台结构中，包括：

- `src/features/agents/types.ts`
- `src/features/agents/api.ts`
- `src/features/agents/discovery.ts`
- `src/features/agents/use-agent-discovery.ts`
- `src/features/agents/use-agent-management.ts`
- `src/features/agents/use-agent-workspace.ts`

这些模块已经承担：

- 资源类型建模
- Tauri invoke 数据获取
- 列表筛选与排序
- 工作台状态编排

因此 MCP 的发现、详情和诊断能力，建议继续沿用这套前端组织方式。

### 4.2 后端落点

MCP 相关后端能力，建议继续落在现有 Tauri command / service / dto 链路中，包括：

- `src-tauri/src/commands/agents.rs`
- `src-tauri/src/dto/agents.rs`
- `src-tauri/src/services/agent_discovery_service.rs`
- `src-tauri/src/scanners/`
- `src-tauri/src/persistence/`

后续若需要引入 MCP 专项探测或独立 DTO，也建议保持相同链路风格。

### 4.3 当前状态判断

从当前项目上下文看，MCP 在产品层面已经被定义为资源类型之一，但后续仍需补强：

- 发现数据来源分层
- config/runtime/view model 三层建模
- 运行时探测结果归一化
- 详情页区块与 diagnostics 结构
- 敏感信息脱敏规则

因此，本文档主要用于为这些增量能力提供统一依据。

---

## 5. 推荐数据模型

一个实用的实现模型是把 MCP 数据分成三层。

### 5.1 Config Layer

目的：

- 描述 server 是如何被注册与连接的

典型字段：

- id
- title
- description
- transportType
- command / args / env
- url / headers
- source
- configPath

### 5.2 Runtime Layer

目的：

- 描述运行中的 server 在协议握手时返回了什么

典型字段：

- protocolVersion
- serverInfo.name
- serverInfo.version
- capabilities
- initializeStatus
- lastCheckedAt

### 5.3 ViewModel Layer

目的：

- 为 UI 提供稳定、易展示的字段

典型字段：

- displayName
- displayDescription
- transportSummary
- capabilityBadges
- tools
- resources
- resourceTemplates
- prompts
- diagnosticsSummary
- connectionStatus

这种分层可以更容易排查“本地配置”和“真实运行结果”之间的不一致。

### 5.4 诊断模型

MCP 详情页必须支持“部分成功”。

开发者需要理解这些状态彼此独立：

- config 存在
- 连接已建立
- initialize 成功
- tools 已加载
- resources 已加载
- prompts 已加载

因此，一个单独的 boolean loading 状态远远不够。

推荐按区块建模：

- idle
- loading
- success
- error
- unsupported

同时建议保留：

- 最近检查时间
- initialize 成功或失败
- 局部加载失败信息
- 最近错误消息
- 原始协议 payload（高级诊断模式）

---

## 6. 详情展示的信息架构

要把 MCP 详情展示做好，开发者需要先明确哪些数据对产品是关键的。

推荐的详情区块包括：

- 概览
- 连接信息
- capabilities
- tools
- resources
- resource templates
- prompts
- diagnostics

### 6.1 概览

通常应包含：

- 展示名称
- 描述
- server 版本
- protocol 版本
- transport 类型
- 当前连接状态

### 6.2 连接信息

通常应包含：

- stdio 模式下的 command / args / env
- HTTP 类 transport 下的 URL / headers
- 敏感信息脱敏规则

### 6.3 Capabilities

通常应包含这些能力开关：

- tools
- resources
- prompts
- sampling
- roots
- logging
- 如果有暴露，也包括 experimental 扩展能力

### 6.4 Resources 与 Prompts 区块

详情页需要能明确区分：

- tools
- resources
- resource templates
- prompts

避免把所有运行时能力都折叠成一个笼统列表。

### 6.5 Diagnostics

通常应包含：

- 最近检查时间
- initialize 成功或失败
- 局部加载失败信息
- 最近错误消息
- 在必要时提供可折叠的原始协议 payload

---

## 7. 后端职责建议

在后端开发里，团队需要理解：

- Tauri commands 如何把 MCP 相关数据暴露给前端
- MCP 发现在哪里发生
- MCP 持久化在哪里发生
- 运行时探测在哪里发生
- 超时与命令失败如何上报

推荐后端职责：

- 加载或发现 MCP 配置
- 归一化 transport 相关连接元数据
- 按需探测运行时 server 信息
- 向前端返回稳定的 DTO
- 默认避免泄露 secrets

推荐首版后端不要承担的职责：

- 渲染 UI 专用文案
- 为所有未来场景过度重塑原始 payload

---

## 8. 前端职责建议

在前端开发里，团队需要理解：

- 如何消费归一化后的 DTO，而不是直接消费原始 payload
- 如何设计可展开的详情区块
- 如何把 schema 渲染成表格或标签
- 如何表达部分加载与 unsupported 状态
- 如何只把 raw JSON 作为高级诊断能力展示

推荐前端职责：

- 从归一化字段中导出展示标签
- 清晰展示状态
- 即使深层探测较慢，也保持概览区快速可用
- 保持 summary 与 diagnostics 的区分
- 把 raw JSON 放在高级 diagnostics 区块

---

## 9. 开发检查清单

在新增或重构 MCP 发现/详情功能前，建议确认实现是否覆盖以下知识点。

### 9.1 协议

- [ ] 理解 initialize 与 capability negotiation
- [ ] 理解 tools/resources/prompts 的 list 接口
- [ ] 理解不同 transport 的差异

### 9.2 数据建模

- [ ] 分离 config、runtime 与 UI view model
- [ ] 在适合诊断的场景保留 raw payload
- [ ] 支持缺失的可选字段

### 9.3 Schema 展示

- [ ] 能解析基础 JSON Schema object properties
- [ ] 能标记 required 字段
- [ ] 能显示 default 和 enum
- [ ] 能提供 raw schema 兜底展示

### 9.4 安全

- [ ] 对 secret 字段脱敏
- [ ] 避免明文日志输出 secret
- [ ] 区分公开元数据与私密配置

### 9.5 UX

- [ ] 支持部分加载
- [ ] 支持 unsupported 状态
- [ ] 保持概览区易读
- [ ] 把 raw JSON 放在高级 diagnostics 区块

### 9.6 集成

- [ ] 保持 Tauri command contract 稳定
- [ ] 避免前端组件与协议内部细节过度耦合
- [ ] 为协议扩展保留空间，不让展示层轻易被破坏

---

## 10. 推荐实现顺序

后续开发可以按下面顺序推进：

1. 定义内部 MCP DTO
2. 定义 parser 与归一化规则
3. 实现后端发现与探测 commands
4. 实现前端 detail view model 映射
5. 实现 overview 与 status 区块
6. 实现 tool/resource/prompt 详情区块
7. 实现 diagnostics 与原始 payload 查看
8. 完善 secret 脱敏与失败处理

---

## 11. 对当前项目的直接指导结论

结合当前项目现状，后续 MCP 开发应遵循以下原则：

1. 不要把 MCP 详情页等同于完整协议客户端，应优先服务资源发现与详情展示。
2. 应明确分离静态配置、运行时握手结果与运行时列表结果。
3. 应建立 config/runtime/view model 三层模型，避免前端直接依赖原始协议 payload。
4. diagnostics 应作为正式区块存在，而不是附属信息。
5. 敏感配置默认脱敏展示，避免在 UI 和日志中明文泄露。
6. 前后端结构应继续复用现有 agents/resource 工作台与 Tauri command 链路。

---

## 12. 参考资料

- MCP 官方网站：https://modelcontextprotocol.io
- MCP 规范：https://spec.modelcontextprotocol.io
- MCP GitHub 组织：https://github.com/modelcontextprotocol
- Anthropic 文档：https://docs.anthropic.com
