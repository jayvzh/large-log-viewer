# Tasks

## 后端任务

- [x] Task 1: 定义核心数据结构
  - [x] SubTask 1.1: 在 `models/mod.rs` 中定义 `LogTemplate` 结构
  - [x] SubTask 1.2: 在 `models/mod.rs` 中定义 `LogEvent` 结构
  - [x] SubTask 1.3: 定义 `TemplateType` 枚举（Builtin/User）
  - [x] SubTask 1.4: 添加序列化/反序列化支持

- [x] Task 2: 重构 Parser 模块
  - [x] SubTask 2.1: 创建 `parser/template.rs` 文件
  - [x] SubTask 2.2: 实现 `LogTemplateParser` 结构体
  - [x] SubTask 2.3: 实现 `parse_with_template` 方法
  - [x] SubTask 2.4: 实现命名捕获组到 LogEvent 的映射
  - [x] SubTask 2.5: 实现字段别名映射（time->timestamp, msg->message）
  - [x] SubTask 2.6: 将现有5种模式转换为内置模板
  - [x] SubTask 2.7: 实现 `LogEvent` 到 `LogEntry` 的转换

- [x] Task 3: 实现模板存储
  - [x] SubTask 3.1: 在 `database/mod.rs` 添加 `store_template` 方法
  - [x] SubTask 3.2: 添加 `get_template` 方法
  - [x] SubTask 3.3: 添加 `get_all_templates` 方法
  - [x] SubTask 3.4: 添加 `delete_template` 方法
  - [x] SubTask 3.5: 添加模板初始化逻辑（创建内置模板）

- [x] Task 4: 实现自动检测功能
  - [x] SubTask 4.1: 在 `parser/template.rs` 实现 `detect_best_template` 方法
  - [x] SubTask 4.2: 实现匹配成功率统计
  - [x] SubTask 4.3: 返回检测结果列表

- [x] Task 5: 添加 Tauri 命令
  - [x] SubTask 5.1: 添加 `get_templates` 命令
  - [x] SubTask 5.2: 添加 `create_template` 命令
  - [x] SubTask 5.3: 添加 `update_template` 命令
  - [x] SubTask 5.4: 添加 `delete_template` 命令
  - [x] SubTask 5.5: 添加 `test_template` 命令（实时预览）
  - [x] SubTask 5.6: 添加 `detect_template` 命令
  - [x] SubTask 5.7: 添加 `export_templates` 命令
  - [x] SubTask 5.8: 添加 `import_templates` 命令
  - [x] SubTask 5.9: 修改 `parse_log` 命令支持模板参数

## 前端任务

- [x] Task 6: 创建模板相关类型定义
  - [x] SubTask 6.1: 创建 `src/lib/types/template.ts`
  - [x] SubTask 6.2: 定义 TypeScript 接口（LogTemplate, LogEvent, TemplateType）

- [x] Task 7: 创建模板 API 封装
  - [x] SubTask 7.1: 在 `src/lib/api/tauri.ts` 添加模板相关 API 调用
  - [x] SubTask 7.2: 添加错误处理

- [x] Task 8: 创建模板 Store
  - [x] SubTask 8.1: 创建 `src/lib/stores/templateStore.ts`
  - [x] SubTask 8.2: 实现模板列表状态管理
  - [x] SubTask 8.3: 实现当前选中模板状态
  - [x] SubTask 8.4: 实现模板 CRUD 操作

- [x] Task 9: 重构 FileBar 组件
  - [x] SubTask 9.1: 添加"日志格式"下拉按钮
  - [x] SubTask 9.2: 实现下拉菜单组件
  - [x] SubTask 9.3: 实现模板选择逻辑
  - [x] SubTask 9.4: 实现自动检测选项

- [x] Task 10: 创建模板管理弹窗
  - [x] SubTask 10.1: 创建 `TemplateManagerModal.svelte`
  - [x] SubTask 10.2: 实现模板列表展示
  - [x] SubTask 10.3: 实现新建/编辑/删除操作
  - [x] SubTask 10.4: 实现导入/导出功能

- [x] Task 11: 创建模板编辑弹窗
  - [x] SubTask 11.1: 创建 `TemplateEditorModal.svelte`
  - [x] SubTask 11.2: 实现模板名称输入
  - [x] SubTask 11.3: 实现正则表达式输入框（大文本）
  - [x] SubTask 11.4: 实现测试日志输入框
  - [x] SubTask 11.5: 实现实时解析预览
  - [x] SubTask 11.6: 实现保存/取消操作

- [x] Task 12: 集成模板到日志解析流程
  - [x] SubTask 12.1: 修改 `logStore.ts` 支持模板选择
  - [x] SubTask 12.2: 实现模板切换后重新解析
  - [x] SubTask 12.3: 实现解析进度显示

