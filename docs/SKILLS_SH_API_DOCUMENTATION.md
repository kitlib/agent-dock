# skills.sh API 文档

## 1. 文档范围

本文档基于两类证据整理：

- `vercel-labs/skills` CLI 源码中实际调用的远端接口
- 2026-04-17 对 `https://skills.sh` 的黑盒探测结果

本文档只记录当前已确认的接口行为，不对未公开的服务端实现做推断性描述。

---

## 2. 结论摘要

截至 2026-04-17，已确认可直接访问的 `skills.sh` API 有 4 类：

1. `GET /api/skills/{board}/{page}`
2. `GET /api/search`
3. `GET /api/download/{owner}/{repo}/{slug}`
4. `GET /api/audit?source=...&skills=...`

其中：

- `skills`、`search` 和 `download` 已可在线上直接访问
- `search` 和 `download` 已被 `vercel-labs/skills` CLI 明确使用
- `audit` 已在线上可访问，但当前 `vercel-labs/skills` CLI 仓库并未直接调用 `skills.sh/api/audit`
- 额外探测到 `GET /api/skill/{owner}/{repo}/{slug}` 返回 `401`，说明该路由可能存在，但当前无法作为公开接口使用

---

## 3. 探测方法

### 3.1 源码依据

来自 `vercel-labs/skills` 仓库的关键位置：

- `src/find.ts`
  - `SEARCH_API_BASE = https://skills.sh`
  - 调用 `/api/search?q=...&limit=10`
- `src/blob.ts`
  - `DOWNLOAD_BASE_URL = https://skills.sh`
  - 调用 `/api/download/{owner}/{repo}/{slug}`

### 3.2 黑盒探测

已对以下候选路径进行探测：

- 存在：`/api/skills/{board}/{page}`、`/api/search`、`/api/download/...`、`/api/audit`
- 需鉴权或疑似内部：`/api/skill/...`
- 不存在：`/api/health`、`/api/docs`、`/api/openapi.json`、`/api/swagger.json`、`/api/trending`、`/api/hot`

---

## 4. 已确认公开接口

## 4.1 榜单接口

### 路径

`GET /api/skills/{board}/{page}`

### Path 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `board` | `string` | 是 | 榜单类型，当前已验证为 `all-time`、`trending`、`hot` |
| `page` | `number` | 是 | 页码，从 `0` 开始 |

### 已验证行为

- `GET /api/skills/all-time/0` 返回 `200`
- `GET /api/skills/trending/0` 返回 `200`
- `GET /api/skills/hot/0` 返回 `200`
- `GET /api/skills/all-time/1` 返回 `200`
- `GET /api/skills/trending/1` 返回 `200`
- `GET /api/skills/hot/1` 返回 `200`

### 已验证返回特征

- 返回 JSON
- 顶层字段当前观测到至少包含：
  - `skills`
- `skills` 为数组
- 当前实测 `page=0` 时，3 种榜单都返回了 `200` 条记录
- 当前实测响应里未发现稳定的 `totalSkills` 或 `total_skills` 字段

### 响应示例

请求：

```http
GET https://skills.sh/api/skills/all-time/0
```

响应：

```json
{
  "skills": [
    {
      "source": "vercel-labs/skills",
      "skillId": "find-skills",
      "name": "find-skills",
      "installs": 1074251
    },
    {
      "source": "vercel-labs/agent-skills",
      "skillId": "vercel-react-best-practices",
      "name": "vercel-react-best-practices",
      "installs": 324388
    }
  ]
}
```

### 字段说明

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `skills[].source` | `string` | 技能来源仓库，格式通常为 `{owner}/{repo}` |
| `skills[].skillId` | `string` | 技能 slug |
| `skills[].name` | `string` | 技能名称 |
| `skills[].installs` | `number` | 安装量或榜单统计值 |
| `skills[].installsYesterday` | `number` | 仅部分榜单项存在，例如 `hot` |
| `skills[].change` | `number` | 仅部分榜单项存在，例如 `hot` |

### 集成建议

- 该接口适合直接用于 Marketplace 榜单浏览
- 调用方不要假设存在总数信息
- 调用方不要假设所有榜单项字段完全一致，尤其是 `hot` 榜单可能附带变化量字段

---

## 4.2 搜索接口

