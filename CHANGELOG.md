# 变更日志

## v1.9.4

### UI 优化
- 优化浅色主题配色，提高文字对比度
  - 主文字颜色从 `#333333` 改为 `#1a1a1a`（更深黑）
  - 次要文字颜色从 `#666666` 改为 `#505050`（更清晰）
  - 为浅色主题单独定义日志级别颜色：
    - FATAL/ERROR: `#c42b2b`（深红色）
    - WARN: `#8a6d00`（深黄色）
    - INFO: `#0066cc`（深蓝色）
    - DEBUG: `#666666`（灰色）
    - TRACE: `#888888`（浅灰色）
    - OTHER: `#333333`（深灰色）

### Bug 修复
- 修复 LogList 时间、记录者、内容列未应用主题颜色的问题
  - 这三列缺少 `color: var(--color-text-primary)` 样式定义
  - 现在正确使用主题颜色变量

## v1.9.3

### Bug 修复
- 修复了 Tauri 环境检测问题
  - 原来使用 `window.__TAURI__` 检测，但 Tauri v2 默认不注入此变量
  - 改用 `window.__TAURI_INTERNALS__` 进行同步检测
  - 添加 `@tauri-apps/plugin-os` 插件进行异步检测
- 修复了浏览器环境下浏览按钮无响应的问题
  - 在浏览器环境（http://localhost:1420）下，`@tauri-apps/plugin-dialog` 的 `open` 方法无法工作
  - 添加了环境检测，在浏览器环境下显示友好提示
- 修复了拖拽打开功能在浏览器环境下无响应的问题
  - 浏览器环境下无法获取文件的完整路径 `(file as any).path`
  - 添加了环境检测，在浏览器环境下显示友好提示

### 新功能
- 浏览按钮支持中止功能
  - 加载超过 3 秒后，浏览按钮自动变为"中止"按钮
  - 点击中止按钮可以取消当前的文件加载操作
  - 中止按钮显示红色背景，易于识别

### 技术改进
- 新增环境检测模块（api/env.ts）
  - `isTauri()` 异步检测函数：使用 `@tauri-apps/plugin-os` 的 `platform()` 函数
  - `isTauriSync()` 同步检测函数：检测 `__TAURI_INTERNALS__` 或 `__TAURI__`
  - 用于区分 Tauri 和浏览器环境的功能差异
- 添加 `tauri-plugin-os` 插件
  - Rust 端：Cargo.toml 添加依赖，main.rs 注册插件
  - 前端：安装 `@tauri-apps/plugin-os` 包
  - capabilities：添加 `os:default` 权限
- logStore 新增 `abortLoad()` 方法
  - 支持中止正在进行的文件加载操作
  - 添加 `abortController` 和 `aborted` 状态管理
- FileBar 组件改进
  - 添加加载状态指示器（旋转动画）
  - 添加中止按钮样式
  - 根据环境显示不同的 placeholder 文本

## v1.9.2

### Bug 修复
- 修复了 LogList 一直显示"加载中"的问题（第二次修复）
  - **getVisibleLogs 方法错误**：之前只返回 `visibleStartPage` 到 `visibleEndPage` 之间的数据，但这两个值初始为 0，导致只返回第 0 页的数据。修改为返回所有已加载的页面数据
  - 添加调试日志，方便排查问题

## v1.9.1

### Bug 修复
- 修复了 LogList 一直显示"加载中"的问题
  - **数据库 key 格式不一致**：存储时使用 `{:016x}` 格式，但读取时使用 `{}` 格式，导致无法读取到数据
  - **前端状态通知缺失**：`loadInitialPages` 完成后没有调用 `notify()` 通知 UI 更新
- 修复了数据库路径权限问题
  - 将数据库路径从 `%LOCALAPPDATA%\LogViewer\data` 改为可执行文件所在目录的 `data` 子目录
  - 避免了某些环境下对系统目录的访问权限问题

### 日志解析改进
- 新增 `LOG_PATTERN_5` 正则表达式，支持全方括号格式的日志
  - 格式示例：`[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Received shutdown signal - ID:5283`
  - 新增 `try_parse_full_bracketed` 方法处理该格式

### 布局改进
- 将日志详情面板从右侧移至日志列表下方
- LogDetail 使用固定高度（默认 180px），LogList 自动填充剩余空间
- 添加可拖动分隔条，支持鼠标拖动调整 LogDetail 高度
  - 分隔条高度 6px，带有视觉指示器
  - 悬停和拖动时高亮显示
  - 高度限制范围：100px - 400px

