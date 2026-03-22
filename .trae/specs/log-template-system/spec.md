# 日志解析模板系统 Spec

## Why
当前系统使用固定字段（时间/级别/记录者/内容）解析日志，硬编码了5种固定的正则表达式模式，无法支持多样化的日志格式。用户无法自定义解析规则，导致非标准格式的日志无法正确解析，影响日志查看器的通用性。

## What Changes
- **重构 Parser 层**：引入 `LogTemplate` 结构，支持用户自定义正则表达式模板
- **支持命名捕获组**：所有解析基于 regex named groups (`?P<field>`)
- **标准化输出结构**：引入 `LogEvent` 结构，支持 `extra` 字段存储额外捕获内容
- **模板匹配流程**：遍历模板列表尝试匹配，成功则解析为 `LogEvent`
- **自动检测机制**：对前100行进行统计，选择匹配成功率最高的模板
- **sled 持久化存储**：模板存储在 sled 数据库中
- **前端模板管理界面**：FileBar 新增模板选择下拉菜单，支持模板管理弹窗
- **前端动态字段支持**：表格列、筛选、搜索均支持模板定义的动态字段

## Impact
- Affected specs: 核心解析模块、数据库模块、前端组件
- Affected code:
  - `src-tauri/src/parser/mod.rs` - 重构为模板化解析
  - `src-tauri/src/models/mod.rs` - 新增 LogTemplate、LogEvent 结构
  - `src-tauri/src/database/mod.rs` - 新增模板存储方法
  - `src-tauri/src/commands/mod.rs` - 新增模板相关命令
  - `src/lib/components/FileBar.svelte` - 新增模板选择UI
  - `src/lib/components/LogList.svelte` - 支持动态列
  - `src/lib/components/ControlBar.svelte` - 支持字段驱动筛选
  - `src/lib/stores/logStore.ts` - 集成模板选择逻辑

## ADDED Requirements

### Requirement: LogTemplate 结构定义
系统 SHALL 定义 `LogTemplate` 结构用于表示日志解析模板。

```rust
struct LogTemplate {
    name: String,
    pattern: String,
    field_mapping: HashMap<String, String>,
    is_builtin: bool,
    created_at: i64,
    updated_at: i64,
}
```

#### Scenario: 创建内置模板
- **WHEN** 系统初始化时
- **THEN** 自动创建5个内置模板（对应现有5种日志格式）
- **AND** 内置模板标记为 `is_builtin: true`
- **AND** 内置模板不可删除但可复制

### Requirement: LogEvent 标准化输出结构
系统 SHALL 定义 `LogEvent` 结构作为解析输出。

```rust
struct LogEvent {
    timestamp: Option<String>,
    level: Option<String>,
    source: Option<String>,
    message: String,
    extra: HashMap<String, String>,
}
```

#### Scenario: 解析包含额外字段的日志
- **WHEN** 用户定义模板包含非标准字段（如 `?P<thread_id>`）
- **THEN** 该字段存储在 `extra` HashMap 中
- **AND** 前端可展示所有额外字段

### Requirement: 命名捕获组解析
系统 SHALL 基于正则表达式命名捕获组进行解析。

#### Scenario: 标准字段映射
- **WHEN** 正则包含命名组 `timestamp`、`level`、`source`、`message`
- **THEN** 自动映射到 `LogEvent` 对应字段
- **AND** 支持字段别名（如 `time` -> `timestamp`，`msg` -> `message`）

#### Scenario: 无效正则处理
- **WHEN** 用户输入无效的正则表达式
- **THEN** 返回明确的错误信息
- **AND** 指出具体的语法错误位置

### Requirement: 模板匹配流程
系统 SHALL 按优先级遍历模板列表进行匹配。

#### Scenario: 模板匹配成功
- **WHEN** 日志行匹配某个模板的正则表达式
- **THEN** 使用该模板解析日志
- **AND** 返回解析后的 `LogEvent`

