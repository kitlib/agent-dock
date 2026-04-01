# 本地 Agent 扫描精度增强接入方案

## 1. 目标

为 AgentDock 的本地 Agent 发现链路设计一套可落地的 `applications` crate 接入方案，用于支持：

- 为本地 Agent 扫描补充“已安装应用”信号
- 提高 agent type 识别的命中率与稳定性
- 改善本地 Agent 的名称、安装路径、可读性线索
- 为后续图标、运行状态、前台应用等增强能力预留接入点
- 在不破坏现有扫描模型的前提下，增强本地 Agent 识别精度

本方案重点不是把 `applications` 作为新的主扫描器，而是将其作为现有本地扫描链路的增强型信号源接入。

---

## 2. 结论摘要

`applications` 适合接入 AgentDock，但推荐定位为：

> 本地 Agent 发现链路的辅助发现源，而不是唯一或主导的发现源。

具体判断如下：

1. `applications` 是 Rust crate，能自然接入当前 Tauri 后端，不会改变前端架构。
2. 该 crate 的能力更偏“发现本机已安装应用”，与 AgentDock 当前“扫描本地 agent/agent type 安装痕迹”的需求存在交集。
3. 它可以为现有 agent type 级扫描补充更准确的安装态线索，例如应用名称、可执行位置、图标、运行状态等。
4. 它不适合直接替代现有扫描逻辑，因为“已安装应用发现”与“Agent 资源识别”并不完全等价。
5. 最合适的落地方式是：保持 `agent_type_scanner` 为主，新增 `applications` 适配层，在 service 层做归并、去重和置信度提升。

这个方案的优点是：

- 对当前架构侵入小
- 可以逐步上线，不需要一次性重构整个扫描链路
- 即使 `applications` 在某个平台返回质量一般，也不会破坏现有功能
- 便于后续继续增强图标、运行状态、安装来源等体验

---

## 3. 当前实现现状

AgentDock 当前已经具备完整的本地 Agent 发现、解析和导入链路。

### 3.1 命令入口

- `src-tauri/src/commands/agents.rs`

当前 Tauri 命令包括：

- `list_managed_agents`
- `list_resolved_agents`
- `scan_agents`
- `refresh_agent_discovery`
- `import_agents`
- `remove_managed_agent`
- `delete_agent`
- `create_agent`

这些命令已经构成前端与本地 Agent 发现链路之间的稳定接口，因此不建议为了接入 `applications` 改动前端调用模式。

### 3.2 服务层职责

- `src-tauri/src/services/agent_discovery_service.rs`

当前服务层承担的职责包括：

- 驱动扫描逻辑
- 将 discovered agents 与 managed agents 做归并
- 生成 resolved agents
- 生成 import candidates
- 根据 agent type 和 managed 状态补全摘要、角色、状态标签

这说明当前服务层已经是“发现结果归并与整形”的中心位置，因此它也是接入 `applications` 信号的最佳位置。

### 3.3 当前扫描来源

从当前实现命名和调用关系看，本地扫描核心来源是：

- `src-tauri/src/scanners/agent_type_scanner.rs`

也就是说，当前发现链路更偏：

- 基于 agent type 规则
- 基于用户目录和本地安装痕迹
- 基于 AgentDock 已知 agent type 模型进行识别

这个思路适合 AgentDock 当前场景，但局限也明显：

- 只能识别“已知 agent type 的既定痕迹”
- 对安装路径变化、命名差异、平台差异的鲁棒性有限
- 难以充分利用系统层面的“已安装应用”事实

---

## 4. `applications` crate 的适配价值

### 4.1 已确认能力

根据公开元数据与文档，`applications` 当前主要提供：

- 获取已安装应用列表
- 获取运行中的应用列表
- 获取前台应用
- 提供跨平台应用信息对象
- 支持 Linux / macOS / Windows 的差异化实现

其文档与仓库信息表明，这个 crate 的定位是：

> 面向桌面环境的跨平台应用发现库。

### 4.2 与 AgentDock 需求的交集

AgentDock 当前本地 Agent 扫描的核心目标不是“罗列全部应用”，而是：

- 找到本地可管理的 agent/agent type 安装实体
- 提高扫描结果的真实性与可导入性
- 为展示层提供更可信的名称、路径和状态信息

`applications` 可以提供以下增强价值：

1. **补充安装态证据**
   - 当 agent type 规则命中某个路径时，可用 `applications` 提供的已安装应用元数据进行佐证。

2. **改进 display name**
   - 某些 agent type 目录名、可执行文件名并不适合直接展示；系统应用名称通常更适合作为 UI 展示名。

3. **辅助 root path / install path 判断**
   - 当前规则扫描可能拿到的是配置路径或用户目录中的局部路径；系统安装信息可能帮助建立更可靠的安装位置映射。