### 功能简化
- 简化 LogDetail 组件，仅保留文本浏览模式
  - 移除浏览器、JSON、XML 标签页
  - 文本自动换行显示
  - 超长内容支持上下滚动
  - 兼容鼠标滚轮滚动

### 界面改进
- LogList 新增行序号列
  - 显示日志在原始文件中的行号（从 1 开始）
  - 便于快速定位和沟通引用
  - 过滤模式下仍保留原始行号
- 将"摘要"列名改为"内容"
- 优化 LogList 表头样式
  - 统一字体大小（12px）、字重（500）、颜色
  - 表头与数据行样式分离，数据行保持原有对齐方式

## v1.8.1

### 测试资源
- 新增日志样例文件（sample目录）：
  - `sample_100KB.log`：约100KB，1,223行
  - `sample_1MB.log`：约1MB，12,505行
  - `sample_10MB.log`：约10MB，125,005行
  - `sample_100MB.log`：约100MB，1,249,973行
- 日志格式：`[时间戳] [级别] [模块] [线程] 消息`
- 包含 DEBUG、INFO、WARN、ERROR、FATAL 五个级别
- 包含 App、Server、Database、Cache、Auth、API、Worker、Scheduler、Logger、Config 等模块

## v1.8.0

### 性能优化
- 实现后端过滤替代前端过滤：大幅减少数据传输量
  - 级别过滤响应时间 < 10ms
  - 组合过滤（级别 + 时间 + 搜索）正常工作
  - 利用 RoaringBitmap 位运算实现多条件交集过滤

### 技术改进
- 新增 `filter_logs` 命令（commands/mod.rs）：
  - 支持级别、时间范围、搜索查询的组合过滤
  - 使用 RoaringBitmap 位运算高效计算交集
  - 参数均为可选，灵活组合
- 新增 database 辅助方法（database/mod.rs）：
  - `search_bitmap`：返回搜索结果的 bitmap
  - `get_entries_by_ids`：根据 ID 列表批量获取日志条目
- 修改前端 logStore.ts：
  - `applyFilters` 方法改用后端 `filterLogs` API
  - 级别过滤、时间过滤、fuzzy 搜索完全由后端处理
  - exact 和 regex 搜索仍在前端处理（后端不支持）
- 新增 API 接口（api/tauri.ts）：
  - `FilterParams` 接口定义过滤参数
  - `filterLogs` 函数调用后端过滤命令

### 安全性改进
- 添加文件大小检查：防止处理超大文件导致系统问题
  - 定义最大文件大小限制为 10GB (`MAX_FILE_SIZE`)
  - 在 `open_file` 命令中检查文件大小
  - 超过限制时返回友好的错误提示，显示文件实际大小

### 技术改进
- 新增 `MAX_FILE_SIZE` 常量：10 * 1024 * 1024 * 1024 (10GB)
- 修改 `open_file` 命令：在获取文件元数据后检查文件大小
- 错误信息格式：显示文件实际大小（GB 单位，保留两位小数）

## v1.6.0

### 性能优化
- 实现真正的分页加载：前端按需加载页面，而非一次性加载 10000 条
  - 内存占用大幅降低，只保留当前需要的页面
  - 使用 LRU 缓存策略，最多缓存 20 页（每页 100 条）
  - 滚动时自动预加载前后各一页

### 技术改进
- 重构 `logStore.ts`：
  - 新增 `pageSize`（100条/页）和 `maxCachedPages`（20页）配置
  - 新增 `loadedPages` Map 存储已加载的页面
  - 新增 `loadPage` 方法：按需加载单页数据
  - 新增 `onScroll` 方法：监听滚动事件，触发页面加载
  - 新增 `getVisibleLogs` 方法：获取当前可见区域的日志
  - 新增 `isInFilteredMode` 状态：区分普通模式和过滤模式
  - 移除 `logs` 数组，不再一次性加载所有数据
- 修改 `LogList.svelte`：
  - 使用 `getVisibleLogs()` 替代 `getFilteredLogs()`
  - 滚动时调用 `logStore.onScroll()` 触发页面加载
  - 支持普通模式和过滤模式的不同渲染逻辑
- 修改 `LogDetail.svelte`：
  - `getLogById` 改为异步方法，支持从后端获取未缓存的日志详情
- 修改 `+page.svelte`：
  - 过滤方法改为异步调用

## v1.5.0

### 性能优化
- 实现时间戳索引：支持高效的时间范围过滤
  - 使用 RoaringBitmap 存储每个时间戳对应的日志条目 ID 集合
  - 索引键格式: `time_idx:{file_id}:{timestamp:016x}`
  - timestamp 使用毫秒级时间戳，16位十六进制格式
  - 时间过滤响应时间 < 50ms