#### Scenario: 所有模板匹配失败
- **WHEN** 日志行无法匹配任何模板
- **THEN** 使用默认处理（整行作为 message）
- **AND** `timestamp`、`level`、`source` 设为 None

### Requirement: 自动检测最佳模板
系统 SHALL 支持自动检测最适合当前日志文件的模板。

#### Scenario: 自动检测流程
- **WHEN** 用户选择"自动检测"
- **THEN** 读取文件前100行
- **AND** 对每个模板统计匹配成功率
- **AND** 选择成功率最高的模板
- **AND** 如果所有模板成功率都低于50%，提示用户创建新模板

#### Scenario: 检测结果展示
- **WHEN** 自动检测完成
- **THEN** 显示各模板的匹配成功率
- **AND** 高亮推荐模板

### Requirement: 模板持久化存储
系统 SHALL 使用 sled 数据库存储用户自定义模板。

#### Scenario: 存储模板
- **WHEN** 用户创建或修改模板
- **THEN** 模板以 JSON 格式存储到 sled
- **AND** key 格式为 `template:{name}`

#### Scenario: 加载模板列表
- **WHEN** 应用启动
- **THEN** 从 sled 加载所有用户模板
- **AND** 合并内置模板列表

### Requirement: 前端模板选择UI
系统 SHALL 在 FileBar 组件提供模板选择功能。

#### Scenario: 模板选择下拉菜单
- **WHEN** 用户点击"日志格式"按钮
- **THEN** 显示下拉菜单包含：
  - 自动检测
  - 内置模板列表
  - 我的模板列表
  - 模板管理入口

#### Scenario: 选择模板后解析
- **WHEN** 用户选择某个模板
- **THEN** 使用该模板重新解析当前文件
- **AND** 更新日志列表显示

### Requirement: 模板管理弹窗
系统 SHALL 提供模板管理界面。

#### Scenario: 模板管理界面
- **WHEN** 用户点击"模板管理"
- **THEN** 显示弹窗包含：
  - 模板列表（名称、类型、操作）
  - 新建模板按钮
  - 导入/导出按钮

#### Scenario: 模板操作
- **WHEN** 用户对模板执行操作
- **THEN** 支持：新建、编辑、复制、删除（仅用户模板）、导入、导出

### Requirement: 模板编辑弹窗
系统 SHALL 提供模板编辑界面。

#### Scenario: 编辑模板界面
- **WHEN** 用户创建或编辑模板
- **THEN** 显示表单包含：
  - 模板名称输入框
  - 正则表达式输入框（大文本）
  - 测试日志输入框
  - 实时解析结果展示区域

#### Scenario: 实时预览解析
- **WHEN** 用户修改正则或测试日志
- **THEN** 实时调用后端解析接口
- **AND** 显示解析结果 JSON
- **AND** 高亮匹配的字段

### Requirement: 后端模板解析命令
系统 SHALL 提供模板相关的 Tauri 命令。

#### Scenario: 测试模板解析
- **WHEN** 前端调用 `test_template` 命令
- **THEN** 接收模板正则和测试日志
- **AND** 返回解析结果或错误信息

#### Scenario: 获取模板列表
- **WHEN** 前端调用 `get_templates` 命令
- **THEN** 返回所有模板（内置 + 用户）

### Requirement: 动态表格列
系统 SHALL 支持基于模板字段的动态表格列。

#### Scenario: 核心字段列
- **WHEN** 显示日志列表
- **THEN** 始终显示核心字段列：行号、时间、级别、来源、内容
- **AND** 列宽可调整

#### Scenario: 动态额外字段列
- **WHEN** 当前模板包含 extra_fields（如 thread_id、request_id）
- **THEN** 在核心字段后动态添加这些列
- **AND** 列表支持横向滚动

#### Scenario: 无级别字段模板
- **WHEN** 模板不包含 level 字段
- **THEN** 隐藏级别列
- **AND** 隐藏级别过滤面板

