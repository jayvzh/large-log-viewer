# 变更日志

## v0.6.2 (2026-03-24)

### Fixed

- **修复时间戳解析问题**：
  - 问题：使用自定义模板解析日志时，时间戳显示为 3/22 00:03:xx 左右的错误时间
  - 原因：时间戳解析格式使用了错误的 `%Y-%m-%d %H:%M:%S%.3f` 格式，正确格式应为 `%Y-%m-%d %H:%M:%S.%f`
  - 修复：修正时间戳解析格式，使用 `%Y-%m-%d %H:%M:%S.%f`、`%Y-%m-%dT%H:%M:%S.%f` 和 `%Y-%m-%dT%H:%M:%S.%f%:z` 格式
  - 文件：[commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)

- **修复模板优先级问题**：
  - 问题：用户选择自定义模板后，模板会被按优先级重新排序，导致用户选择的模板不是第一个被使用的模板
  - 原因：`LogTemplateParser::new` 函数会按照 priority 字段重新排序模板
  - 修复：移除模板排序逻辑，保持用户选择的模板顺序
  - 文件：[template.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/parser/template.rs)

- **修复高亮引擎问题**：
  - 问题：highlight_engine.rs 中存在调试输出、逻辑错误和代码重复问题
  - 原因：生产代码中包含 println! 语句，source 字段查找逻辑错误，重复构建 raw_content，方法过长
  - 修复：
    - 移除所有 println! 调试语句
    - 修复 source 字段查找逻辑，改为在 raw_content 中查找
    - 提取 raw_content 构建逻辑，避免重复构建
    - 为 extra 字段的 span 添加注释说明
    - 重构 process_field_rules 方法，拆分为多个小方法：process_level_field、process_source_field、process_message_field、process_extra_field
    - 提高代码可维护性和可读性
  - 文件：[highlight_engine.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/highlight_engine.rs)

### Added

- **实现动态隐藏来源列功能**：
  - 当日志模板的 `has_source` 为 false 时，来源列会自动隐藏
  - 为 templateStore 添加 `hasSource` 属性支持
  - 修改 LogList 组件，根据 `hasSource` 条件显示/隐藏来源列
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)、[templateStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/templateStore.ts)

- **添加表头拖动调整宽度功能**：
  - 实现表头列宽拖动调整功能，用户可通过拖动表头边缘调整列宽
  - 为表头添加拖动手柄和相关事件处理
  - 最小列宽限制为 30px
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **为 accesslog 类型日志调整列宽**：
  - 为 accesslog 模板的不同字段设置合适的列宽
  - 加宽 request 列（250px）和 useragent 列（300px）
  - 收窄 status 列（60px）和 size 列（80px）
  - 为 referer 列设置合适宽度（200px）
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **添加表头竖线分隔**：
  - 为表头列之间添加竖线分隔，使列边界更明显，方便用户拖动调整宽度
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **添加 accesslog 模板**：
  - 新增 accesslog 内置模板，支持解析标准的 Nginx 访问日志格式
  - 模板包含 ip、request、status、size、referer、useragent 字段
  - 设置 `has_level: false` 和 `has_source: false`，自动隐藏级别和来源列
  - 文件：[templates.json](file:///e:/Code/github/large-log-viewer/src-tauri/data/templates.json)

- **实现列宽保存和加载功能**：
  - 当用户调整列宽时，自动保存列宽配置到 localStorage
  - 列宽配置与当前日志模板关联，不同模板有独立的列宽设置
  - 切换模板时自动加载对应的列宽配置
  - 删除模板时自动删除对应的列宽配置，避免产生垃圾数据
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)、[templateStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/templateStore.ts)

- **实现 loglist 多行选择和复制功能**：
  - 支持 Ctrl+点击：切换行选中状态
  - 支持 Shift+点击：选择连续范围的行
  - 支持 Ctrl+C：复制选中的日志行
  - 复制内容包含所有可见列的信息，按列顺序排列
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

### Changed

- **统一模板和高亮存储路径**：
  - 将模板文件（templates.json）和高亮模板文件（highlights.json）的存储路径改为与数据库文件一致的位置
  - 现在所有数据文件都存储在 `{exe_dir}/data/` 目录中，方便备份和迁移
  - 文件：[template_store.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/template_store.rs)、[highlight_store.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/highlight_store.rs)

