# Tasks

## 第一阶段：核心性能优化（预计 5-8 天）

- [x] Task 1.1: 启用并行解析
  - [x] 修改 `commands/mod.rs` 中的 `parse_log` 函数，调用 `parse_lines_parallel`
  - [x] 实现批量收集后并行解析，或流式并行处理方案
  - [x] 添加并行度配置选项
  - [x] 验证多核 CPU 利用率

- [x] Task 1.2: 修复零拷贝实现
  - [x] 修改 `MmapLine` 结构体，使用生命周期引用 `&'a [u8]`
  - [x] 移除 `MmapLineIterator::next()` 中的 `to_vec()` 调用
  - [x] 调整相关代码以适应生命周期参数
  - [x] 验证内存占用降低

- [x] Task 1.3: 优化数据库查询
  - [x] 设计二级索引键格式 `log_idx:{file_id}:{entry_id:016x}`
  - [x] 实现 `get_entries_fast` 函数，使用范围查询直接定位
  - [x] 在写入日志时同步创建索引
  - [x] 验证跳转性能

- [x] Task 1.4: 实现全文搜索索引
  - [x] 设计倒排索引结构 `word:{file_id}:{word_hash} → RoaringBitmap`
  - [x] 实现分词器 `tokenize` 函数
  - [x] 实现 `build_search_index` 函数
  - [x] 实现 `search_with_index` 函数
  - [x] 移除搜索只处理前 10000 条的限制
  - [x] 验证搜索性能

## 第二阶段：功能增强（预计 4-6 天）

- [x] Task 2.1: 添加时间戳索引
  - [x] 设计时间戳索引键格式 `time_idx:{file_id}:{timestamp} → RoaringBitmap`
  - [x] 实现 `filter_by_time_indexed` 函数
  - [x] 在写入日志时同步创建时间索引
  - [x] 验证时间过滤性能

- [x] Task 2.2: 实现真正的分页加载
  - [x] 修改 `logStore.ts`，实现 `loadedPages` Map 缓存
  - [x] 实现 `loadPage` 方法，按需加载页面
  - [x] 实现 `onScroll` 方法，动态加载可见区域
  - [x] 添加页面淘汰策略（LRU）
  - [x] 验证内存占用

- [x] Task 2.3: 后端过滤替代前端过滤
  - [x] 实现 `filter_logs` 命令，支持组合过滤
  - [x] 使用 RoaringBitmap 实现级别过滤
  - [x] 使用时间索引实现时间过滤
  - [x] 使用倒排索引实现搜索过滤
  - [x] 修改前端调用后端过滤接口
  - [x] 验证过滤性能

## 第三阶段：健壮性增强（预计 3-5 天）

- [x] Task 3.1: 添加文件大小检查
  - [x] 定义 `MAX_FILE_SIZE` 常量（10GB）
  - [x] 在 `open_file` 命令中添加文件大小检查
  - [x] 返回友好的错误提示
  - [x] 验证大文件拒绝逻辑

- [x] Task 3.2: 性能测试与验证
  - [x] 测试 1GB 文件解析时间（目标 < 12s）
  - [x] 测试内存占用（目标 < 600MB）
  - [x] 测试跳转到第 100 万条（目标 < 50ms）
  - [x] 测试全文搜索响应（目标 < 100ms）
  - [x] 测试级别过滤响应（目标 < 10ms）

# Task Dependencies
- [Task 1.2] 依赖 [Task 1.1] - 零拷贝修改可能影响解析流程
- [Task 1.3] 依赖 [Task 1.1] - 数据库优化需要先完成解析优化
- [Task 1.4] 依赖 [Task 1.3] - 搜索索引依赖数据库索引结构
- [Task 2.1] 依赖 [Task 1.3] - 时间索引依赖数据库索引结构
- [Task 2.2] 独立 - 前端分页加载可并行开发
- [Task 2.3] 依赖 [Task 1.4, 2.1] - 后端过滤依赖搜索和时间索引
- [Task 3.1] 独立 - 文件大小检查可并行开发
- [Task 3.2] 依赖 [所有前置任务] - 性能测试需要所有优化完成
