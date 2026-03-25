# Skills 开发指南

## 1. 目标与适用范围

本文用于为 AgentDock 后续的 Skills 相关开发提供统一依据，覆盖以下目标：

- 统一 Skills 发现与详情展示的核心认知
- 明确 Skills 发现、解析、归一化与展示所需的数据层次
- 结合当前项目已有代码结构，确定前后端的实现落点
- 为 Skill 列表、详情展示和后续增强能力提供数据模型依据
- 为前后端 DTO、详情区块与诊断展示提供统一参考

本文主要服务于以下开发场景：

- 本地 Skills 发现与扫描
- Skill 基础信息展示
- Skill 详情面板展示
- Skill DTO 与前后端接口设计
- Skill 配置解析、归一化、诊断与容错

---

## 2. 核心概念

### 2.1 本项目里的 Skill 是什么

在本项目中，Skill 应被理解为一种用于承载可复用提示、工作流说明与上下文注入内容的资源类型。

对 AgentDock 来说，Skill 不只是一个 Markdown 文件，它还是一种面向产品的资源，因此需要支持：

- 发现
- 解析
- 归一化
- 持久化接入
- 列表展示
- 详情展示
- 诊断与调试

### 2.2 Skills 的核心认知

在实现 Skills 发现或详情展示之前，开发者需要先理解这些基本点：

- Skill 通常以目录为单位组织
- Skill 常以 `SKILL.md` 作为主要入口文件
- Skill 可能包含 frontmatter、正文和 supporting files
- Skill 同时有“发现层结构”和“展示层结构”两类模型
- Skill 会被纳入当前统一资源体系，与 MCP / Subagents 并列展示

之所以必须掌握这些概念，是因为详情页应该反映 Skill 的结构与来源，而不是只展示零散文本。

### 2.3 核心限制与展示边界

当前应明确遵循以下认知：

- Skill 详情展示不等于完整 Skill 执行系统实现
- 发现结果、解析结果和 UI 展示结果不应混为一层
- 本地 Skill 与 marketplace Skill 的字段语义不完全相同
- 详情页必须支持解析失败但仍可展示部分信息
- 原始 Markdown、frontmatter 和结构化元数据应分层展示

这些限制会直接影响：

- DTO 分层
- 详情页区块设计
- 解析诊断建模
- 本地与 marketplace 资源兼容方式

---

## 3. 配置与协议模型

### 3.1 基本文件结构

Skill 通常以目录为基本单位组织，入口文件一般为 `SKILL.md`。

在当前开发语境中，一个 Skill 至少可能包含以下组成部分：

- Skill 根目录
- `SKILL.md` 入口文件
- frontmatter
- Markdown 正文
- supporting files
- 链接、示例或其它辅助内容

后续后端解析时，应把“目录结构”和“入口文档结构”分开处理。

### 3.2 来源与归属

后续实现 Skill 发现与详情展示时，必须考虑来源与归属。

结合 `src/features/agents/tooling-matrix.json`，当前项目在“多平台资源发现”语境下，Skill 的本地发现不应只假设 Claude Code 一种目录结构，而应按平台配置矩阵识别。

以 Claude Code 为例：

- 平台目录：`.claude/`
- Skills 目录：`.claude/skills/`
- 参考配置：`src/features/agents/tooling-matrix.json:43`

同一矩阵中，很多平台也都采用“平台目录 + `skills/` 子目录”的结构，例如：

- Cursor：`.cursor/skills/`
- Claude Code Plugin：`.claude-plugin/skills/`
- Continue：`.continue/skills/`
- Windsurf：`.windsurf/skills/`

也有少量平台使用变体路径，例如：

- Pi-Mono：`.pi/agent/skills/`

因此建议至少支持以下来源相关信息：

- `platform`
- `sourceScope`
- `sourceRootPath`
- `toolingDirectory`
- `skillPath`
- `entryFilePath`
- `ownerAgentId`
- `workspaceName`

这意味着详情页至少应支持展示：

- 平台类型
- 平台根目录与 skills 目录
- 根路径与入口文件路径
- 所属 agent 或 workspace 上下文
- 是否来自本地发现还是 marketplace

### 3.3 基于 tooling-matrix 的发现策略

Skill 发现不应只靠几个示例路径来实现，后续 scanner 更适合直接复用 `src/features/agents/tooling-matrix.json` 里的平台规则。

