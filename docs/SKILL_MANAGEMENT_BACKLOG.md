# Skill 管理功能待办清单

## 说明

本文用于记录 AgentDock 当前 Skill 管理功能的实现状态与后续待办。

范围约束：
- 不包含 Skill 创建能力
- 不包含 Skill 编辑能力
- 当前重点聚焦本地 Skill 管理、全局资源池、Marketplace 接入与关系视图

当前判断：
- 现状更接近“本地 Skill 浏览 + 基础维护”
- 距离“完整 Skill 管理系统”仍缺少独立资源页、真实 Marketplace、关系视图与统一资源模型

---

## 已实现

### A. 全局本地 Skill 资源池（第一阶段）

已实现内容：
- 左侧 Agent 列表顶部支持“全部”入口
- 选中“全部”后，聚合所有已管理且未隐藏 Agent 的本地 Skills
- 中栏继续复用现有 Skill 列表视图
- 右栏支持“全部 Skills”空选中态说明
- 全局视图下，Skill 列表会显示归属 Agent 标签

当前边界：
- 仅 Skill 支持“全部”聚合
- MCP / Subagents 暂未接入
- 仍然位于 Home 工作台内，尚未拆分为独立 Resources 页面

结论：
- 该项已完成第一阶段
- 后续不再按“完全未实现”处理

### B. 本地 Skill 复制闭环（基础版）

已实现内容：
- 支持批量复制本地 Skill 到其它 Agent
- 支持单个 Skill 从列表“更多”菜单发起复制
- 复制目标改为平铺 Agent 卡片，而不是下拉框
- 支持多选目标 Agent
- 支持按目标 Agent 分别预览冲突
- 支持 overwrite / skip 冲突决议
- 弹窗关闭后会重置选择、预览和冲突状态，避免状态残留

当前边界：
- 复制完成后的结果反馈仍偏轻量
- 复制后尚未提供更强的定位反馈，例如自动跳到目标 Agent 或目标 Skill

结论：
- 基础复制链路已完成
- 后续只剩体验增强项

### C. 本地 Skill 操作体验优化

已实现内容：
- 列表“更多”菜单顺序统一为：打开、编辑、复制、停用/启用、删除
- 右栏详情操作顺序统一为：打开、编辑、停用/启用、删除
- 删除入口后置，降低误触风险
- 复制弹窗预留状态区域高度，避免选择目标后弹窗高度抖动
- 复制目标卡片副标题改为优先显示别名，其次显示目录名，避免重复显示 agent type

---

## P0

### 1. 独立 Resources 页面

目标：
- 将“全局本地 Skill 资源池”从 Home 工作台中拆出，形成独立资源管理页面

当前缺口：
- 目前只有 Home 内的“全部”视图，没有独立页面入口

验收标准：
- 新增独立 `Resources` 页面入口
- 页面可查看全局本地 Skill 列表与详情
- 尽量复用现有资源列表和详情组件，避免重复实现

涉及文件：
- `src/main.tsx`
- `src/pages/`
- `src/features/resources/`
- `src/features/home/`

复杂度：
- 中

风险：
- 低到中，主要是状态复用和入口组织

### 2. Marketplace 真接入

目标：
- 用真实远程数据替代当前 mock Skill 市场
- 支持真实搜索、浏览与安装

当前缺口：
- 前端 Marketplace 仍为 mock
- 安装按钮只切换前端状态
- 后端缺少 provider、command 和安装落盘链路

验收标准：
- 可从真实 source 获取 Skill 列表
- 可搜索远程 Skills
- 可执行真实安装，而不是只修改前端状态
- 安装结果有明确成功或失败反馈

涉及文件：
- `src/features/marketplace/mock.ts`
- `src/features/home/use-resource-browser.ts`
- `src/features/resources/core/discovery.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/`
- `src-tauri/src/services/`

参考文档：
- `docs/SKILLS_SH_INTEGRATION_PLAN.md`

复杂度：
- 高

风险：
- 高，涉及网络访问、数据归一化、安装落盘与错误处理

### 3. 安装后刷新本地 Skill 池

目标：
- Marketplace 安装完成后，本地 Skill 列表自动刷新

当前缺口：
- 当前“安装”仍是前端 install state 变化
- 没有真实安装后的本地资源刷新逻辑

验收标准：
- 安装完成后，Skill 出现在本地资源池中
- 可立即查看详情、启停、删除、复制

涉及文件：
- `src/features/home/use-resource-browser.ts`
- `src/features/home/queries.ts`
- `src/features/agents/api.ts`
- `src-tauri/src/commands/`

复杂度：
- 中

风险：
- 中，依赖 Marketplace 真接入

---

## P1

### 4. Skill 详情增强

目标：
- 将后端已解析的结构化信息完整展示到详情面板

当前缺口：
- 当前主要展示 `description`、`markdown`、简单 `diagnostics`
- `frontmatter`、`frontmatterRaw`、`supportingFiles`、`allowedTools` 仍未完整进入 UI

验收标准：
- 显示结构化元数据
- 显示 supporting files
- 显示 allowed tools
- 显示 warnings / errors
- 保持当前 Markdown 展示能力

