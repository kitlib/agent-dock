# skills.sh 集成方案

## 1. 目标

为 AgentDock 的 Marketplace 能力设计一套可落地的 `skills.sh` 集成方案，用于支持：

- 榜单浏览
- 关键词搜索
- 技能详情展示
- 从 marketplace 安装技能到本地资源池
- 已安装技能的更新检查与更新
- 与本地 Agent / 资源池模型对齐

本方案参考 `skills-manager` 的源码实现，重点提炼其已验证的接入路径，并结合 AgentDock 当前架构给出适配建议。

---

## 2. 结论摘要

`skills-manager` 对接 `skills.sh` 的核心思路可以概括为：

> `skills.sh` 负责发现，GitHub 负责实际安装与更新。

具体表现为：

1. 榜单页通过请求 `skills.sh` 页面 HTML 并解析页面注水数据获得技能列表。
2. 搜索通过请求 `skills.sh` 的搜索接口获得技能列表。
3. 安装时并不直接从 `skills.sh` 下载技能包，而是根据返回的 `source` 拼出 GitHub 仓库地址后执行 clone。
4. 更新检查与更新也不再依赖 `skills.sh`，而是直接基于 Git 仓库 revision 做比较与重装。

这个方案有两个优点：

- 对 `skills.sh` 的依赖面较小，主要集中在“发现”能力
- 安装与更新逻辑统一复用现有 Git 安装链路，降低实现复杂度

---

## 3. 参考实现中的关键文件

以下文件来自本地分析的 `skills-manager` 仓库，可作为 AgentDock 设计参考。

### 3.1 skills.sh 适配层

- `D:/Workspace/AI/skills-manager/src-tauri/src/core/skillssh_api.rs`

职责：

- 定义 `SkillsShSkill` 数据结构
- 封装榜单抓取逻辑
- 封装搜索接口调用逻辑
- 解析 `skills.sh` 页面中的技能数据

### 3.2 browse 命令层

- `D:/Workspace/AI/skills-manager/src-tauri/src/commands/browse.rs`

职责：

- 提供 `fetch_leaderboard`
- 提供 `search_skillssh`
- 为前端页面提供 Tauri 命令入口
- 添加基础缓存能力

### 3.3 安装命令层

- `D:/Workspace/AI/skills-manager/src-tauri/src/commands/skills.rs`

职责：

- 提供 `install_from_skillssh`
- 将 `skills.sh` skill 映射为 GitHub 仓库安装
- 存储 source metadata
- 支持更新检查和更新

### 3.4 Marketplace 前端页面

- `D:/Workspace/AI/skills-manager/src/views/InstallSkills.tsx`

职责：

- 展示榜单 / 搜索结果
- 调用 Tauri 命令加载 skills.sh 数据
- 发起安装
- 展示安装进度和错误反馈

### 3.5 前端 Tauri API 封装

- `D:/Workspace/AI/skills-manager/src/lib/tauri.ts`

职责：

- 统一封装前端到后端的命令调用
- 暴露 `fetchLeaderboard`、`searchSkillssh`、`installFromSkillssh`

---

## 4. skills.sh 可用接口与数据来源

## 4.1 榜单页面

参考实现中使用了以下页面：

- `https://skills.sh/`
- `https://skills.sh/trending`
- `https://skills.sh/hot`

用途：

- all time 榜单
- trending 榜单
- hot 榜单

### 接入方式

不是调用独立榜单 API，而是：

1. 拉取页面 HTML
2. 尝试从 Next.js 的 `__NEXT_DATA__` 中读取技能数组
3. 若失败，再从页面嵌入对象中用正则提取技能信息

### 风险

这种方式依赖 `skills.sh` 页面结构，稳定性弱于正式 API。

因此在 AgentDock 中应将其视为：

- 可用的榜单获取方案
- 但需要容错、缓存与降级策略

---

## 4.2 搜索接口

参考实现使用：

- `https://skills.sh/api/search?q=<query>&limit=<limit>`

### 返回兼容处理

参考实现兼容两种返回结构：

1. 顶层直接返回数组
2. 顶层对象中包含 `skills` 数组

说明：

`skills.sh` 搜索接口可能存在返回格式演进，因此 AgentDock 应保留类似兼容逻辑。

---

## 4.3 skills.sh skill 数据结构

参考实现中的核心字段为：

```ts
interface SkillsShSkill {
  id: string
  skill_id: string
  name: string
  source: string
  installs: number
}
```

字段语义建议定义为：

- `source`: 技能来源仓库，格式通常为 `owner/repo`
- `skill_id`: 技能 ID
- `id`: 组合主键，格式为 `source/skill_id`
- `name`: 展示名
- `installs`: 安装量

---

## 5. 参考实现的数据流

## 5.1 榜单浏览数据流

```txt
Frontend Marketplace
  -> Tauri command: fetch_leaderboard(board)
    -> browse.rs
      -> skillssh_api.fetch_leaderboard(board)
        -> GET skills.sh leaderboard HTML
        -> parse __NEXT_DATA__ / embedded JSON
        -> return SkillsShSkill[]
  -> Frontend render list
```