4. **为图标与运行状态增强铺路**
   - 即使第一阶段不把图标纳入主链路，也可以先在适配层保留字段，为后续 detail panel 或 rail 展示做准备。

5. **提升未知或边缘安装路径的识别率**
   - 当 agent type 规则不够覆盖时，已安装应用列表可以作为候选补全源。

---

## 5. 不推荐的接法

### 5.1 不推荐：直接用 `applications` 替代现有扫描器

原因：

- `applications` 关注的是“系统中有哪些应用”，不是“哪些应用可映射为 AgentDock 的 agent type 资源”
- 系统应用条目里可能包含大量与 agent 无关的信息
- 不同平台的应用数据质量不一致
- 仅依赖该 crate 容易让 AgentDock 的 agent type 识别规则变得模糊

如果直接替代现有扫描器，可能会带来：

- 误识别增加
- 去重逻辑复杂化
- agent type 判断不稳定
- 扫描结果与现有 managed/import 逻辑耦合失衡

### 5.2 不推荐：让前端直接依赖 `applications` 原始模型

原因：

- 前端当前依赖的是 AgentDock 自己的 DTO
- 外部 crate 数据结构不应穿透到 UI 层
- 后续如果替换实现，不应要求前端同步重构

因此推荐始终通过 AgentDock 自己的 DTO 做统一收口。

---

## 6. 推荐架构

推荐采用“保守增强方案”：

```txt
Frontend
  -> Tauri commands (agents.rs)
    -> agent_discovery_service
      -> agent_type_scanner             # 现有主扫描链路
      -> applications adapter           # 新增辅助发现源
      -> merge / dedupe / confidence
      -> ResolvedAgentDto / CandidateDto
```

### 6.1 架构原则

1. **agent type scanner 仍是主来源**
   - 继续承担 agent type 识别、规则命中、已有导入逻辑兼容。

2. **applications adapter 只提供补充信号**
   - 不直接决定最终导入结果。

3. **service 层统一做结果归并**
   - 所有规则优先级、置信度、去重和字段回填，都集中在服务层处理。

4. **DTO 边界保持稳定**
   - 前端继续消费 AgentDock 自己的 DTO，不感知外部 crate 结构。

---

## 7. 推荐落点

### 7.1 依赖接入

- `src-tauri/Cargo.toml`

职责：

- 新增 `applications` crate 依赖
- 仅后端感知，不向前端暴露

### 7.2 适配层

建议新增：

- `src-tauri/src/services/application_catalog_service.rs`
  或
- `src-tauri/src/scanners/application_catalog.rs`

职责：

- 调用 `applications` crate
- 拉取本机已安装应用列表
- 将原始应用模型转换为 AgentDock 内部统一中间结构
- 屏蔽平台差异与 crate 原始字段差异

建议这里不要直接输出 `ResolvedAgentDto`，而是先输出内部中间模型，例如：

- `LocalApplicationRecord`
- `ApplicationEvidence`

这样更利于后续扩展和测试。

### 7.3 服务层归并

- `src-tauri/src/services/agent_discovery_service.rs`

职责：

- 接收 agent type 扫描结果
- 接收 applications 适配层结果
- 做 agent type 命中增强、名称回填、路径归一、去重和置信度处理
- 最终输出现有 `ResolvedAgentDto` / `ScannedAgentCandidateDto`

这是最关键的改动点。

### 7.4 DTO 层

- `src-tauri/src/dto/agents.rs`

第一阶段建议：

- 尽量不破坏现有 DTO
- 只在确有必要时增加少量增强字段

可选增强字段包括：

- `detection_sources`
- `display_icon_hint`
- `install_hint`
- `confidence`

但如果当前前端暂时用不上，第一阶段可以不加，避免扩散修改面。

### 7.5 命令层

- `src-tauri/src/commands/agents.rs`

建议：

- 第一阶段尽量不新增命令
- 复用现有 `scan_agents` / `refresh_agent_discovery` / `list_resolved_agents`
- 由服务层内部完成增强，不改变前端调用方式

这能最大程度降低接入成本。

---

## 8. 推荐数据流

### 8.1 当前数据流

```txt
Frontend
  -> scan_agents(scan_targets)
    -> agent_discovery_service
      -> agent_type_scanner::scan_discovered_agents(scan_targets)
      -> merge with managed agents
      -> return candidate / resolved DTOs
```

### 8.2 增强后数据流

```txt
Frontend
  -> scan_agents(scan_targets)
    -> agent_discovery_service
      -> agent_type_scanner::scan_discovered_agents(scan_targets)
      -> application_catalog_service::list_installed_applications()
      -> correlate agent type results with installed applications
      -> dedupe and enrich fields
      -> merge with managed agents
      -> return candidate / resolved DTOs
```

### 8.3 建议的归并逻辑

建议 service 层按以下优先级处理：