推荐发现流程如下：

1. 读取 `tooling-matrix.json` 中的所有平台项
2. 对每个平台取 `directory` 作为平台根目录候选
3. 读取该平台的 `skills` 字段
4. 如果 `skills` 为空，则说明该平台当前没有 Skill 目录，直接跳过
5. 如果 `skills` 非空，则将 `directory + skills` 组合为候选目录
6. 分别按项目级、用户级、插件级等 source scope 生成实际扫描路径
7. 在候选目录下查找 Skill 根目录，并进一步识别其入口文件 `SKILL.md`
8. 为每个发现结果记录平台、来源、根目录、入口文件路径与解析状态

结合当前矩阵，发现逻辑应至少覆盖这些典型情况：

- 常规 `skills/` 子目录，如 Claude Code、Cursor、Continue、Windsurf
- 变体目录，如 Pi-Mono 使用 `agent/skills/`
- 不支持 skills 的平台，应通过 `skills: null` 明确跳过

这意味着 Skill discovery 的核心不是写死某个平台目录，而是根据矩阵把“平台目录 + skills 相对子路径”转换为可扫描的候选路径集合。

### 3.4 解析与归一化

Skill 目录及其入口文件不应直接原样传给 UI 组件。

开发者需要理解为什么必须做解析与归一化：

- `SKILL.md` 可能包含 frontmatter，也可能没有
- 不同 Skill 目录的 supporting files 结构可能不同
- 本地 Skill 与 marketplace Skill 的原始字段来源不同
- UI 需要稳定、可显示、可降级的字段结构
- 详情页需要区分原始内容、结构化元数据和诊断信息

推荐的解析阶段：

1. 扫描 Skill 根目录与入口文件
2. 解析 `SKILL.md` 的 frontmatter 与 Markdown 正文
3. 提取 supporting files、links、headings、code blocks 等结构化信息
4. 生成面向 UI 的列表模型、详情模型与能力摘要
5. 保留必要的原始信息用于高级诊断展示

### 3.5 关键字段

在当前开发阶段，应按以下字段作为重点支持对象：

- `id`
- `name`
- `displayName`
- `qualifiedName`
- `fingerprint`
- `summary`
- `description`
- `tags`
- `markdown`
- `frontmatter`
- `frontmatterRaw`
- `supportingFiles`
- `placeholders`
- `dynamicInjections`
- `allowedTools`
- `context`
- `agent`
- `hooks`
- `warnings`
- `errors`
- `status`
- `updatedAt`

### 3.6 字段展示重点

这些字段在详情页中建议重点按以下方式理解与展示：

- `name` / `displayName`：作为列表标题和详情标题
- `summary` / `description`：作为摘要说明与说明信息
- `tags`：作为分类和筛选标签
- `markdown`：作为正文预览与详情阅读内容
- `frontmatter` / `frontmatterRaw`：作为结构化元信息与高级调试信息
- `supportingFiles`：作为附属文件与引用内容展示
- `allowedTools` / `context` / `agent`：作为执行与上下文约束信息展示
- `warnings` / `errors` / `status`：作为诊断与健康状态展示
- `updatedAt`：作为更新时间展示

---

## 4. 当前项目的实现落点

当前项目已经具备统一资源工作台与详情面板原型，因此 Skills 文档与实现应建立在现有资源体系之上，而不是脱离现有结构单独设计。

### 4.1 前端落点

当前与 Skills 展示相关的前端模块主要集中在：

- `src/pages/home.tsx`
- `src/features/agents/use-agent-workspace.ts`
- `src/features/agents/hooks.ts`
- `src/features/agents/discovery.ts`
- `src/features/agents/types.ts`
- `src/features/agents/resource-panel.tsx`
- `src/features/agents/detail-panel.tsx`
- `src/features/agents/agent-rail.tsx`

这些模块已经承担：

- 三栏工作台布局
- 当前选中 agent / resource / tab 的聚合
- 本地资源与 marketplace 资源的统一搜索与排序
- 通用资源详情入口

因此 Skills 的发现、详情和诊断能力，建议继续沿用这套前端组织方式。

### 4.2 后端落点

结合当前仓库结构，Skills 相关后端能力后续应优先落在现有 Tauri command / service / dto 链路中，包括：

- `src-tauri/src/commands/`
- `src-tauri/src/dto/`
- `src-tauri/src/services/`
- `src-tauri/src/scanners/`
- `src-tauri/src/persistence/`