## 前端动态字段任务（新增）

- [x] Task 15: 后端支持模板字段元数据
  - [x] SubTask 15.1: 在 LogTemplate 添加 `extra_fields: Vec<String>` 字段
  - [x] SubTask 15.2: 在 LogTemplate 添加 `has_level: bool` 字段
  - [x] SubTask 15.3: 添加 `extract_fields_from_pattern` 方法提取字段列表
  - [x] SubTask 15.4: 修改 LogEntry 添加 `extra: HashMap<String, String>` 字段
  - [x] SubTask 15.5: 修改 LogEntryView 包含 extra 字段

- [x] Task 16: 重构 LogList 组件支持动态列
  - [x] SubTask 16.1: 定义核心列配置（行号、时间、级别、来源、内容）
  - [x] SubTask 16.2: 实现动态 extra_fields 列渲染
  - [x] SubTask 16.3: 实现表格横向滚动
  - [x] SubTask 16.4: 实现列宽调整功能
  - [x] SubTask 16.5: 根据模板 has_level 动态显示/隐藏级别列

- [x] Task 17: 重构级别过滤面板
  - [x] SubTask 17.1: 添加 `hasLevel` 状态到 templateStore
  - [x] SubTask 17.2: 根据模板动态显示/隐藏级别过滤面板
  - [x] SubTask 17.3: 实现级别分布统计（基于当前日志数据）

- [x] Task 18: 实现字段驱动筛选系统
  - [x] SubTask 18.1: 创建 `FilterCondition` 类型定义
  - [x] SubTask 18.2: 创建 `FilterPanel.svelte` 组件
  - [x] SubTask 18.3: 实现字段选择器（核心 + extra_fields）
  - [x] SubTask 18.4: 实现筛选操作符选择（等于、包含、正则、大于、小于）
  - [x] SubTask 18.5: 实现多条件组合（AND/OR）
  - [x] SubTask 18.6: 在 logStore 添加筛选条件状态管理

- [x] Task 19: 重构搜索系统
  - [x] SubTask 19.1: 修改搜索范围选择器支持动态字段
  - [x] SubTask 19.2: 实现字段选择下拉（全部字段 / 特定字段）
  - [x] SubTask 19.3: 保持现有搜索模式（模糊、精确、正则）

- [x] Task 20: 实现未解析日志显示控制
  - [x] SubTask 20.1: 在 LogEntry 添加 `is_parsed: bool` 字段
  - [x] SubTask 20.2: 在 logStore 添加 `showUnparsed` 状态
  - [x] SubTask 20.3: 在 ControlBar 添加显示开关
  - [x] SubTask 20.4: 实现未解析日志的特殊样式

- [x] Task 21: 状态管理完善
  - [x] SubTask 21.1: 在 templateStore 添加 `currentFields` 状态
  - [x] SubTask 21.2: 实现模板切换时重置筛选条件
  - [x] SubTask 21.3: 实现筛选条件持久化（可选）

## 测试任务

- [x] Task 13: 后端单元测试
  - [x] SubTask 13.1: 测试 LogTemplate 结构
  - [x] SubTask 13.2: 测试 LogEvent 解析
  - [x] SubTask 13.3: 测试命名捕获组映射
  - [x] SubTask 13.4: 测试自动检测功能
  - [x] SubTask 13.5: 测试模板存储

- [ ] Task 14: 集成测试
  - [ ] SubTask 14.1: 测试完整解析流程
  - [ ] SubTask 14.2: 测试前端模板选择
  - [ ] SubTask 14.3: 测试模板管理功能

# Task Dependencies

- Task 2 依赖 Task 1（需要数据结构定义）
- Task 3 依赖 Task 1（需要数据结构定义）
- Task 4 依赖 Task 2（需要解析器）
- Task 5 依赖 Task 2, Task 3, Task 4（需要解析器和存储）
- Task 7 依赖 Task 5（需要后端命令）
- Task 8 依赖 Task 6, Task 7（需要类型和 API）
- Task 9 依赖 Task 8（需要 Store）
- Task 10 依赖 Task 8（需要 Store）
- Task 11 依赖 Task 7（需要测试 API）
- Task 12 依赖 Task 9, Task 5（需要 UI 和后端支持）
- Task 13 可与 Task 1-5 并行开发
- Task 14 依赖所有前置任务完成
- Task 15 依赖 Task 1（需要修改数据结构）
- Task 16 依赖 Task 15（需要字段元数据）
- Task 17 依赖 Task 15（需要 has_level 字段）
- Task 18 依赖 Task 15, Task 16（需要字段列表）
- Task 19 依赖 Task 15（需要字段列表）
- Task 20 依赖 Task 15（需要 is_parsed 字段）
- Task 21 依赖 Task 15-20（需要完整字段支持）