### Fixed

- **修复表头和列对齐问题**：
  - 统一表头和列的渲染逻辑，确保使用相同的列配置和样式
  - 将所有列类（如col-line, col-time等）替换为统一的.col类
  - 添加基于位置的样式，确保第一列（行号）、第二列（时间）和第三列（级别）具有正确的样式
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **修复表头竖线和文字间距问题**：
  - 为表头添加 8px 的左右 padding，增加表头竖线和文字之间的间距
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **修复内容列宽不跟随表头变化问题**：
  - 移除固定宽度的 CSS 样式，使用内联样式来设置列宽
  - 确保内容列宽与表头保持一致，表头拖动调整宽度时内容列也会同步变化
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **修复 extra 字段列不能拖动问题**：
  - 添加 extraFieldWidths 存储用户调整的宽度
  - 修改 handleResizeMove 函数，支持 extra 字段列的宽度调整
  - 修改 columns 的派生计算逻辑，确保 extra 字段列也能正常拖动调整宽度
  - 文件：[LogList.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LogList.svelte)

- **修复 LevelBar 显示问题**：
  - 当打开 accesslog 等无级别日志时，LevelBar 显示的日志条数现在使用筛选后的统计数据
  - 在 LevelBar 右侧添加显示搜索/筛选结果的条目数
  - 文件：[LevelBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LevelBar.svelte)

- **修复模板选择不生效问题**：
  - 问题：选择模板后，只有选中的模板会被使用，当模板不匹配时没有回退到其他模板
  - 修复：修改 `get_or_create_parser` 函数，当用户选择模板时，优先使用该模板，同时在不匹配时尝试其他模板
  - 确保即使选择了特定模板，也能解析不匹配该模板的日志行
  - 文件：[commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)

## v0.6.1 (2026-03-24)

### Fixed

- **修复测试用例中的类型错误**：
  - 问题：测试用例中的LogTemplate初始化缺少default_highlight字段，且类型不匹配
  - 修复：为测试用例添加default_highlight字段，并使用正确的Option<String>类型
  - 文件：[lib.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/lib.rs)

- **修复测试用例期望不匹配问题**：
  - 问题：测试用例期望5个内置模板，但实际只有3个
  - 修复：更新测试用例的期望数量，并修改测试数据以匹配实际的内置模板格式
  - 文件：[lib.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/lib.rs)

- **修复级别栏点击异常问题**：
  - 问题：点击级别标签时，级别栏的统计数据会被筛选结果覆盖，导致所有级别数量显示为当前筛选级别的数量
  - 修复：修改LevelBar组件，使用原始统计数据（getStats()）而非筛选后的数据（getFilteredStats()），确保级别栏显示的是原始日志的级别分布
  - 文件：[LevelBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/LevelBar.svelte)

### Added

- **在前端设置默认高亮模板**：
  - 问题：程序启动时高亮模板显示"无"
  - 修复：在highlightStore.loadProfiles()中，当currentProfile为null时，自动设置general_default为默认高亮模板
  - 文件：[highlightStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/highlightStore.ts)

- **为关键词规则添加样式支持**：
  - 问题：高亮模板的关键词规则没有样式设置选项
  - 修复：在HighlightEditorModal中为关键词规则添加样式输入框
  - 文件：[HighlightEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/HighlightEditorModal.svelte)

- **支持关键词规则多个关键词**：
  - 问题：关键词规则只能输入一个关键词
  - 修复：支持逗号分隔的多个关键词
  - 文件：[HighlightEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/HighlightEditorModal.svelte)

### Changed

- **移除调试日志**：
  - 移除前端和后端的debug日志输出，使代码更整洁
  - 文件：[highlightStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/highlightStore.ts)、[FileBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FileBar.svelte)、[commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)、[highlight_store.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/highlight_store.rs)

- **优化高亮模板下拉菜单**：
  - 移除"无"选项，将general_default移动到顶部并改名为"通用默认"
  - 为其他内置高亮模板添加中文名称（web_default → "Web默认"，security_default → "安全默认"）
  - 文件：[FileBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FileBar.svelte)

