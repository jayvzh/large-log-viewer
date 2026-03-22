# 变更日志

## v0.4.0

### Added

- **Extra 字段索引系统**：
  - 新增后端 Extra 字段索引，支持高效过滤
  - 索引键格式：`extra_idx:{file_id}:{field_name}:{value_hash}` → RoaringBitmap
  - 支持等于、包含操作符使用索引快速查询
  - 正则、大于、小于操作符回退到全量扫描

- **模板优先级支持**：
  - LogTemplate 新增 `priority` 字段
  - 模板按优先级从高到低匹配
  - 模板编辑器支持设置优先级

- **模板解析器缓存**：
  - AppState 缓存 LogTemplateParser 实例
  - 避免重复编译正则表达式
  - 模板变更时自动清除缓存

- **FilterPanel 增强**：
  - 添加折叠/展开功能
  - 折叠状态持久化到 localStorage
  - 添加"清除所有筛选"按钮

### Fixed

- **修复 FilterPanel 不显示问题**：
  - 问题：FilterPanel 依赖未实现的 `field_mapping` 字段，导致大多数情况下不显示
  - 修复：改用模板的 `extra_fields` 字段

- **修复 LogList 动态列不显示问题**：
  - 问题：LogList 的动态列依赖 `extraFields`，但该值始终为空
  - 修复：修复 templateStore 的字段提取逻辑

- **修复选择模板时未更新 templateStore**：
  - 问题：选择模板后，templateStore 的 currentTemplate 未更新
  - 修复：在 FileBar.selectTemplate 中调用 `templateStore.setCurrentTemplate()`

### Changed