1. **已有 agent type 强规则命中结果优先保留**
2. **若命中应用目录 / 可执行文件 / 显示名对应关系，则补充应用元数据**
3. **若 agent type 规则弱命中，但应用安装信息强匹配，可提高可信度**
4. **若仅有应用信息但无法映射到已知 agent type，不直接生成可导入 agent**

这个原则可以避免把“本机所有应用”误引入 AgentDock 的 agent 候选列表。

---

## 9. 适用边界

### 9.1 适合使用 `applications` 提升的能力

- 已安装应用名称识别
- 安装路径辅助判断
- 图标线索预留
- 运行状态补充
- 前台应用上下文增强
- agent type 规则命中后的可信度提升

### 9.2 不适合让 `applications` 单独负责的能力

- agent type 判定
- AgentDock 导入资格判定
- managed agent 主键生成
- 资源池模型映射
- AgentDock 内部资源分类

这些仍应由 AgentDock 现有规则和服务层负责。

---

## 10. 风险与限制

## 10.1 平台差异

`applications` 的跨平台实现本质上是按平台分头处理，不同平台质量可能不一致。

可能表现为：

- 字段完整度不同
- 路径规则不同
- 返回结果粒度不同
- 某些平台上应用名与可执行路径关联较弱

因此不应假设三端行为完全一致。

## 10.2 应用发现不等于 Agent 发现

系统中的“已安装应用”与 AgentDock 里的“可管理 Agent/agent type”不是同一个概念。

如果没有 service 层的映射与过滤，直接暴露应用列表会引入大量噪音。

## 10.3 图标链路复杂

虽然该 crate 具备图标相关能力，但图标在 Tauri/前端展示中通常涉及：

- 二进制传输方式
- base64 或路径策略
- 平台格式兼容
- 扫描性能

因此图标不适合在第一阶段作为阻塞项。

## 10.4 扫描性能

如果每次刷新都全量扫描 agent type 痕迹并同时全量拉取已安装应用信息，可能带来性能抖动。

建议至少考虑：

- 短期缓存
- 按平台裁剪字段
- 将应用扫描结果作为一次扫描周期内的共享只读数据

## 10.5 文档与生态成熟度一般

从公开信息看，`applications` 并不是高度成熟、广泛验证的基础设施级 crate。

这意味着：

- 适合做能力增强
- 不适合作为不可替代的底层核心依赖

---

## 11. 分阶段落地建议

## Phase 1：最小增强接入

目标：

- 后端接入 `applications`
- 不改前端命令与 UI 结构
- 不引入图标主链路
- 只用于提升本地扫描结果质量

建议动作：

- 新增 Rust 适配层，读取已安装应用列表
- 在 `agent_discovery_service` 中引入归并逻辑
- 仅回填 display name、安装线索、路径匹配信息
- 保持现有 DTO 尽量不变

这是风险最低、收益最稳定的一步。

## Phase 2：可观测性增强

目标：

- 让调试和验证更容易

建议动作：

- 为扫描结果增加内部调试字段或日志
- 标记某个候选是由 agent type 规则命中，还是由应用安装线索增强
- 为后续调优去重和匹配策略提供证据

这一阶段主要服务于算法和规则调优。

## Phase 3：展示能力增强

目标：

- 在 UI 里利用更丰富的应用元数据

建议动作：

- 在 detail panel 中展示更友好的应用名
- 在 agent rail 或详情面板引入图标线索
- 视实际价值决定是否暴露运行状态或前台应用信息

这一阶段应建立在前两阶段已经稳定的基础上。

---

## 12. 推荐实施原则

1. **先增强准确率，再增强表现力**
   - 第一优先级是识别更准，不是 UI 更炫。

2. **先服务层收敛，再考虑 DTO 扩张**
   - 避免为了外部 crate 的字段扩散修改前端。

3. **先保留现有 agent type 扫描主导权**
   - 这样即使新依赖效果一般，也不会造成主链路退化。

4. **把 `applications` 当作可替换适配层**
   - 不要让业务层直接依赖其原始数据模型。

---

## 13. 最终建议

对于 AgentDock 当前“本地 Agent 扫描更精准”的目标，`applications` 是值得接入的。

但最合理的定位不是：

- 新的主扫描器
- 直接暴露给前端的应用模型
- 唯一可信的数据来源

而是：

- 一个 Rust 后端可控、低侵入的辅助发现源
- 一个提升本地 Agent/agent type 识别质量的增强模块
- 一个为图标、运行状态、安装态展示预留能力的扩展点

因此推荐采用：

> 保留现有 agent type 扫描主链路，新增 `applications` 适配层，在 `agent_discovery_service` 中统一归并。

这条路线最符合 AgentDock 现有架构，也最适合在不破坏现有 Agent 导入与管理逻辑的前提下，逐步提升本地扫描精度。