### 要点

- 榜单适合做缓存
- 榜单适合做只读浏览，不适合作为安装凭据来源
- 解析失败时必须返回可理解的错误信息

---

## 5.2 搜索数据流

```txt
Frontend Marketplace Search
  -> Tauri command: search_skillssh(query, limit)
    -> browse.rs
      -> skillssh_api.search_skills(query, limit)
        -> GET skills.sh/api/search
        -> parse JSON
        -> return SkillsShSkill[]
  -> Frontend render result list
```

### 要点

- 搜索应带 debounce
- 搜索结果适合做短期缓存
- limit 应在后端做上限保护

---

## 5.3 skills.sh 安装数据流

```txt
User clicks install
  -> Frontend call install_from_skillssh(source, skill_id)
    -> Backend build repo URL: https://github.com/{source}.git
    -> clone repo to temp dir
    -> locate skill directory by skill_id
    -> copy/install into central local repo
    -> store source metadata
    -> refresh local resource list
```

### 关键点

- `skills.sh` 安装并非“下载 marketplace 包”
- 本质是根据 `source` 转 Git 仓库地址后执行 Git 安装
- `skill_id` 仅用于定位仓库中的 skill 目录

---

## 5.4 更新检查数据流

```txt
User clicks check update
  -> Backend load installed skill metadata
  -> If source_type in [git, skillssh]
    -> convert to git source
    -> resolve remote revision
    -> compare with local revision
    -> mark up_to_date / update_available / error
```

### 关键点

- `skillssh` 类型在更新层面与 `git` 类型共享一套逻辑
- `skills.sh` 不参与更新检查
- 更新检查依赖本地记录的：
  - source_ref
  - source_ref_resolved
  - source_subpath
  - source_revision
  - remote_revision

---

## 5.5 更新执行数据流

```txt
User clicks update
  -> Backend rebuild git source from skill metadata
  -> clone repo
  -> checkout target revision
  -> locate skill directory
  -> install to staged path
  -> swap current directory atomically
  -> resync copy targets if needed
  -> update metadata
```

### 关键点

- 使用 staged path 再替换当前目录，避免更新中断导致资源损坏
- 更新后需要刷新本地资源池状态
- 如果资源已同步到 Agent 目录，copy 模式应补一次重同步

---

## 6. 对 AgentDock 的落地建议

结合 AgentDock 当前目标，建议把 `skills.sh` 集成拆成 4 层。

## 6.1 Provider 适配层

建议新增后端模块：

```txt
src-tauri/src/marketplace/providers/skillssh.rs
```

职责：

- 请求 `skills.sh` 榜单页
- 请求 `skills.sh` 搜索接口
- 解析返回数据
- 将外部结构转换为 AgentDock 内部 marketplace item 结构

建议内部接口：

```ts
listLeaderboard(board)
search(query, limit)
normalizeSkill(item)
```

这样后续如果接入其他 source，不需要污染上层页面。

---

## 6.2 Marketplace 服务层

建议新增后端模块：

```txt
src-tauri/src/marketplace/service.rs
```

职责：

- 聚合 provider 返回结果
- 做缓存
- 做 source 启用/禁用判断
- 统一错误输出
- 对外暴露给 Tauri command 使用

建议由它提供：

- 获取榜单
- 搜索资源
- 获取详情（首期可先不做远端详情接口，先展示列表基础信息）
- 安装 marketplace item

---

## 6.3 安装桥接层

建议复用你现有或未来将实现的“资源安装管线”，不要单独为 `skills.sh` 写一套本地落盘逻辑。

推荐方案：

- `skills.sh` 只负责返回 `source` 与 `skill_id`
- 后端将其桥接为统一的 Git 安装请求
- 后续所有校验、下载、解压、扫描、入库都走统一安装管线

即：

```txt
skills.sh item
  -> marketplace install adapter
    -> git install request
      -> local resource install pipeline
```

好处：

- 减少重复逻辑
- Marketplace 与 Git 安装统一
- 更新机制天然复用

---

## 6.4 前端页面层

建议在 AgentDock 的 `Marketplace` 页面中，将 `skills.sh` 视为一个 source provider，而不是特判逻辑。

建议页面状态至少包含：

- 当前 source
- 榜单类型
- 搜索词
- 搜索结果
- 安装中状态
- 安装错误
- 已安装状态

建议交互：

- 默认展示榜单
- 输入关键词后自动进入搜索模式
- 卡片展示 name / source / installs
- 安装后允许：
  - 仅加入本地资源池
  - 立即绑定到当前 Agent

这与 `docs/FEATURE_DESIGN.md` 中“Marketplace 辅助导入”的定位一致。

---

## 7. AgentDock 推荐数据模型

为避免后续扩展困难，建议在 AgentDock 内部统一定义 Marketplace item，而不是直接把 `SkillsShSkill` 透传到所有前端逻辑。

建议模型：

