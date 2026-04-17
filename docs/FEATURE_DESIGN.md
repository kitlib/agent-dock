# AgentDock 多 Agent 资源管理功能设计

## 功能定位

AgentDock 当前的核心目标是成为一个面向多 Agent 的本地资源管理中心。

当前产品重点：

- 管理本地 Agent
- 浏览和维护本地 Skills
- 在统一工作区内承载 Skills、MCP、Subagents 三类资源
- 通过 Marketplace 作为补充导入入口

当前产品决策：

- `Home / Agents` 是当前唯一主工作台
- 不再规划独立 `Resources` 页面
- Marketplace 已对 `skills.sh` 打通真实链路，但仅覆盖 Skill

## 当前代码实现现状（基于代码核对，2026-04-17）

### 1. Agents / Home 工作台

当前已实现：

- 左侧 Agent Rail 支持折叠
- 左侧顶部提供“全部”入口
- 选中“全部”时，可在 Home 内查看所有已管理且未隐藏 Agent 的聚合 Skill 列表
- 选中单个 Agent 时，可查看该 Agent 的本地资源
- 工作区支持 `browse / adding` 两种模式切换

当前边界：

- “全部”聚合目前只覆盖 Skill
- MCP / Subagent 仍是占位资源模型，未形成真实本地发现链路

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

- `commands` 类型的 Markdown 文件也被纳入 Skill 资源视图
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

- 后端不再返回未被消费的结构化字段
- Skill 详情当前聚焦已有可见信息，不再规划这些冗余解析结果的展示

### 4. Skill 复制链路

当前已实现：

- 批量复制本地 Skill 到其他 Agent
- 单个 Skill 从列表“更多”菜单发起复制
- 复制目标改为平铺 Agent 卡片，而不是下拉框
- 支持多选目标 Agent
- 支持冲突预览
- 支持 `overwrite / skip`
- 复制完成后刷新本地 Skill 列表与 Agent 列表

当前边界：

- 复制完成后的反馈仍较轻量，主要是 toast
- 尚未提供“跳转到目标 Agent / 目标 Skill”的定位反馈

### 5. Marketplace（Skill）

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
- `mcp / subagent` 仍使用前端 mock 数据
- Source 管理、健康检查、配置页尚未完成

结论：

- “Marketplace 真接入未完成”这个旧判断不再准确
- 更准确的描述应是：“Skill Marketplace 已接入，统一 Marketplace 仍未完成”

### 6. 列表与详情操作顺序

当前已实现：

- 列表“更多”菜单顺序统一为：打开、编辑、复制、停用 / 启用、删除
- 右侧详情操作顺序统一为：访问 / 安装、更新、打开、编辑、停用 / 启用、删除
- 删除操作在详情侧有确认弹窗

当前边界：

- 列表项中的删除入口仍然是直接执行，未经过二次确认

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
- MCP / Subagent 仍主要是占位模型与展示壳

### 3. Agent 资源绑定管理

当前已实现：

- 通过复制将 Skill 放入目标 Agent
- 对复制冲突执行预览与决策

当前未实现：

- 显式“绑定关系”模型
- 从 Agent 解除绑定
- 绑定顺序调整
- 只读“被哪些 Agent 使用”关系视图

### 4. 分组管理

规划支持但尚未实现：

- 创建分组
- 重命名分组
- 删除分组
- 折叠 / 展开分组
- 分组排序

### 5. 拖拽编排

当前现状：

- 列表项具备 `draggable`

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

未完成：

- Skill 使用关系视图
- 批量删除
- 复制结果反馈增强
- 分组管理
- 拖拽编排
- MCP / Subagent 的真实本地与 Marketplace 管理链路

## 使用原则

- 日常主操作优先在 `Home / Agents` 中完成
- “全部”视图当前用于聚合本地 Skill
- Marketplace 目前是 Skill 的真实获取入口，不承担完整资源编排职责
- 删除绑定与删除资源本体必须明确区分
- 危险删除操作应统一具备确认
