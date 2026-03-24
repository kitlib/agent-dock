# AgentDock 功能实现方案

## 1. 目标
围绕 `docs/FEATURE_DESIGN.md` 中定义的功能，分阶段实现一个以多 Agent 为核心的本地资源管理系统，支持：

- 多 Agent 管理
- 本地资源池管理
- Agent 资源绑定
- 分组与拖拽编排
- 批量操作
- Marketplace 辅助导入
- 市场源配置

本方案聚焦实现路径与模块拆分，不展开过多设计背景。

---

## 2. 实现原则

- 以本地资源管理为主，Marketplace 为辅
- 先做主流程，再补高级能力
- 先打通数据结构和页面骨架，再接真实能力
- 明确区分“资源本体”和“Agent 绑定关系”
- 明确区分“删除绑定”和“删除资源”
- 所有高风险删除操作都需要确认

---

## 3. 实现范围

### 首期实现
- Agents 页面
- Resources 页面
- Marketplace 页面
- Agent CRUD
- 本地资源池展示
- 资源绑定与解绑
- 启用 / 禁用
- 分组管理
- 基础拖拽排序
- 基础批量操作
- skillhub 接入
- source 配置管理

### 后续实现
- MCP 在线安装
- Subagents 在线安装
- 自动依赖安装
- 多 provider 聚合搜索
- 云端同步
- 推荐与评分

---

## 4. 模块拆分

## 4.1 前端模块
建议拆成以下模块：

```txt
src/features/
  agents/
  resource-registry/
  agent-bindings/
  marketplace/
```

### `agents`
负责：
- Agent 列表
- Agent 新增/删除/重命名
- 当前 Agent 状态

### `resource-registry`
负责：
- 本地资源池列表
- 资源详情
- 本地导入
- 资源删除
- 资源使用关系

### `agent-bindings`
负责：
- Agent 绑定资源列表
- 启用 / 禁用
- 分组
- 拖拽排序
- 批量操作

### `marketplace`
负责：
- 市场资源列表
- source 配置
- skillhub 接入
- 安装到本地资源池
- 安装后绑定到 Agent

---

## 4.2 后端模块
建议拆成以下模块：

```txt
src-tauri/src/
  agents/
  resources/
  bindings/
  marketplace/
```

### `agents`
负责 Agent 配置读写。

### `resources`
负责本地资源扫描、导入、删除、状态读取。

### `bindings`
负责 Agent 与资源绑定关系、分组、排序和批量变更。

### `marketplace`
负责 source 配置、provider 接入和安装逻辑。

---

## 5. 页面实现顺序

## 5.1 Agents 页面
优先实现，因为这是主工作台。

首期页面内容：
- 左侧 Agent 列表
- 顶部筛选与批量操作栏
- 中间资源编排区
- 分组区与未分组区
- 右侧详情区

首批打通能力：
- 切换 Agent
- 查看绑定资源
- 单项启用 / 停用
- 新增绑定
- 移除绑定

---

## 5.2 Resources 页面
第二优先级。

首期页面内容：
- 资源类型切换
- 资源列表
- 资源详情
- 删除资源
- 导入资源
- 查看被哪些 Agent 使用

---

## 5.3 Marketplace 页面
第三优先级。

首期页面内容：
- 市场资源列表
- 搜索
- 详情面板
- 安装按钮
- source 设置入口

首期只打通：
- skillhub
- Skills 资源安装到本地池
- 安装后可绑定到 Agent

---

## 6. 数据实现顺序

建议优先定义以下本地数据结构：

### 6.1 Agent
- id
- name
- description
- createdAt
- updatedAt

### 6.2 LocalResource
- id
- type
- name
- description
- source
- installed
- version
- path
- metadata

### 6.3 AgentResourceBinding
- id
- agentId
- resourceId
- resourceType
- enabled
- order
- groupId
- configOverride

### 6.4 AgentResourceGroup
- id
- agentId
- name
- order
- collapsed

### 6.5 MarketplaceSourceConfig
- id
- name
- provider
- endpoint
- enabled
- resourceTypes
- authType
- healthStatus

---

## 7. 分阶段实施方案

## 阶段 1：基础数据与页面骨架
目标：先把页面和本地数据结构搭起来。