后续若引入 Skill 专项 scanner、parser、diagnostic DTO，也建议保持相同链路风格。

### 4.3 当前状态判断

从当前项目上下文看，Skills 已经被纳入统一资源类型之一，但后续仍需补强：

- 技能目录扫描入口
- `SKILL.md` 的正式 parser
- supporting files 扫描与分类
- Skill 发现层模型与解析层模型
- Skill 详情专用结构化元数据
- diagnostics 与原始内容分层展示

因此，本文档主要用于为这些增量能力提供统一依据。

---

## 5. 推荐数据模型

为支持 Skills 列表与详情展示，建议将数据模型分为四层。

### 5.1 发现模型

目的：

- 表达“从哪里发现了一个 Skill，以及发现是否成功”

典型字段：

- `sourceScope`
- `sourceRootPath`
- `skillPath`
- `entryFilePath`
- `ownerAgentId`
- `status`
- `warnings`
- `errors`
- `detectedAt`
- `parserVersion`

### 5.2 详情模型

目的：

- 为右侧详情面板、资源详情页和高级查看模式提供完整字段结构

典型字段：

#### 基础信息

- `id`
- `name`
- `displayName`
- `qualifiedName`
- `fingerprint`
- `sourceScope`
- `skillPath`
- `entryFilePath`
- `ownerAgentId`
- `updatedAt`

#### 内容字段

- `summary`
- `description`
- `markdown`
- `frontmatter`
- `frontmatterRaw`
- `headings`
- `links`
- `codeBlocks`
- `supportingFiles`

#### 执行与上下文字段

- `allowedTools`
- `disableModelInvocation`
- `userInvocable`
- `context`
- `agent`
- `hooks`
- `placeholders`
- `dynamicInjections`

### 5.3 视图模型

目的：

- 为 UI 提供稳定、易展示的字段组合与派生摘要

典型字段：

- `name`
- `summary`
- `tags`
- `updatedAt`
- `enabled`
- `usageCount`
- `markdownPreview`
- `metadataSummary`
- `diagnosticsSummary`
- `sourceSummary`

### 5.4 诊断模型

目的：

- 为 diagnostics 区块提供结构化错误、警告和降级信息

典型字段：

- `status`
- `warnings`
- `errors`
- `missingFields`
- `parserVersion`
- `detectedAt`
- `normalizedHash`

设计原则：

- 即使解析失败，也尽量返回可展示的基础信息
- 诊断结果应服务于“展示 + 调试 + 后续编辑”，而不只是简单报错

---

## 6. 详情展示的信息架构

当前项目已有 Agents 页面三栏原型，Skill 详情应优先适配右侧详情面板，而不是单独设计孤立页面。

### 6.1 推荐分区

建议右侧详情结构按以下分区组织：

1. 基本信息
2. 来源与归属
3. 正文内容
4. 结构化元数据
5. supporting files
6. 执行与上下文约束
7. Diagnostics
8. 原始配置（高级模式）

### 6.2 基本信息区

展示内容建议包括：

- 名称
- 摘要
- 类型（Skill）
- 标签
- 是否启用
- 最后更新时间

### 6.3 来源与归属区

展示内容建议包括：

- 来源类型
- source root path
- skill path
- entry file path
- 所属 agent / workspace

### 6.4 正文内容区

展示内容建议包括：

- Markdown 正文
- 摘要预览
- headings 导航（可选）

### 6.5 结构化元数据区

展示内容建议包括：

- frontmatter
- description
- tags
- placeholders
- dynamic injections
- code blocks / links（按需）

### 6.6 supporting files 与执行约束区

展示内容建议包括：

- supporting files
- allowed tools
- context
- agent
- hooks
- user invocable 等执行约束信息

### 6.7 Diagnostics 区

展示内容建议包括：

- 解析状态
- warnings
- errors
- missing fields
- parser version
- detected at

### 6.8 原始配置区

在高级模式下，可展示：

- 原始 `SKILL.md`
- 原始 frontmatter
- 原始 supporting files 列表

---

## 7. 后端职责建议

在后端开发里，团队需要理解：

- Skills 发现在哪里发生
- `SKILL.md` 解析在哪里发生
- supporting files 扫描在哪里发生
- 结构化元数据如何输出给前端
- 解析失败如何上报与保留诊断信息

