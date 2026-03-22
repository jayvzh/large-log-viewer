# Tasks

## 第一阶段：紧急修复（核心功能恢复）

- [ ] Task 1: 修复 templateStore 字段提取逻辑
  - [ ] SubTask 1.1: 修改 `setCurrentTemplate` 方法，使用 `template.extra_fields` 而非 `template.field_mapping`
  - [ ] SubTask 1.2: 修改 `extractFields` 函数，直接返回 `template.extra_fields`
  - [ ] SubTask 1.3: 修改 `checkHasLevel` 函数，使用 `template.has_level`
  - [ ] SubTask 1.4: 验证 Nginx Access Log 模板的 extra_fields 正确传递到前端

- [ ] Task 2: 修复 FilterPanel 显示逻辑
  - [ ] SubTask 2.1: 修改 `+page.svelte`，移除 FilterPanel 的条件渲染
  - [ ] SubTask 2.2: FilterPanel 始终显示，提供核心字段筛选
  - [ ] SubTask 2.3: 当有 extra 字段时，自动添加到筛选选项

- [ ] Task 3: 修复 LogList 动态列显示
  - [ ] SubTask 3.1: 修改 LogList 组件，从 templateStore 获取 extraFields
  - [ ] SubTask 3.2: 确保动态列按模板定义顺序显示
  - [ ] SubTask 3.3: 验证 Nginx Access Log 的 IP、Status 等字段正确显示

## 第二阶段：后端 Extra 字段索引

- [ ] Task 4: 设计 Extra 字段索引结构
  - [ ] SubTask 4.1: 确定索引键格式：`extra_index:{file_id}:{field_name}:{value}` → RoaringBitmap
  - [ ] SubTask 4.2: 或使用 `extra_index:{file_id}:{field_name}` → HashMap<String, RoaringBitmap>
  - [ ] SubTask 4.3: 评估内存占用和查询效率

- [ ] Task 5: 实现 Extra 字段索引存储
  - [ ] SubTask 5.1: 在 `parse_log` 命令中收集 extra 字段值
  - [ ] SubTask 5.2: 实现 `store_extra_index` 数据库方法
  - [ ] SubTask 5.3: 为每个 extra 字段值创建 RoaringBitmap 索引
  - [ ] SubTask 5.4: 优化：只为模板定义的 extra_fields 创建索引

- [ ] Task 6: 实现 Extra 字段过滤 API
  - [ ] SubTask 6.1: 扩展 `filter_logs` 命令，支持 extra 字段过滤参数
  - [ ] SubTask 6.2: 实现 `filter_by_extra` 数据库方法
  - [ ] SubTask 6.3: 支持等于、包含操作符（利用索引）
  - [ ] SubTask 6.4: 正则操作符回退到全量扫描

## 第三阶段：前端集成

- [ ] Task 7: FilterPanel 后端过滤集成
  - [ ] SubTask 7.1: 修改 `applyFilters` 方法，将 extra 字段条件传递给后端
  - [ ] SubTask 7.2: 扩展 `filterLogs` API 调用，添加 extraConditions 参数
  - [ ] SubTask 7.3: 后端过滤替代前端过滤（extra 字段部分）

- [ ] Task 8: 用户体验优化
  - [ ] SubTask 8.1: FilterPanel 添加折叠/展开功能
  - [ ] SubTask 8.2: 筛选条件持久化（localStorage）
  - [ ] SubTask 8.3: 添加"清除所有筛选"按钮

## 第四阶段：架构优化

- [ ] Task 9: 模板优先级支持
  - [ ] SubTask 9.1: LogTemplate 结构添加 `priority: u32` 字段
  - [ ] SubTask 9.2: 模板列表按 priority 排序
  - [ ] SubTask 9.3: 前端模板编辑器添加优先级设置

- [ ] Task 10: 模板解析器缓存
  - [ ] SubTask 10.1: AppState 添加 `template_parser: RwLock<Option<Arc<LogTemplateParser>>>`
  - [ ] SubTask 10.2: 模板变更时清除缓存
  - [ ] SubTask 10.3: 复用解析器实例，避免重复编译正则

## 第五阶段：测试与验证

- [ ] Task 11: 功能测试
  - [ ] SubTask 11.1: 测试 Nginx Access Log 模板解析和显示
  - [ ] SubTask 11.2: 测试 Extra 字段过滤性能
  - [ ] SubTask 11.3: 测试多模板场景
  - [ ] SubTask 11.4: 测试边界情况（无 extra 字段、空值等）

- [ ] Task 12: 文档更新
  - [ ] SubTask 12.1: 更新 CHANGELOG.md
  - [ ] SubTask 12.2: 更新 templatesys.md（标记已实现的改进）
  - [ ] SubTask 12.3: 更新 TODO.md

# Task Dependencies

- [Task 1] 是最高优先级，阻塞其他所有任务
- [Task 2] 和 [Task 3] 依赖 [Task 1] 完成
- [Task 4-6] 可并行进行，但建议在 [Task 1-3] 完成后开始
- [Task 7] 依赖 [Task 6] 完成
- [Task 8] 可独立进行
- [Task 9-10] 可独立进行，属于架构优化
- [Task 11] 依赖所有功能任务完成
- [Task 12] 依赖 [Task 11] 完成

# Priority Justification

## 本次纳入的改进（来自 templatesys.md）

### 短期改进（低成本）- 全部纳入
1. ✅ **实现模板优先级排序** - Task 9
2. ✅ **复用 LogTemplateParser 实例** - Task 10
3. ⚠️ **增强 field_mapping 功能** - 替换为使用 extra_fields（Task 1）

### 中期改进（中等成本）- 部分纳入
1. ✅ **Extra 字段索引** - Task 4-6（核心功能，必须实现）
2. ❌ **多模板匹配模式** - 暂不纳入，需要更复杂的设计
3. ❌ **类型推断** - 暂不纳入，可作为后续迭代
4. ❌ **多行日志支持** - 暂不纳入，需要独立规划

## 本次纳入的改进（来自 TODO.md）

### 性能优化
- ❌ **数据库查询优化** - 暂不纳入，当前性能可接受
- ❌ **前端缓存策略优化** - 暂不纳入，当前实现已足够

### 功能增强
- ❌ **文件大小限制** - 已有 10GB 限制
- ❌ **进度反馈优化** - 当前实现已足够

## 不纳入的理由

1. **多模板匹配模式**：需要重新设计匹配算法，影响面大，建议独立规划
2. **类型推断**：需要修改数据模型，增加复杂度，可作为后续迭代
3. **多行日志支持**：需要修改解析器核心逻辑，影响面大，建议独立规划
4. **JSON 日志解析器**：可作为独立功能开发，不影响当前模板系统