包含：
- Agent 基础模型
- LocalResource 基础模型
- Binding / Group 基础模型
- Agents 页面骨架
- Resources 页面骨架
- Marketplace 页面骨架

完成后应能：
- 看到页面
- 使用 mock 数据切换 Agent
- 展示资源列表和基础详情

---

## 阶段 2：Agent 与资源池基础管理
目标：先打通本地主流程。

包含：
- Agent CRUD
- 本地资源池读取
- 本地资源导入
- 资源删除
- Agent 添加资源
- Agent 移除资源
- 单项启用 / 禁用

完成后应能：
- 创建多个 Agent
- 给 Agent 绑定本地资源
- 启停单个资源
- 删除绑定或删除资源

---

## 阶段 3：分组与拖拽
目标：打通编排能力。

包含：
- 分组 CRUD
- 分组折叠 / 展开
- 未分组区展示
- 同组排序
- 跨组移动
- 分组与未分组区互移

完成后应能：
- 创建分组
- 拖拽资源重新编排
- 保存顺序与分组状态

---

## 阶段 4：批量操作
目标：提升管理效率。

包含：
- 多选模式
- 批量启用
- 批量停用
- 批量移动到分组
- 批量移出分组
- 批量解除绑定
- 可选批量删除资源

完成后应能：
- 对多条绑定执行统一操作
- 保持反馈清晰

---

## 阶段 5：Marketplace 接入
目标：补齐辅助导入能力。

包含：
- source 配置管理
- skillhub provider
- 市场列表与搜索
- 资源详情
- 安装到本地资源池
- 安装后绑定到 Agent

完成后应能：
- 从 Marketplace 安装 skill
- 安装后加入本地资源池
- 可选立即挂到某个 Agent

---

## 阶段 6：收尾与优化
目标：补齐体验与边界。

包含：
- i18n 文案补齐
- 空状态与错误状态
- 删除确认
- MCP 敏感信息隐藏
- 状态刷新与反馈优化

完成后应能：
- 主流程稳定可用
- 删除边界清晰
- 错误提示明确

---

## 8. 文件级建议

## 8.1 前端
建议新增或扩展：

```txt
src/pages/
  agents.tsx
  resources.tsx
  marketplace.tsx

src/features/
  agents/
  resource-registry/
  agent-bindings/
  marketplace/
```

### 重点文件方向
- `agents.tsx`：主工作台
- `resources.tsx`：本地资源池
- `marketplace.tsx`：市场导入页
- `features/agents/*`：Agent 列表与 CRUD
- `features/resource-registry/*`：本地资源池与资源详情
- `features/agent-bindings/*`：绑定、分组、拖拽、批量操作
- `features/marketplace/*`：source、provider、安装导入

---

## 8.2 后端
建议新增或扩展：

```txt
src-tauri/src/
  agents/
  resources/
  bindings/
  marketplace/
```

### 重点职责
- `agents/`：Agent 配置持久化
- `resources/`：资源扫描、导入、删除、状态读取
- `bindings/`：绑定关系、分组、顺序管理
- `marketplace/`：source、provider、安装逻辑

---

## 9. 关键边界

### 9.1 绑定与资源分离
必须区分：
- 从 Agent 中移除绑定
- 删除本地资源本体

### 9.2 Marketplace 与主工作台分离
Marketplace 只负责导入：
- 不承担主编排
- 不承担分组拖拽主逻辑

### 9.3 敏感信息保护
MCP 相关配置默认不展示：
- token
- password
- env value
- header value

### 9.4 高风险操作确认
以下操作需要确认：
- 删除 Agent
- 删除资源
- 批量删除资源
- 删除分组
- 删除 source

---

## 10. 验收标准

### 功能验收
- 支持多个 Agent
- 支持本地资源池管理
- 支持为 Agent 添加/移除资源
- 支持单项启停
- 支持分组与拖拽
- 支持批量启停和批量移动
- 支持从 Marketplace 导入资源

### 体验验收
- 主操作集中在 Agents 页面
- Resource 页面可快速查看全局资源
- Marketplace 不喧宾夺主
- 删除边界清晰
- 状态反馈清楚

### 安全验收
- 敏感配置不明文展示
- 删除操作有明确确认
- 绑定删除与资源删除不会混淆