涉及文件：
- `src/features/resources/core/components/resource-detail.tsx`
- `src/features/home/components/detail-panel.tsx`
- `src-tauri/src/scanners/skill_scanner.rs`
- `src/features/agents/types.ts`

复杂度：
- 低到中

风险：
- 低

### 5. “被哪些 Agent 使用”关系视图

目标：
- 从“Skill 属于哪个扫描源”扩展到“Skill 被哪些 Agent 使用”的视角

当前缺口：
- 现有 `ownerAgentId` 只表示扫描归属
- 没有多 Agent 绑定或使用关系模型

验收标准：
- 能区分“来源 Agent”和“使用 Agent”
- 至少有一套只读关系视图
- 不与复制后的文件存在状态混淆

涉及文件：
- `src/features/agents/types.ts`
- `src/features/home/`
- `src/features/resources/`
- `src-tauri/src/dto/`
- `src-tauri/src/services/`

复杂度：
- 中到高

风险：
- 中，容易把目录存在关系误当成逻辑绑定关系

### 6. 批量删除

目标：
- 补齐本地 Skill 管理的基础批量删除能力

当前缺口：
- 已有批量启停、批量复制
- 缺少批量删除

验收标准：
- 可对选中的多个本地 Skills 执行删除
- 删除前有明确确认
- 删除后列表与详情状态能正确刷新

涉及文件：
- `src/pages/home.tsx`
- `src/features/home/components/resource-panel.tsx`
- `src/features/resources/core/components/resource-list.tsx`
- `src-tauri/src/commands/skills.rs`

复杂度：
- 低到中

风险：
- 中，属于高风险删除操作，交互必须谨慎

### 7. 复制结果反馈增强

目标：
- 让复制能力从“可用”提升到“可管理”

当前缺口：
- 已有冲突预览
- 仍缺少更明确的复制结果反馈与复制后的定位体验

验收标准：
- 清楚显示复制成功、跳过、覆盖的结果
- 复制后可快速定位到目标 Skill 或目标 Agent

涉及文件：
- `src/features/home/components/copy-skill-dialog.tsx`
- `src/pages/home.tsx`
- `src/features/home/use-home-workspace.ts`

复杂度：
- 低

风险：
- 低

---

## P2

### 8. 分组管理

目标：
- 支持未分组区、分组、分组排序

当前缺口：
- 设计中已有分组概念
- 代码中尚无实际实现

验收标准：
- 可创建和维护分组
- Skill 可进入分组或回到未分组区
- 分组状态可持久化

涉及文件：
- `src/features/home/`
- `src/features/resources/`
- `src-tauri/src/persistence/`
- `src-tauri/src/services/`

复杂度：
- 高

风险：
- 中到高

### 9. 拖拽编排

目标：
- 支持拖拽排序、拖入分组、跨组移动

当前缺口：
- 列表项已支持 `draggable`
- 但没有 drop 目标与实际编排逻辑

验收标准：
- 可拖拽 Skill 进行重排
- 可拖拽进入分组
- 可跨组移动

涉及文件：
- `src/features/resources/core/components/resource-list.tsx`
- `src/features/home/`
- `src/features/resources/`

复杂度：
- 中到高

风险：
- 中

### 10. Marketplace Source 管理

目标：
- 支持 source 配置、启停、健康检查与错误展示

当前缺口：
- 文档中已有设计
- 代码中尚无实际实现

验收标准：
- 可配置 source
- 可查看 source 健康状态
- source 错误会反馈到 UI

涉及文件：
- `src/features/marketplace/`
- `src/pages/settings.tsx`
- `src-tauri/src/commands/`
- `src-tauri/src/services/`

复杂度：
- 高

风险：
- 中

### 11. 统一资源管理收口

目标：
- 让 Skill、MCP、Subagent 逐步收口到同一套真实资源管理模型

当前缺口：
- Skill 已有较真实链路
- MCP 和 Subagent 仍有大量 mock 占位

验收标准：
- 三类资源的本地与远程数据流不再严重分裂
- 资源管理层可复用统一模式

涉及文件：
- `src/features/resources/core/resource-catalog.ts`
- `src/features/resources/core/discovery.ts`
- `src/features/marketplace/`
- `src-tauri/src/`

复杂度：
- 高

风险：
- 中到高

---

## 建议里程碑

### 里程碑 1

- 独立 Resources 页面
- Skill 详情增强

### 里程碑 2

- Marketplace 真接入
- 安装后刷新本地 Skill 池

### 里程碑 3

- “被哪些 Agent 使用”关系视图
- 批量删除
- 复制结果反馈增强

### 里程碑 4

- 分组管理
- 拖拽编排
- Marketplace Source 管理

---

## 当前优先顺序建议

1. 独立 Resources 页面
2. Skill 详情增强
3. Marketplace 真接入
4. 安装后刷新本地 Skill 池
5. “被哪些 Agent 使用”关系视图
6. 批量删除
7. 复制结果反馈增强
8. 分组管理
9. 拖拽编排
10. Marketplace Source 管理
11. 统一资源管理收口
