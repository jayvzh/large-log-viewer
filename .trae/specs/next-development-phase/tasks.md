# Tasks

## 第一阶段：代码质量改进

- [x] Task 1: 移除调试日志
  - [x] SubTask 1.1: 审查 `src-tauri/src/commands/mod.rs` 中的 `eprintln!` 调用
  - [x] SubTask 1.2: 移除或替换为条件日志（使用 `log` crate 或 feature flag）
  - [x] SubTask 1.3: 验证编译通过且功能正常

- [x] Task 2: 确认时间过滤后端实现
  - [x] SubTask 2.1: 验证 `filter_logs` 命令是否正确使用时间索引
  - [x] SubTask 2.2: 验证前端 `filterLogs` API 是否正确传递时间参数
  - [x] SubTask 2.3: 如有问题，修复时间过滤实现

## 第二阶段：多实例支持

- [x] Task 3: 实现多实例支持
  - [x] SubTask 3.1: 移除 `tauri-plugin-single-instance` 依赖
  - [x] SubTask 3.2: 修改 `main.rs` 移除单实例插件初始化代码
  - [x] SubTask 3.3: 修改数据库路径，使用进程ID隔离不同实例的数据目录
  - [x] SubTask 3.4: 移除 `open-file-argument` 事件监听（单实例相关）
  - [x] SubTask 3.5: 测试多实例同时运行是否正常

## 第三阶段：模板管理优化

- [x] Task 4: 模板存储优化
  - [x] SubTask 4.1: 创建模板文件存储模块，支持读写 `{exe_dir}/data/templates.json`
  - [x] SubTask 4.2: 修改 `get_all_templates` 从 JSON 文件读取用户模板
  - [x] SubTask 4.3: 修改 `store_template` 写入 JSON 文件
  - [x] SubTask 4.4: 修改 `delete_template` 更新 JSON 文件
  - [x] SubTask 4.5: 实现旧数据库模板迁移逻辑（首次启动时）
  - [x] SubTask 4.6: 添加文件锁机制，防止多实例并发写入冲突

- [x] Task 5: 模板检测优化
  - [x] SubTask 5.1: 修改 `detect_template` 命令，限制只读取文件前 100 行
  - [x] SubTask 5.2: 验证检测速度提升

## 第四阶段：导出功能

- [x] Task 6: 实现日志导出功能
  - [x] SubTask 6.1: 后端添加 `export_logs` Tauri 命令
  - [x] SubTask 6.2: 支持导出过滤后的日志
  - [x] SubTask 6.3: 支持导出搜索结果
  - [x] SubTask 6.4: 前端添加导出按钮和导出对话框
  - [x] SubTask 6.5: 支持选择导出格式（纯文本、JSON）

## 第五阶段：测试与完善

- [x] Task 7: 添加单元测试
  - [x] SubTask 7.1: 后端解析器测试
  - [x] SubTask 7.2: 后端数据库操作测试
  - [x] SubTask 7.3: 后端过滤功能测试
  - [x] SubTask 7.4: 模板存储模块测试

- [x] Task 8: 功能验证
  - [x] SubTask 8.1: 验证多实例同时运行正常
  - [x] SubTask 8.2: 验证多实例共享模板正常
  - [x] SubTask 8.3: 验证导出功能正常工作
  - [x] SubTask 8.4: 验证时间过滤使用后端索引

## 第六阶段：文档更新

- [x] Task 9: 更新项目文档
  - [x] SubTask 9.1: 更新 README.md，添加新功能说明
  - [x] SubTask 9.2: 更新 CHANGELOG.md，记录本次变更
  - [x] SubTask 9.3: 更新 TODO.md，标记已完成事项

# Task Dependencies

- [Task 3] 可独立进行
- [Task 4] 依赖 [Task 3] 完成（多实例支持后模板共享才有意义）
- [Task 5] 可独立进行
- [Task 6] 可独立进行
- [Task 7] 依赖 [Task 1] 完成
- [Task 8] 依赖 [Task 1-7] 完成
- [Task 9] 依赖 [Task 8] 完成（功能验证通过后更新文档）