```ts
interface MarketplaceItem {
  provider: "skillssh"
  remoteId: string
  name: string
  sourceRepo: string
  externalUrl: string
  installs: number | null
  resourceType: "skill"
  installKind: "git"
  installRef: string
  subpathHint?: string | null
}
```

字段建议：

- `provider`: 来源 provider
- `remoteId`: 如 `owner/repo/skill_id`
- `sourceRepo`: 如 `owner/repo`
- `externalUrl`: 指向 skills.sh 或 GitHub 页面
- `installKind`: 首期固定为 `git`
- `installRef`: Git clone 所需标识
- `subpathHint`: 若未来 provider 可直接返回 skill 目录提示，可用于加速定位

这样未来接入其他 marketplace 时，前端页面无需重写。

---

## 8. 建议的模块落位

结合当前项目结构，建议如下。

### 前端

```txt
src/features/marketplace/
  api/
    marketplace.ts
  components/
    marketplace-search-bar.tsx
    marketplace-source-tabs.tsx
    marketplace-skill-card.tsx
    marketplace-detail-panel.tsx
  hooks/
    use-marketplace-query.ts
  types/
    marketplace.ts
```

### 后端

```txt
src-tauri/src/marketplace/
  mod.rs
  service.rs
  models.rs
  providers/
    mod.rs
    skillssh.rs
```

### Tauri command

```txt
src-tauri/src/commands/marketplace.rs
```

职责：

- `list_marketplace_items`
- `search_marketplace_items`
- `install_marketplace_item`
- `check_marketplace_source_health`

---

## 9. 缓存与容错建议

## 9.1 缓存

建议：

- 榜单缓存：5 分钟
- 搜索缓存：1 到 2 分钟
- 失败请求不长期缓存

缓存位置可先放本地 SQLite 或内存缓存，首期以简单为主。

---

## 9.2 容错

由于榜单页依赖 HTML 解析，必须考虑结构变化。

建议：

1. 先解析 `__NEXT_DATA__`
2. 再解析嵌入 JSON / RSC 片段
3. 再失败则返回明确错误：
   - `skills.sh response format changed`
4. 前端显示 source 不可用状态，而不是空白页

---

## 9.3 健康检查

既然 `FEATURE_DESIGN.md` 里已经定义了 source 健康检查，`skills.sh` source 可实现：

- 请求首页或搜索接口
- 校验是否返回可解析内容
- 记录成功/失败时间
- 在设置页展示 source 状态

---

## 10. 安全与稳定性建议

## 10.1 不信任 marketplace 页面内容

不要把 `skills.sh` 返回内容直接当成已验证安装源。

安装前至少应做：

- 校验 `source` 格式是否符合 `owner/repo`
- 校验 `skill_id` 非空
- Git URL 白名单化为 GitHub HTTPS URL

---

## 10.2 安装阶段继续做本地安全校验

如果复用现有安装管线，建议保持以下保护：

- 只允许安装合法 skill 目录
- 忽略符号链接
- archive 解压时防 Zip Slip
- 使用 staged 目录再替换正式目录

这些在参考实现中都是值得保留的。

---

## 10.3 不把 source 页面结构耦合到业务模型

前端与业务逻辑不要依赖 `skills.sh` 页面字段细节。

原则应是：

- provider 层负责解析外部结构
- service 层负责统一模型
- UI 只消费内部模型

---

## 11. 首期实施建议

建议首期只做最小闭环：

### Phase 1

- 接入 `skills.sh` 榜单浏览
- 接入 `skills.sh` 搜索
- Marketplace 列表页展示 skills

### Phase 2

- 支持从 `skills.sh` 安装到本地资源池
- 安装后刷新本地资源池
- 支持“安装后绑定到当前 Agent”

### Phase 3

- 支持 `skillssh` 类型资源的更新检查
- 支持一键更新
- 支持 source 健康检查与错误展示

### 首期不建议做

- 复杂详情页抓取
- 多 provider 聚合排序
- skills.sh 账号态能力
- 评分、评论、推荐

---

## 12. 建议的最小接口清单

针对 AgentDock，建议首期后端至少提供：

```txt
listMarketplaceLeaderboard(source, board)
searchMarketplaceItems(source, query, limit)
installMarketplaceItem(provider, remoteId, options)
checkMarketplaceItemUpdate(localResourceId)
updateMarketplaceItem(localResourceId)
checkMarketplaceSourceHealth(source)
```

如果当前只支持 `skills.sh`，也建议保留 `source/provider` 参数，避免未来重构接口。

---

## 13. 最终建议

如果 AgentDock 要快速落地 `skills.sh` 集成，最稳妥的做法不是去实现一套“skills.sh 专属安装系统”，而是采用以下原则：

1. `skills.sh` 只做发现层
2. Git 安装作为统一落地层
3. Marketplace item 统一映射为内部模型
4. 更新检查统一复用 Git 更新逻辑
5. 榜单与搜索做好缓存、容错和健康检查

这样可以最小成本打通“发现 -> 安装 -> 绑定 -> 更新”的完整闭环，并且与 AgentDock 当前“本地资源池为主、Marketplace 为辅”的方向保持一致。
