# Checklist

## 代码质量改进

- [x] `commands/mod.rs` 中无 `eprintln!` 调试输出
- [x] 时间过滤正确使用后端时间索引

## 多实例支持

- [x] `tauri-plugin-single-instance` 依赖已移除
- [x] `main.rs` 中单实例插件代码已移除
- [x] 数据库路径使用进程ID隔离
- [x] 多个实例可同时运行
- [x] 各实例数据互不干扰

## 模板管理优化

### 模板存储
- [x] 模板文件存储模块实现完成
- [x] 用户模板存储到 `{exe_dir}/data/templates.json`
- [x] `get_all_templates` 从 JSON 文件读取用户模板
- [x] `store_template` 写入 JSON 文件
- [x] `delete_template` 更新 JSON 文件
- [x] 旧数据库模板迁移逻辑实现
- [x] 文件锁机制防止并发写入冲突

### 模板检测
- [x] `detect_template` 只读取文件前 100 行
- [x] 大文件检测速度明显提升

## 导出功能

- [x] 后端 `export_logs` 命令实现完成
- [x] 支持导出过滤后的日志
- [x] 支持导出搜索结果
- [x] 前端导出按钮正常显示
- [x] 导出对话框支持格式选择
- [x] 导出的文件内容正确

## 测试覆盖

- [x] 后端解析器测试通过
- [x] 后端数据库操作测试通过
- [x] 后端过滤功能测试通过
- [x] 模板存储模块测试通过

## 编译验证

- [x] Rust 后端编译通过 (cargo check)
- [x] TypeScript 前端类型检查通过 (pnpm check)
- [x] 后端单元测试全部通过 (cargo test)

## 文档更新

- [x] README.md 已更新，包含新功能说明
- [x] CHANGELOG.md 已更新，记录本次变更
- [x] TODO.md 已更新，标记已完成事项
