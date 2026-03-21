# LogViewer 性能优化 Spec

## Why
当前实现存在严重的性能问题：并行解析函数未被调用、零拷贝设计被破坏、数据库查询效率低下、全文搜索只处理前 10000 条记录。这些问题导致大文件处理缓慢、内存占用过高，无法满足 README 中声称的性能目标。

## What Changes
- **启用并行解析**：将单线程解析改为使用 rayon 多线程并行解析
- **修复零拷贝实现**：消除 MmapLineIterator 中的 `to_vec()` 调用，实现真正的零拷贝
- **优化数据库查询**：添加二级索引，实现 O(1) 复杂度的偏移量定位
- **实现全文搜索索引**：构建倒排索引，支持百万级日志的快速搜索
- **添加时间戳索引**：后端实现时间范围过滤索引
- **实现真正的分页加载**：前端按需加载页面，而非一次性加载 10000 条
- **后端过滤替代前端过滤**：将过滤逻辑移至后端，减少数据传输
- **添加文件大小检查**：防止处理超大文件导致系统问题

## Impact
- Affected specs: 核心解析模块、数据库模块、查询引擎、前端状态管理
- Affected code:
  - `src-tauri/src/parser/mod.rs` - 并行解析
  - `src-tauri/src/reader/mod.rs` - 零拷贝迭代器
  - `src-tauri/src/database/mod.rs` - 索引优化
  - `src-tauri/src/commands/mod.rs` - 查询命令
  - `src/lib/stores/logStore.ts` - 分页加载

## ADDED Requirements

### Requirement: 并行解析
系统 SHALL 使用 rayon 多线程并行解析日志文件，充分利用多核 CPU。

#### Scenario: 大文件并行解析
- **WHEN** 用户加载 1GB 日志文件
- **THEN** 系统使用所有可用 CPU 核心并行解析
- **AND** 解析速度相比单线程提升 3-3.5 倍

### Requirement: 零拷贝读取
系统 SHALL 使用零拷贝技术读取内存映射文件，避免不必要的内存分配。

#### Scenario: 内存映射读取
- **WHEN** 系统读取日志文件
- **THEN** MmapLineIterator 返回引用而非拷贝
- **AND** 内存占用降低 40-60%

### Requirement: 高效数据库查询
系统 SHALL 使用二级索引实现 O(1) 复杂度的偏移量定位。

#### Scenario: 快速跳转
- **WHEN** 用户跳转到第 100 万条日志
- **THEN** 响应时间 < 50ms
- **AND** 无需遍历前面的记录

### Requirement: 全文搜索索引
系统 SHALL 构建倒排索引支持全文搜索。

#### Scenario: 百万级日志搜索
- **WHEN** 用户在百万级日志中搜索关键词
- **THEN** 响应时间 < 100ms
- **AND** 搜索覆盖所有日志记录

### Requirement: 时间戳索引
系统 SHALL 在后端维护时间戳索引，支持高效的时间范围过滤。

#### Scenario: 时间范围过滤
- **WHEN** 用户选择时间范围过滤
- **THEN** 后端使用时间索引快速定位
- **AND** 响应时间 < 50ms

### Requirement: 分页加载
系统 SHALL 实现按需分页加载，而非一次性加载所有数据。

#### Scenario: 滚动加载
- **WHEN** 用户滚动日志列表
- **THEN** 系统动态加载可见区域的日志
- **AND** 内存中只保留当前需要的页面

### Requirement: 后端过滤
系统 SHALL 在后端执行过滤操作，减少前端数据传输量。

#### Scenario: 组合过滤
- **WHEN** 用户同时应用级别、时间、搜索过滤
- **THEN** 后端使用位图索引组合过滤
- **AND** 只返回匹配的日志条目

### Requirement: 文件大小保护
系统 SHALL 检查文件大小，防止处理超大文件。

#### Scenario: 大文件警告
- **WHEN** 用户尝试打开超过 10GB 的文件
- **THEN** 系统显示错误提示
- **AND** 拒绝加载该文件

## MODIFIED Requirements

### Requirement: 日志解析
系统 SHALL 使用并行解析处理日志文件，而非单线程循环。

**原实现**：单线程循环调用 `parse_line`
**新实现**：使用 `par_iter` 并行调用 `parse_lines_parallel`

### Requirement: 内存映射迭代器
MmapLineIterator SHALL 返回切片引用而非 Vec 拷贝。

**原实现**：`data: Vec<u8>` + `to_vec()`
**新实现**：`data: &'a [u8]` + 直接引用

### Requirement: 数据库查询
get_entries SHALL 使用范围索引直接定位偏移量。

**原实现**：遍历跳过前 offset 条记录
**新实现**：使用 `range(start_key..)` 直接定位

## REMOVED Requirements
无移除的需求。

## 预期性能提升

| 指标 | 当前实现 | 改进后 | 提升幅度 |
|------|----------|--------|----------|
| 1GB 文件解析时间 | ~45s | ~12s | 3.75x |
| 内存占用（1GB文件） | ~1.5GB | ~600MB | 60%↓ |
| 跳转到第100万条 | ~3s | <50ms | 60x |
| 全文搜索响应 | N/A（只搜前1万条） | <100ms | ∞ |
| 级别过滤响应 | ~500ms | <10ms | 50x |