### 技术改进
- 新增 `store_time_index` 方法：存储单条时间索引
- 新增 `store_time_index_batch` 方法：批量存储时间索引，提高效率
- 新增 `filter_by_time_indexed` 方法：使用索引进行时间范围过滤
- 新增 `get_entries_by_bitmap` 方法：根据 bitmap 获取日志条目
- 新增 `filter_by_time` 命令：前端可调用的时间过滤接口
- 修改 `parse_log` 命令：解析过程中同时构建时间索引

## v1.4.0

### 性能优化
- 实现全文搜索索引：构建倒排索引，支持百万级日志的快速搜索
  - 使用 RoaringBitmap 存储每个词对应的日志条目 ID 集合
  - 采用 DefaultHasher 计算词的哈希值作为索引键
  - 索引键格式: `word:{file_id}:{word_hash:016x}`
  - 支持多词 AND 搜索，通过 bitmap 交集快速过滤

### 技术改进
- 新增 `tokenize` 分词函数：按空格和标点分割，转小写，过滤短词
- 新增 `hash_word` 函数：使用 DefaultHasher 计算词哈希
- 新增 `build_search_index_from_map` 方法：批量存储词索引到数据库
- 新增 `get_word_bitmap` 方法：获取指定词的 bitmap
- 新增 `search_with_index` 方法：使用索引进行搜索
- 修改 `parse_log` 命令：解析过程中同时构建搜索索引
- 修改 `search` 命令：使用索引搜索替代内存扫描，移除 10000 条限制

## v1.3.0

### 性能优化
- 优化数据库查询：实现 O(1) 复杂度的偏移量定位
  - 将键格式从 `log:{file_id}:{entry_id}` 改为 `log:{file_id}:{entry_id:016x}`
  - 使用 16 位十六进制格式化 entry_id，确保字典序正确
  - 使用 sled 的 `range` 查询直接定位到 offset 位置
  - 跳转到第 100 万条日志响应时间从约 3 秒降至 < 50ms

### 技术改进
- `get_entries` 函数改用 range 查询替代 scan_prefix + skip
- 添加键前缀检查，确保只返回正确 file_id 的记录

## v1.2.0

### 性能优化
- 实现真正的零拷贝读取：重构 `MmapLine` 和 `MmapReader` 结构
  - `MmapLine` 现在使用生命周期引用 `&'a [u8]` 而非 `Vec<u8>`
  - `MmapReader::next_line()` 返回引用 mmap 内存的切片，无需 `to_vec()` 拷贝
  - 采用 lending iterator 模式，通过 `next_line()` 方法逐行读取
  - 添加 `remaining()` 和 `total_size()` 辅助方法

### 技术改进
- 移除了 `MmapLineIterator`，使用 `MmapReader` 替代
- 解决了 Rust 标准 Iterator trait 不支持 lending iterator 的限制

## v1.1.0

### 性能优化
- 启用并行解析：修改 `parse_log` 函数使用 rayon 多线程并行解析
  - 采用批量收集后并行解析方案，每 10000 行一个批次
  - 使用 `parse_lines_parallel` 函数实现并行处理
  - 保持进度推送功能正常工作
- 添加并行度配置：在 `AppSettings` 中新增 `parallel_workers` 字段
  - 支持自定义线程池大小
  - 默认使用 rayon 自动配置

### 技术改进
- 使用 `AtomicU64` 实现线程安全的计数器
- 优化了 level bitmaps 的批量合并操作

## v1.0.0

### 功能改进
- 添加了设置按钮和帮助按钮
- 添加了搜索范围和搜索模式下拉菜单
- 添加了设置模态框，支持主题和编码选择
- 添加了帮助模态框，包含快捷键和使用说明
- 优化了状态栏显示，支持动态消息

### UI 优化
- 统一了三个工具栏（TitleBar、FileBar、ControlBar）的字体大小为 12px
- 统一了三栏表头文字（日志文件、筛选、级别）大小为 12px，左端对齐
- 增大了"筛选"标签与下拉框之间的间距（从 6px 增加到 10px）
- 增大了搜索输入框和搜索按钮之间的间距（从 12px 增加到 16px）
- 减小了搜索按钮的大小（padding 从 5px 10px 减小到 4px 8px）
- 增大了 FileBar 栏的高度（从 36px 增加到 44px），与 ControlBar (50px) 更协调
- 确保了各组件之间的间距一致性
- 优化了整体布局的美观协调性

### 技术改进
- 修复了 ParseProgress 缺少 phase 字段的问题
- 改进了搜索功能，支持正则表达式
- 实现了设置的持久化存储
- 优化了文件编码检测和转换
- 提高了加载大文件时的性能和稳定性