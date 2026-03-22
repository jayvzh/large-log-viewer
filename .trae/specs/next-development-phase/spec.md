# LogViewer 下一步开发计划

## Why

LogViewer 已完成核心功能（大文件加载、日志解析、模板系统、性能优化），但仍有若干待改进事项需要处理，包括已知问题修复、功能增强和代码质量提升。

## What Changes

### 已知问题修复
- 时间过滤后端优化（确认是否已正确使用时间索引）

### 功能增强
- 导出功能：支持导出过滤后的日志和搜索结果
- **多实例支持**：移除单实例限制，允许多个 EXE 实例同时运行
  - 每个实例使用独立的数据库目录（基于进程ID）
  - 移除 `tauri-plugin-single-instance` 插件
- 进度反馈优化：基于时间和数量的双重策略

### 代码质量
- 移除调试日志：清理 `commands/mod.rs` 中的 `eprintln!` 调试输出
- 错误处理改进：添加错误码，优化错误信息
- 单元测试覆盖：后端核心逻辑测试

## Impact

- Affected specs: 无
- Affected code:
  - `src-tauri/src/main.rs` - 移除单实例插件
  - `src-tauri/Cargo.toml` - 移除单实例依赖
  - `src-tauri/src/database/mod.rs` - 支持多实例数据库隔离
  - `src-tauri/src/commands/mod.rs` - 移除调试日志
  - `src-tauri/src/` - 添加单元测试
  - `src/lib/` - 新增导出功能组件

## ADDED Requirements

### Requirement: 多实例支持

The system SHALL support running multiple instances simultaneously.

#### Scenario: 打开多个文件
- **WHEN** 用户双击多个日志文件或多次启动程序
- **THEN** 每个文件在独立的进程实例中打开

#### Scenario: 数据库隔离
- **WHEN** 多个实例同时运行
- **THEN** 每个实例使用独立的数据库目录，互不干扰

#### Scenario: 实例独立运行
- **WHEN** 一个实例崩溃或关闭
- **THEN** 其他实例不受影响

### Requirement: 日志导出功能

The system SHALL provide log export functionality.

#### Scenario: 导出过滤后的日志
- **WHEN** 用户设置过滤条件后点击"导出"按钮
- **THEN** 系统将过滤后的日志导出为文本文件

#### Scenario: 导出搜索结果
- **WHEN** 用户执行搜索后点击"导出"按钮
- **THEN** 系统将搜索结果导出为文本文件

### Requirement: 代码质量提升

The system SHALL have clean production code without debug output.

#### Scenario: 移除调试日志
- **WHEN** 编译发布版本
- **THEN** 不包含 `eprintln!` 等调试输出

#### Scenario: 错误处理
- **WHEN** 发生错误
- **THEN** 系统返回友好的错误信息和错误码

## MODIFIED Requirements

### Requirement: 时间过滤优化

当前 `filterLogs` API 已在后端使用时间索引，需确认前端正确调用。

## REMOVED Requirements

### Requirement: 单实例模式

**Reason**: 用户希望每个日志文件在独立进程中打开，提供更好的隔离性和稳定性。
**Migration**: 移除 `tauri-plugin-single-instance` 插件，修改数据库路径使用进程ID隔离。