- **修改默认高亮模板回落规则**：
  - 当日志模板没有设定高亮规则时，默认使用general_default
  - 新建日志模板页面，高亮模板下拉框默认选中general_default
  - 文件：[TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)、[logStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/logStore.ts)

## v0.6.0 (2026-03-23)

### Fixed

- **修复内置高亮模板不显示的问题**：
  - 问题：内置高亮模板（general_default、web_default、security_default）无法正常显示
  - 修复：添加调试日志，确认后端正确返回内置模板
  - 文件：[commands/mod.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/commands/mod.rs)、[highlightStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/highlightStore.ts)

- **修复新建高亮模板点击无反应的问题**：
  - 问题：HighlightEditorModal 的 open 方法未导出，无法从外部调用
  - 修复：为 open 和 close 方法添加 export 关键字
  - 文件：[HighlightEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/HighlightEditorModal.svelte)

- **修复 TemplateEditorModal 类型错误**：
  - 问题：selectedHighlight 类型为 `string | null`，但 LogTemplate.default_highlight 类型为 `string | undefined`
  - 修复：将 selectedHighlight 类型改为 `string | undefined`
  - 文件：[TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)

- **修复 TemplateManagerModal 参数错误**：
  - 问题：`highlightStore.loadProfiles(true)` 调用错误，loadProfiles 方法不接受参数
  - 修复：移除多余的参数
  - 文件：[TemplateManagerModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateManagerModal.svelte)

### Added

- **为内置日志模板添加默认高亮模板字段**：
  - Linux Syslog → general_default
  - Linux Auth Log → security_default
  - Nginx Access Log → web_default
  - 文件：[template.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/parser/template.rs)

- **在模板编辑页面添加高亮模板选择下拉框**：
  - 允许用户为日志模板设置默认高亮模板
  - 下拉框显示内置和用户自定义的高亮模板
  - 文件：[TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)

## v0.5.3

### Fixed