- **FilterPanel 位置调整**：
  - 将 FilterPanel 移至 CategoryTabs 上方
  - 文件：[src/routes/+page.svelte](file:///e:/Code/github/large-log-viewer/src/routes/+page.svelte)

- **FilterPanel 显示逻辑**：
  - 移除条件渲染，FilterPanel 始终显示
  - 当无 extra 字段时，仅显示核心字段筛选选项

- **Extra 字段过滤优化**：
  - Extra 字段过滤从客户端移至后端
  - 利用索引实现高效过滤

## v0.3.3

### Added

- **模板系统分析文档**：
  - 新增 [templatesys.md](file:///e:/Code/github/large-log-viewer/templatesys.md) 文档
  - 详细说明模板系统的架构、数据流向、语法规范
  - 分析数据处理流程：原始日志 → LogEvent → LogEntry → 数据库存储
  - 说明 Extra 字段提取机制和存储方式
  - 指出当前方案的缺点（架构、性能、功能、用户体验四个层面）
  - 提出短期、中期、长期改进方向建议

## v0.3.2

### Fixed

- **修复 Windows 换行符导致日志解析失败的问题**：
  - 问题：Windows 风格换行符（`\r\n`）的日志文件无法正确解析时间戳，所有日志显示为当前系统时间
  - 原因：`MmapReader::next_line` 只处理 `\n` 作为换行符，`\r` 字符留在行尾导致正则表达式无法匹配
  - 修复：在 `LogFileReader::decode_line()` 中去除行尾的 `\r` 字符
  - 文件：[src-tauri/src/reader/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/reader/mod.rs)

### Changed

- **重构内置模板系统**：
  - 删除原有的5个通用模板（Standard Format、Bracketed Level、Level First、Date First、Full Bracketed）
  - 新增3个针对特定日志格式的内置模板：
    - **Linux Syslog**：Linux系统全局日志格式（rsyslog），支持主机名字段
    - **Linux Auth Log**：Linux系统认证日志格式（sshd等），支持主机名字段
    - **Nginx Access Log**：Nginx访问日志格式（combined格式），支持IP、请求、状态码等字段
  - 文件：[src-tauri/src/parser/template.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/parser/template.rs)

- **优化模板存储路径**：
  - 开发模式（`pnpm tauri dev`）下模板存储路径改为 `src-tauri/data/templates.json`
  - 生产模式下模板存储路径仍为 `{exe_dir}/data/templates.json`
  - 通过 `CARGO_MANIFEST_DIR` 环境变量自动检测开发模式
  - 文件：[src-tauri/src/template_store.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/template_store.rs)

## v0.3.0

### Added

- **多实例支持**：
  - 移除 `tauri-plugin-single-instance` 插件，允许多个程序实例同时运行
  - 数据库路径使用进程ID隔离（`logs_{pid}.db`），各实例数据互不干扰
  - 文件：[src-tauri/Cargo.toml](file:///e:/Code/github/large-log-viewer/src-tauri/Cargo.toml)、[main.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/main.rs)、[database/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/database/mod.rs)
- **模板存储优化**：
  - 新增 `TemplateStore` 模块，将用户模板存储到独立 JSON 文件
  - 模板存储路径：`{exe_dir}/data/templates.json`
  - 多实例共享模板数据，便于备份迁移
  - 文件：[src-tauri/src/template_store.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/template_store.rs)
- **日志导出功能**：
  - 新增 `export_logs` Tauri 命令
  - 支持导出过滤后的日志和搜索结果
  - 支持纯文本和 JSON 格式导出
  - 前端 ControlBar 添加导出按钮和导出对话框
  - 导出时自动应用当前过滤条件（级别、搜索、时间筛选、字段筛选）
  - 文件：[src-tauri/src/commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)、[src/lib/api/tauri.ts](file:///e:/Code/github/large-log-viewer/src/lib/api/tauri.ts)、[src/lib/components/ControlBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/ControlBar.svelte)

- **字段筛选功能集成**：
  - FilterPanel 组件集成到主页面（CategoryTabs 下方）
  - 字段筛选条件自动应用到过滤流程
  - 支持核心字段（时间、级别、来源、消息、原始内容）和模板额外字段
  - 支持 AND/OR 组合模式
  - 文件：[src/routes/+page.svelte](file:///e:/Code/github/large-log-viewer/src/routes/+page.svelte)、[src/lib/stores/logStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/logStore.ts)

### Changed

- **解析状态筛选改进**：
  - 将"显示未解析"复选框改为下拉选择
  - 支持三种模式：全部日志、仅已解析、仅未解析
  - **改为后端过滤**：解析状态筛选现在在后端进行，触发 CategoryTabs 统计更新
  - 文件：[src/lib/components/ControlBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/ControlBar.svelte)、[src/lib/stores/logStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/logStore.ts)

- **统一下拉菜单样式**：
  - 所有下拉菜单统一使用原生 `<select>` 元素
  - 移除自定义下拉菜单组件，减少代码复杂度
  - 文件：[src/lib/components/ControlBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/ControlBar.svelte)

- **编码/模板选择器即时重载**：
  - 选择编码后，如果已打开文件则立即重新加载
  - 选择模板后，如果已打开文件则立即重新解析
  - 文件：[src/lib/components/FileBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FileBar.svelte)

- **CategoryTabs 统计优化**：
  - 级别统计现在基于过滤结果而非全部日志
  - 搜索后各级别数量会相应更新
  - 文件：[src/lib/stores/logStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/logStore.ts)、[src/lib/components/CategoryTabs.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/CategoryTabs.svelte)

- **导出按钮重命名**：
  - "导出"改为"导出筛选"，更准确描述功能

- **后端序列化修复**：
  - 修复 `LogEntryView` 字段序列化为 camelCase
  - `is_parsed` 正确序列化为 `isParsed`
  - 文件：[src-tauri/src/models/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/models/mod.rs)

- **移除调试日志**：
  - 清理 `commands/mod.rs` 和 `database/mod.rs` 中的 `eprintln!` 调试输出
  - 代码更整洁，减少不必要的控制台输出
- **模板检测优化**：
  - `detect_template` 命令只读取文件前 100 行进行检测
  - 提升大文件模板检测速度
- **时间过滤确认**：
  - 确认时间过滤正确使用后端时间索引
  - 前端正确传递时间参数

## v0.2.3

### Added

- 实现日志解析模板系统
  - **后端核心功能**：
    - 新增核心数据结构：`LogTemplate`、`LogEvent`、`TemplateTestResult`、`DetectResult`
    - 创建 `LogTemplateParser` 模板解析器，支持自定义正则表达式模板
    - 内置 5 种常用日志格式模板（Standard Format、Bracketed Level、Level First、Date First、Full Bracketed）
    - 实现模板存储功能（`store_template`、`get_template`、`get_all_templates`、`delete_template`）
    - 添加 Tauri 命令：`get_templates`、`create_template`、`update_template`、`delete_template`、`test_template`、`detect_template`、`export_templates`、`import_templates`
    - 支持模板测试和自动检测最佳匹配模板功能
    - 支持命名捕获组字段别名映射（time→timestamp, msg→message, lvl→level 等）
    - `parse_log` 命令支持可选的 `template_name` 参数，使用指定模板解析日志
  - **前端功能**：
    - 新增模板类型定义文件 [src/lib/types/template.ts](file:///e:/Code/github/large-log-viewer/src/lib/types/template.ts)
    - 新增模板状态管理 [src/lib/stores/templateStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/templateStore.ts)
    - 新增模板管理弹窗组件 [TemplateManagerModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateManagerModal.svelte)
    - 新增模板编辑弹窗组件 [TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)
- 新增 [TODO.md](file:///e:/Code/github/large-log-viewer/TODO.md) 文档，记录已完成功能和待改进事项
  - 重构 [FileBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FileBar.svelte) 添加模板选择下拉菜单
  - 支持实时预览模板解析结果
  - 支持模板导入/导出功能
  - logStore 支持选中模板状态，解析时传递模板参数
  - **测试**：
    - 新增 11 个模板相关单元测试（共 24 个测试全部通过）
- **后端支持模板字段元数据**：
  - `LogTemplate` 结构新增字段：`extra_fields`、`has_level`、`has_timestamp`、`has_source`
  - `LogEntry` 结构新增字段：`extra`（额外字段）、`is_parsed`（是否成功解析）
  - `LogEntryView` 结构新增字段：`extra`、`is_parsed`
  - `LogTemplateParser` 新增 `extract_fields_from_pattern` 方法，从正则表达式提取字段元数据
  - 内置模板设置正确的字段元数据标志
  - `parse_log` 命令支持存储 extra 字段和设置 is\_parsed 标志
- **前端动态字段支持**：
  - 新增 [FilterPanel.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FilterPanel.svelte) 字段驱动筛选组件
    - 支持多条件筛选（等于、包含、正则、大于、小于）
    - 支持 AND/OR 组合模式
    - 动态字段选择（核心字段 + 模板额外字段）
  - templateStore 新增状态管理：
    - `currentFields`：当前模板的额外字段列表
    - `hasLevel`：当前模板是否包含级别字段
    - `filterConditions`：筛选条件列表
    - `filterCombineMode`：筛选组合模式
    - `showUnparsed`：是否显示未解析日志
  - logStore 新增功能：
    - `applyFieldFilters`：字段驱动筛选逻辑
    - `setShowUnparsed`/`getShowUnparsed`：未解析日志显示控制
    - `LogEntry` 接口新增 `extra` 和 `isParsed` 字段
- **模板编辑器语法指引功能**：
  - 新增右侧语法指引面板，可折叠/展开
  - 添加命名捕获组语法说明：`(?P<字段名>正则表达式)`
  - 添加内置字段名及别名对照表（timestamp/time/ts、level/lvl/severity 等）
  - 添加常用正则语法速查表（\d、\w、\s、[]、{} 等）
  - 添加 4 个示例模板（标准日志格式、方括号级别格式、JSON 日志格式、带线程信息格式）
  - 示例模板支持一键应用，自动填充正则表达式和测试日志
  - 文件：[TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)

### Changed

- **完善文件编码支持功能**：
  - 后端 `parse_log` 和 `detect_template` 命令新增 `encoding` 参数
  - 使用 `LogFileReader::decode_line()` 替代 `String::from_utf8_lossy()` 进行正确的编码解码
  - 支持 UTF-8、UTF-16 LE/BE、ANSI 编码
  - 文件：[src-tauri/src/commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)
- **重构 FileBar 组件布局**：
  - 将编码选择和模板选择移至 FileBar 右侧
  - 添加"编码:"和"模板:"标签前缀
  - 编码选择支持：自动检测、UTF-8、UTF-16 LE、UTF-16 BE、ANSI
  - 优化按钮行为：浏览按钮加载时保持不变（仅禁用），打开按钮加载时变为中止按钮
  - 文件：[src/lib/components/FileBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FileBar.svelte)
- **从设置页面移除编码设置**：
  - 编码设置已移至 FileBar，设置页面仅保留主题和文件关联
  - 文件：[src/lib/components/SettingsModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/SettingsModal.svelte)
- **修复 UTF-8 BOM 字符导致日志解析失败的问题**：
  - 问题：带有 UTF-8 BOM (`EF BB BF`) 的日志文件，第一行无法被正则表达式匹配，被识别为 `OTHER / Unknown`
  - 修复：在 `MmapReader::new()` 中检测并跳过 UTF-8 BOM 字符
  - 文件：[src-tauri/src/reader/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/reader/mod.rs)
- 将日志字段"记录者" (logger) 重命名为"来源" (source)
  - 前端组件：LogList.svelte、LogDetail.svelte、ControlBar.svelte
  - 前端状态管理：logStore.ts
  - 后端模型：models/mod.rs (LogEntry、LogEntryView)
  - 后端解析器：parser/mod.rs (正则表达式捕获组名称)
  - 后端命令和数据库：commands/mod.rs、database/mod.rs
  - 文档：README.md
- **重构 LogList 组件支持动态列**：
  - 定义 `ColumnConfig` 接口管理列配置
  - 根据 `hasLevel` 条件显示/隐藏级别列
  - 根据 `extraFields` 动态添加额外字段列
  - 表格支持横向滚动（overflow-x: auto）
  - 未解析日志添加特殊样式（灰色斜体）
- **重构 CategoryTabs 组件**：
  - 新增 `hasLevel` prop 控制级别过滤面板显示
  - 当模板无级别字段时显示简化版（仅显示总数）
- **重构 ControlBar 组件**：
  - 搜索范围改为动态字段选择
  - 新增"显示未解析"开关
  - 支持模板额外字段作为搜索范围选项
  - 修复搜索范围选项：添加"级别"选项，将"消息"改为"内容"
  - 移除"原始内容"选项（"全部范围"已包含原始内容，功能重复）
- **修复搜索功能字段映射**：
  - logStore 的 `applyClientSideSearch` 现在支持所有核心字段（级别、来源、内容）
  - 支持动态额外字段（extra）的搜索
  - "全部范围"搜索包含原始内容（raw）及所有解析字段
- **统一字段命名**：
  - 将 `summary` 字段重命名为 `message`，统一前后端命名
  - 后端：LogEntry.summary → LogEntry.message，summary\_str() → message\_str()
  - 前端：LogEntry.summary → LogEntry.message
  - LogList 列配置：content key → message key
  - 正则捕获组：summary → message
  - 现在所有层级统一使用 `message` 表示日志消息内容
  - 修复 LogList 的 getColumnStyle 函数中遗漏的 'content' → 'message'

