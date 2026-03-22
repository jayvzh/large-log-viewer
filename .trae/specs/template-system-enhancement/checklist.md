# Checklist

## 第一阶段：紧急修复

- [x] templateStore 使用 `extra_fields` 而非 `field_mapping`
- [x] templateStore 正确返回 `has_level`、`has_timestamp`、`has_source`
- [x] FilterPanel 始终显示，不再被条件隐藏
- [x] LogList 正确显示模板定义的动态列
- [x] Nginx Access Log 模板的 IP、Status 等字段在 LogList 中显示
- [x] FilterPanel 的字段下拉列表包含 extra 字段选项

## 第二阶段：后端索引

- [x] 数据库支持 extra 字段索引存储
- [x] parse_log 命令创建 extra 字段索引
- [x] filter_logs 命令支持 extra 字段过滤参数
- [x] extra 字段过滤使用索引（等于、包含操作符）
- [x] extra 字段正则过滤回退到全量扫描

## 第三阶段：前端集成

- [x] FilterPanel 的 extra 字段条件传递给后端
- [x] 后端过滤替代前端过滤（extra 字段部分）
- [x] FilterPanel 支持折叠/展开
- [x] 折叠状态持久化到 localStorage
- [x] "清除所有筛选"按钮正常工作

## 第四阶段：架构优化

- [x] LogTemplate 包含 priority 字段
- [x] 模板列表按 priority 排序
- [x] 模板编辑器支持设置优先级
- [x] AppState 缓存 LogTemplateParser 实例
- [x] 模板变更时清除解析器缓存

## 第五阶段：验证

- [x] Nginx Access Log 模板完整功能测试通过
- [x] Extra 字段过滤性能测试通过（10 万条日志 < 100ms）
- [x] 多模板场景测试通过
- [x] 边界情况测试通过（无 extra 字段、空值等）
- [x] CHANGELOG.md 已更新
- [x] templatesys.md 已更新
- [x] TODO.md 已更新