- **修复拖拽文件打开功能**：
  - 问题：拖拽文件到界面没有反应，无法打开文件
  - 原因：原代码使用 HTML5 原生拖拽事件（ondragover/ondrop），但 Tauri webview 中这些事件无法正确获取文件路径
  - 修复：使用 Tauri 专门的拖拽事件系统 `getCurrentWindow().onDragDropEvent()`
  - Tauri 环境：使用 `onDragDropEvent` 监听拖拽事件，通过 `event.payload.paths` 获取文件路径
  - 文件：[+page.svelte](file:///e:/Code/github/large-log-viewer/src/routes/+page.svelte)

### Changed

- **清理网页端冗余代码**：
  - 移除拖拽功能中的网页端兼容代码（isTauriSync 检查和提示信息）
  - 清理 env.ts 中未使用的函数（isTauri、getEnvironment、platform 导入）
  - 该项目为 Tauri 桌面应用，网页端仅用于开发预览，无需兼容
  - 文件：[+page.svelte](file:///e:/Code/github/large-log-viewer/src/routes/+page.svelte)、[env.ts](file:///e:/Code/github/large-log-viewer/src/lib/api/env.ts)

## v0.5.2

### Fixed

- **修复模板导出功能**：
  - 问题：点击导出按钮没有反应，无法弹出保存对话框
  - 原因：原代码使用传统 Web 方式（创建 `<a>` 元素下载），在 Tauri 应用中无法正常工作
  - 修复：使用 Tauri 的 `@tauri-apps/plugin-dialog` 的 `save()` API 弹出保存对话框，使用 `@tauri-apps/plugin-fs` 的 `writeTextFile()` 写入文件
  - 新增依赖：前端 `@tauri-apps/plugin-fs`，后端 `tauri-plugin-fs`
  - 配置：capabilities 添加 `fs:read-all` 和 `fs:write-all` 权限
  - 文件：[TemplateManagerModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateManagerModal.svelte)、[Cargo.toml](file:///e:/Code/github/large-log-viewer/src-tauri/Cargo.toml)、[main.rs](file:///e:/Code/github/large-log-viewer/src-tauri/src/main.rs)、[default.json](file:///e:/Code/github/large-log-viewer/src-tauri/capabilities/default.json)

## v0.5.1

### Changed

- **文档结构重组**：
  - 创建 [development.md](file:///e:/Code/github/large-log-viewer/development.md) 开发文档，整合技术架构、模板系统详解、开发进度和待改进事项
  - 精简优化 [README.md](file:///e:/Code/github/large-log-viewer/README.md)，保留用户使用相关内容，移除开发细节
  - 删除 TODO.md 和 templatesys.md，内容已整合到 development.md
  - README.md 添加指向 development.md 的链接，方便开发者查阅详细文档
- **README 模板系统章节精简**：
  - 将模板系统章节从详细说明改为简略介绍
  - 详细内容（语法、字段映射、自定义示例、自动检测、创建步骤、AI辅助）移至 development.md
  - README 仅保留核心功能概述和快速创建模板指南

### Added

- **模板编辑器优先级提示**：
  - 在优先级字段标签后添加「数值越大优先级越高」提示文字
  - 文件：[TemplateEditorModal.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/TemplateEditorModal.svelte)

## v0.5.0

### Changed

- **重构搜索与筛选界面**：
  - 移除 ControlBar 的「搜索范围」选择器，简化为全局搜索（搜索全部字段）
  - 搜索按钮改为图标按钮，界面更简洁
  - FilterPanel 重命名为「高级筛选」，定位更清晰
  - 高级筛选面板默认隐藏，通过 ControlBar 的筛选按钮展开/折叠
  - 筛选按钮显示当前筛选条件数量的徽章
  - 减少搜索和筛选功能的重叠，降低用户学习成本
  - 文件：[ControlBar.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/ControlBar.svelte)、[FilterPanel.svelte](file:///e:/Code/github/large-log-viewer/src/lib/components/FilterPanel.svelte)、[logStore.ts](file:///e:/Code/github/large-log-viewer/src/lib/stores/logStore.ts)、[+page.svelte](file:///e:/Code/github/large-log-viewer/src/routes/+page.svelte)

## v0.4.2

### Changed

- **更新 README 文档**：
  - 重构标题和简介，添加核心亮点列表（性能、解析、搜索、过滤）
  - 重构「模板系统」章节，详细说明模板语法、核心字段名、内置模板
  - 新增「模板系统详解」章节，包含自定义模板示例、自动检测流程、AI 辅助生成说明
  - 更新「支持的日志格式」章节，区分内置模板支持的格式和自定义模板支持的格式
  - 更新「项目结构」章节，添加模板相关文件说明
  - 文件：[README.md](file:///e:/Code/github/large-log-viewer/README.md)

## v0.4.1

### Fixed

- **修复模板操作后界面闪烁问题**：
  - 问题：新增、编辑、删除模板后界面会短暂消失再重新出现，类似重启效果
  - 原因1：FileBar.svelte 中无依赖的 `$effect` 导致重复调用 `loadTemplates()`
  - 原因2：开发模式下模板存储在 `src-tauri/data/` 目录，Tauri dev 监视该目录变化导致重新构建
  - 修复：移除 FileBar 中无依赖的 `$effect`；为 `loadTemplates()` 添加静默模式参数；将模板存储路径改为用户数据目录（`~/.local/share/LogViewer/` 或 `%APPDATA%/LogViewer/`）

### Changed

- **移除调试日志**：移除 `commands/mod.rs` 中的 DEBUG 日志输出

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

- **模板编辑器简化**：
  - 移除右侧冗长的语法指引面板
  - 添加 AI 辅助生成提示词，用户可复制给 ChatGPT/Claude 生成模板
  - 底部简化语法说明，一目了然
  - 测试结果更直观地显示提取的字段

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

- **修复 Nginx Access Log 时间戳解析**：
  - 问题：Nginx 时间格式 `03/Feb/2026:15:36:05 +0800` 无法被正确解析
  - 修复：添加 `%d/%b/%Y:%H:%M:%S %z` 格式支持

- **修复自动模板检测未生效**：
  - 问题：打开文件时未自动检测最佳匹配模板
  - 根本原因：缓存判断逻辑 `cached_name.as_deref() == template_name` 当两者都是 `None` 时返回 `true`，导致直接返回空的缓存解析器，跳过了自动检测
  - 修复：修改缓存判断逻辑，仅在两者都是 `Some` 且相等时才使用缓存
  - `parse_log` 命令现在会自动检测并返回检测到的模板名称
  - 前端收到检测结果后自动设置 templateStore

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