### 路径

`GET /api/search`

### Query 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `q` | `string` | 是 | 搜索关键词，长度不足时会报错 |
| `limit` | `number` | 否 | 返回条数 |

### 已验证行为

- `GET /api/search?q=react` 返回 `200`
- `GET /api/search?q=react&limit=2` 返回 `200`
- `GET /api/search?q=react&limit=500` 返回 `200`
- `GET /api/search` 返回 `400`
- `GET /api/search?q=a&limit=1` 返回 `400`
- `GET /api/search?q=react&limit=abc` 返回 `200`，但结果为空

### 已验证返回特征

- 返回 JSON
- 顶层字段至少包含：
  - `query`
  - `searchType`
  - `skills`
  - `count`
  - `duration_ms`
- 当前观测到 `searchType` 为 `fuzzy`

### 响应示例

请求：

```http
GET https://skills.sh/api/search?q=react&limit=2
```

响应：

```json
{
  "query": "react",
  "searchType": "fuzzy",
  "skills": [
    {
      "id": "vercel-labs/agent-skills/vercel-react-best-practices",
      "skillId": "vercel-react-best-practices",
      "name": "vercel-react-best-practices",
      "installs": 324388,
      "source": "vercel-labs/agent-skills"
    },
    {
      "id": "vercel-labs/agent-skills/vercel-react-native-skills",
      "skillId": "vercel-react-native-skills",
      "name": "vercel-react-native-skills",
      "installs": 92797,
      "source": "vercel-labs/agent-skills"
    }
  ],
  "count": 2,
  "duration_ms": 47
}
```

### 字段说明

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `query` | `string` | 原始搜索词 |
| `searchType` | `string` | 当前观测值为 `fuzzy` |
| `skills[].id` | `string` | 完整技能标识，格式通常为 `{source}/{skillId}` |
| `skills[].skillId` | `string` | 技能 slug |
| `skills[].name` | `string` | 技能名称 |
| `skills[].installs` | `number` | 安装量 |
| `skills[].source` | `string` | 技能来源仓库，格式通常为 `{owner}/{repo}` |
| `count` | `number` | 返回项数 |
| `duration_ms` | `number` | 服务端耗时 |

### 集成建议

- 前端应在发起请求前校验 `q.length >= 2`
- `limit` 建议只传数字字符串或整数
- 不要依赖默认条数，调用方应显式传入 `limit`

---

## 4.3 技能快照下载接口

### 路径

`GET /api/download/{owner}/{repo}/{slug}`

### Path 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `owner` | `string` | 是 | 仓库 owner |
| `repo` | `string` | 是 | 仓库名 |
| `slug` | `string` | 是 | 技能 slug |

### 已验证行为

- `GET /api/download/vercel-labs/skills/find-skills` 返回 `200`
- `GET /api/download/google-labs-code/stitch-skills/react%3Acomponents` 返回 `200`
- `GET /api/download/vercel-labs/skills/react:components` 返回 `404`

### 关键结论

- `slug` 中若包含特殊字符，例如 `:`
  - 调用方必须先做 URL 编码
  - 否则可能得到 `404`
- 返回的是技能文件快照，不是单一 `SKILL.md`

### 响应示例

请求：

```http
GET https://skills.sh/api/download/vercel-labs/skills/find-skills
```

响应：

```json
{
  "files": [
    {
      "path": "SKILL.md",
      "contents": "---\nname: find-skills\n..."
    }
  ],
  "hash": "9e1c8b3103f92fa8092568a44fe64858de7c5c9dc65ce4bea8f168080e889cfd"
}
```

### 字段说明

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `files` | `array` | 技能快照内所有文件 |
| `files[].path` | `string` | 文件相对路径 |
| `files[].contents` | `string` | 文件全文内容 |
| `hash` | `string` | 当前快照哈希 |

### 已验证样例

| 请求 | 结果 |
| --- | --- |
| `/api/download/vercel-labs/skills/find-skills` | `200`，`files=1` |
| `/api/download/google-labs-code/stitch-skills/react%3Acomponents` | `200`，`files=11` |

### 集成建议

- 始终对 `owner`、`repo`、`slug` 做 URL 编码
- 安装或预览逻辑应优先消费 `files[]`，不要假设只有 `SKILL.md`
- 可以把 `hash` 作为本地缓存键或快照版本标识

