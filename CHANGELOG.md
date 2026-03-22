# 变更日志

## [Unreleased]

### Changed
- 将日志字段 "记录者" (logger) 重命名为 "来源" (source)
  - 前端组件：LogList.svelte、LogDetail.svelte、ControlBar.svelte
  - 前端状态管理：logStore.ts
  - 后端模型：models/mod.rs (LogEntry、LogEntryView)
  - 后端解析器：parser/mod.rs (正则表达式捕获组名称)
  - 后端命令和数据库：commands/mod.rs、database/mod.rs
  - 文档：README.md
