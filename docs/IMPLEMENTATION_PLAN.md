# AgentDock 开发计划

## 结论
- 基于 `src/pages/home.tsx` 继续开发
- 不拆独立业务页面
- 先收敛首页结构，再接 Tauri 真实数据
- Marketplace 作为首页内能力接入
- source 使用 `skills.sh`

## 阶段
1. **首页收敛**：拆分 `home.tsx`，抽离 rail、列表、详情、筛选和选择逻辑
2. **数据层收敛**：统一前端类型、展示模型和 mock 数据入口
3. **Tauri 接入**：补 commands 和 Rust 数据结构，用 `invoke()` 替换 mock
4. **编排能力**：补分组、拖拽、批量启停、批量移动、批量解绑
5. **Marketplace**：接入 `skills.sh`，支持导入到本地资源池并绑定 Agent
6. **持续增强**：优化首页布局、视图切换和导入流

## 优先修改文件
- `src/pages/home.tsx`
- `src/hooks/use-app-translation.ts`
- `src/lib/shortcut.ts`
- `src/lib/window.ts`
- `src-tauri/src/lib.rs`

## 建议新增目录
```txt
src/features/agents/
src/features/resource-registry/
src/features/agent-bindings/
src/features/marketplace/
```

## 边界
- 区分本地资源、Marketplace 资源、Agent 绑定资源
- 区分“移除绑定”和“删除资源”
- Marketplace 只负责导入，不负责主编排
- 敏感配置不明文展示
- 删除类操作必须确认

## Checklist

### 1. 首页结构收敛
- [x] 识别 `home.tsx` 中的三栏结构边界
- [x] 抽离左侧 Agent rail 组件
- [x] 抽离中间资源列表组件
- [x] 抽离右侧详情面板组件
- [x] 抽离资源卡片行组件
- [x] 抽离 Marketplace 详情渲染组件
- [x] 抽离本地资源详情渲染组件
- [x] 保持现有 UI 布局和交互不回退

### 2. 首页状态收敛
- [x] 抽离搜索状态
- [x] 抽离资源类型切换状态
- [x] 抽离当前 Agent 选中状态
- [x] 抽离当前资源选中状态
- [x] 抽离勾选状态
- [ ] 抽离批量操作状态
- [x] 抽离 Marketplace 安装状态流转逻辑
- [x] 让页面只负责组合，不直接承载大段状态逻辑

### 3. 类型与 mock 数据整理
- [x] 定义 Agent 前端类型
- [x] 定义 ResourceKind 类型
- [x] 定义 AgentDiscoveryItem 类型
- [x] 整理本地资源展示字段
- [x] 整理 Marketplace 资源展示字段
- [x] 统一安装状态枚举
- [x] 统一 mock 数据入口
- [x] 避免组件内散落硬编码数据

### 4. Tauri 数据接入
- [ ] 在 `src-tauri/src/lib.rs` 中补充资源管理 commands
- [ ] 定义 Agent Rust 数据结构
- [ ] 定义 Resource Rust 数据结构
- [ ] 定义 Binding Rust 数据结构
- [ ] 定义 Group Rust 数据结构
- [ ] 实现 Agent 配置读写
- [ ] 实现本地资源扫描
- [ ] 实现本地资源导入
- [ ] 实现本地资源删除
- [ ] 实现绑定关系持久化
- [ ] 实现分组持久化
- [ ] 前端用 `invoke()` 替换 mock 读取
- [ ] 前端用 `invoke()` 替换 mock 更新

### 5. 工作台能力补齐
- [ ] 支持创建分组
- [ ] 支持重命名分组
- [ ] 支持删除分组
- [ ] 支持分组折叠与展开
- [ ] 支持组内拖拽排序
- [ ] 支持跨组移动
- [ ] 支持移入未分组区
- [ ] 支持单项启用
- [ ] 支持单项停用
- [ ] 支持批量启用
- [ ] 支持批量停用
- [ ] 支持批量移动
- [ ] 支持批量解绑

### 6. Marketplace 接入
- [ ] 支持配置 source
- [ ] 接入 `skills.sh`
- [ ] 拉取 Marketplace 列表
- [ ] 支持 Marketplace 搜索
- [ ] 展示 Marketplace 详情
- [ ] 支持安装到本地资源池
- [ ] 支持安装后立即绑定到 Agent
- [ ] 支持安装状态反馈

### 7. 边界与体验
- [ ] 区分本地资源、Marketplace 资源、Agent 绑定资源
- [ ] 区分“移除绑定”和“删除资源”
- [ ] 为删除 Agent 增加确认
- [ ] 为删除资源增加确认
- [ ] 为删除分组增加确认
- [ ] 为删除 source 增加确认
- [ ] 敏感配置默认不明文展示
- [ ] 保持单页工作台结构清晰

### 8. 验收
- [ ] `home.tsx` 不再承载过多实现细节
- [ ] 首页已接入真实数据源
- [ ] 支持多个 Agent
- [ ] 支持本地资源池管理
- [ ] 支持绑定、解绑、启停
- [ ] 支持分组、拖拽、批量操作
- [ ] 支持从 Marketplace 导入资源
- [ ] 删除边界清晰
- [ ] 敏感配置不明文展示