---

## 4.4 安全审计接口

### 路径

`GET /api/audit`

### Query 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `source` | `string` | 是 | 技能来源仓库，例如 `vercel-labs/skills` |
| `skills` | `string` | 是 | 逗号分隔的技能 slug 列表 |

### 已验证行为

- `GET /api/audit?source=vercel-labs/skills&skills=find-skills` 返回 `200`
- `GET /api/audit?source=vercel-labs/skills&skills=find-skills,nonexistent` 返回 `200`
- `GET /api/audit?source=vercel-labs/skills` 返回 `400`
- `GET /api/audit?skills=find-skills` 返回 `400`

### 响应示例

请求：

```http
GET https://skills.sh/api/audit?source=vercel-labs/skills&skills=find-skills
```

响应：

```json
{
  "find-skills": {
    "ath": {
      "risk": "safe",
      "analyzedAt": "2026-03-14T07:45:39.850Z"
    },
    "socket": {
      "risk": "safe",
      "alerts": 0,
      "score": 90,
      "analyzedAt": "2026-03-18T16:47:53.829Z"
    },
    "snyk": {
      "risk": "medium",
      "analyzedAt": "2026-03-14T07:45:26.162192+00:00"
    },
    "zeroleaks": {
      "risk": "safe",
      "score": 93,
      "analyzedAt": "2026-04-16T07:47:59.444Z"
    }
  }
}
```

### 字段说明

响应顶层是一个对象：

- key 为技能 slug
- value 为多个审计来源的结果集合

单个审计来源当前观测到的字段可能包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `risk` | `string` | 风险等级 |
| `alerts` | `number` | 告警数，可选 |
| `score` | `number` | 评分，可选 |
| `analyzedAt` | `string` | 审计时间 |

### 额外观察

- 传入不存在的技能 slug 不会导致整体请求失败
- 当前观测结果中，不存在的 skill key 仍可能出现在响应中，但其内部结构未保证稳定

### 集成建议

- 调用方应把响应视为动态对象，而不是固定 schema
- UI 层应允许“部分审计源缺失”
- 不应假设所有技能都有完整审计结果

---

## 5. 疑似内部或需鉴权接口

以下接口不建议作为产品接入依据，但值得记录：

### 5.1 技能详情接口

请求：

```http
GET https://skills.sh/api/skill/vercel-labs/skills/find-skills
```

观测结果：

- 返回 `401`

推断：

- 该路由可能存在
- 当前需要鉴权或具备额外访问限制
- 不能视为公开可集成接口

---

## 6. 已探测但未发现的接口

以下路由在 2026-04-17 探测结果为 `404`：

- `/api/health`
- `/api/docs`
- `/api/openapi.json`
- `/api/swagger.json`
- `/api/trending`
- `/api/hot`

结论：

- 当前未发现公开的 OpenAPI / Swagger 描述文件
- 当前未发现独立的公开健康检查接口
- 当前未发现独立的公开榜单 API

---

## 7. 对 AgentDock 的接入建议

### 7.1 最小可用接入面

如果 AgentDock 只需要完成技能发现、预览和风险展示，最小接入集合如下：

1. `GET /api/skills/{board}/{page}`
2. `GET /api/search`
3. `GET /api/download/{owner}/{repo}/{slug}`
4. `GET /api/audit`

### 7.2 推荐职责划分

- 榜单列表：使用 `/api/skills/{board}/{page}`
- 搜索列表：使用 `/api/search`
- 技能详情预览：使用 `/api/download/...` 解析 `SKILL.md` 和 supporting files
- 风险提示：使用 `/api/audit`

### 7.3 实现注意点

- 不要依赖未公开的详情 API
- 不要假设 `download` 只返回单文件
- `search` 入参应先做本地校验
- `audit` 响应 schema 需要按动态对象处理

---

## 8. 参考来源

### 8.1 源码来源

- `vercel-labs/skills/src/find.ts`
- `vercel-labs/skills/src/blob.ts`

### 8.2 线上探测时间

- 2026-04-17

### 8.3 备注

本文档描述的是“当前已验证可见的接口行为”，不是 `skills.sh` 服务端完整接口清单。后续若站点前端或服务端发布变更，应重新探测并更新本文档。