推荐后端职责：

- 扫描 Skill 根目录与入口文件
- 解析 `SKILL.md` 的 frontmatter 和 Markdown 正文
- 提取 supporting files、links、headings 等结构化信息
- 生成稳定的 DTO
- 在解析失败时仍尽量返回基础可展示信息

推荐首版后端不要承担的职责：

- 渲染 UI 专用文案
- 过度设计与未来所有 Skill 形态兼容的超大模型

---

## 8. 前端职责建议

在前端开发里，团队需要理解：

- 如何消费归一化后的 DTO，而不是直接消费 parser 原始结果
- 如何区分正文、结构化元数据和 diagnostics
- 如何在现有 detail panel 中渐进增强 Skill 详情
- 如何兼容本地 Skill 与 marketplace Skill 的字段差异
- 如何只把 raw 内容作为高级诊断能力展示

推荐前端职责：

- 从归一化字段中导出列表项展示标签
- 在详情面板中结构化展示 metadata 与正文
- 清晰展示来源、状态与诊断
- 即使深层解析较慢或失败，也保持基础概览可用
- 把 raw Markdown / raw frontmatter 放在高级 diagnostics 区块

---

## 9. 开发检查清单

后续实现 Skills 列表与详情功能时，可按以下清单推进。

### 9.1 基础能力

- [ ] 扫描 Skill 目录
- [ ] 识别 `SKILL.md`
- [ ] 区分来源类型与归属关系
- [ ] 解析 frontmatter 与 Markdown 正文
- [ ] 提取 supporting files 与结构化内容

### 9.2 数据建模

- [ ] 定义发现模型
- [ ] 定义详情模型
- [ ] 定义视图模型
- [ ] 定义诊断模型
- [ ] 保持本地与 marketplace Skill 的兼容字段映射

### 9.3 详情展示

- [ ] 显示基本信息与来源信息
- [ ] 显示 Markdown 正文
- [ ] 显示 frontmatter 与结构化元数据
- [ ] 显示 supporting files 与执行约束
- [ ] 显示 diagnostics 与原始配置

### 9.4 诊断与容错

- [ ] 解析失败时仍可展示基础信息
- [ ] 返回 warnings 与 errors
- [ ] 返回 missing fields 与 parser version
- [ ] 将 raw 内容作为高级调试兜底

### 9.5 集成

- [ ] 接入现有 `useAgentDiscovery()` / `useAgentWorkspace()` 聚合链路
- [ ] 保持与当前 `ResourceKind = "skill"` 的资源体系一致
- [ ] 复用现有三栏布局与详情面板结构
- [ ] 为后续搜索、过滤和增强详情保留空间

---

## 10. 推荐实现顺序

后续开发可以按下面顺序推进：

1. 定义 Skill 发现模型、详情模型与诊断模型
2. 定义 `SKILL.md` 解析与归一化规则
3. 实现后端目录扫描、parser 与 DTO
4. 实现前端 detail view model 映射
5. 接入当前 `useAgentDiscovery()` / `useAgentWorkspace()` 链路
6. 实现概览、来源与正文区块
7. 实现结构化元数据、supporting files 与 diagnostics 区块
8. 完善搜索、过滤与本地/marketplace 兼容展示

---

## 11. 对当前项目的直接指导结论

结合当前项目现状，后续 Skills 开发应遵循以下原则：

1. 不要把当前 `SkillResource` 直接等同于完整的发现/解析模型。
2. 应明确分离发现结果、解析结果与 UI 展示结果。
3. 应建立发现模型、详情模型、视图模型与诊断模型，避免前端直接依赖 parser 原始结果。
4. diagnostics 应作为正式区块存在，而不是附属信息。
5. 技能详情增强应建立在现有三栏资源工作台结构之上，而不是另起新页面体系。
6. 本地 Skill 与 marketplace Skill 字段语义不同，详情模型必须兼容两类来源。

---

## 12. 参考资料

- `src/pages/home.tsx`
- `src/features/agents/use-agent-workspace.ts`
- `src/features/agents/discovery.ts`
- `src/features/agents/types.ts`
- `src/features/agents/resource-panel.tsx`
- `src/features/agents/detail-panel.tsx`
- `docs/MCP_DEVELOPMENT_GUIDE.md`
- `docs/SUBAGENTS_DEVELOPMENT_GUIDE.md`