### Requirement: 动态级别过滤
系统 SHALL 根据模板字段动态显示级别过滤。

#### Scenario: 有级别字段时显示过滤
- **WHEN** 当前模板包含 level 字段
- **THEN** 显示级别过滤面板
- **AND** 自动统计各级别数量

#### Scenario: 无级别字段时隐藏过滤
- **WHEN** 当前模板不包含 level 字段
- **THEN** 隐藏级别过滤面板

### Requirement: 字段驱动筛选系统
系统 SHALL 支持基于字段的筛选。

#### Scenario: 字段选择器
- **WHEN** 用户打开筛选面板
- **THEN** 显示可用字段列表（核心字段 + extra_fields）
- **AND** 用户可选择要筛选的字段

#### Scenario: 多条件组合筛选
- **WHEN** 用户添加多个筛选条件
- **THEN** 支持 AND/OR 组合
- **AND** 每个条件可选择字段、操作符、值

#### Scenario: 筛选操作符
- **WHEN** 用户设置筛选条件
- **THEN** 支持操作符：等于、包含、正则匹配、大于、小于

### Requirement: 字段驱动搜索系统
系统 SHALL 支持基于字段的搜索。

#### Scenario: 搜索字段选择
- **WHEN** 用户进行搜索
- **THEN** 可选择搜索范围（全部字段 / 特定字段）
- **AND** 字段列表来自当前模板

#### Scenario: 搜索模式
- **WHEN** 用户输入搜索词
- **THEN** 支持模式：模糊匹配、精确匹配、正则表达式

### Requirement: 模板字段状态管理
系统 SHALL 管理模板相关的状态。

#### Scenario: 当前模板状态
- **WHEN** 用户选择模板
- **THEN** 保存当前模板信息
- **AND** 更新可用字段列表

#### Scenario: 字段集合状态
- **WHEN** 模板切换
- **THEN** 更新字段集合（core + extra）
- **AND** 重置筛选条件

#### Scenario: 过滤条件状态
- **WHEN** 用户设置筛选条件
- **THEN** 保存过滤条件
- **AND** 支持清除所有条件

### Requirement: 未解析日志显示控制
系统 SHALL 支持控制未解析日志的显示。

#### Scenario: 显示未解析日志开关
- **WHEN** 用户开启"显示未解析日志"
- **THEN** 显示所有日志行（包括匹配失败的）
- **AND** 未解析的日志行有特殊标记

#### Scenario: 隐藏未解析日志
- **WHEN** 用户关闭"显示未解析日志"
- **THEN** 仅显示成功匹配模板的日志行

## MODIFIED Requirements

### Requirement: 日志解析模块
系统 SHALL 使用模板化解析替代固定模式解析。

**原实现**：硬编码5种 `LOG_PATTERN_*` 正则表达式
**新实现**：基于 `LogTemplate` 列表动态匹配

### Requirement: LogEntry 结构
LogEntry 结构 SHALL 保持兼容，但从 LogEvent 转换生成。

**原实现**：直接解析生成 LogEntry
**新实现**：先解析为 LogEvent，再转换为 LogEntry

### Requirement: LogList 组件
LogList 组件 SHALL 支持动态列显示。

**原实现**：固定5列（行号、时间、级别、来源、内容）
**新实现**：核心列 + 动态 extra_fields 列，支持横向滚动

### Requirement: ControlBar 组件
ControlBar 组件 SHALL 支持字段驱动筛选。

**原实现**：固定时间筛选 + 固定范围搜索
**新实现**：字段选择器 + 多条件组合筛选 + 字段搜索

## REMOVED Requirements

### Requirement: 固定日志模式
**Reason**: 被模板系统取代
**Migration**: 将现有5种模式转换为内置模板

### Requirement: 固定级别过滤面板
**Reason**: 改为根据模板动态显示
**Migration**: 检测模板是否有 level 字段决定是否显示
