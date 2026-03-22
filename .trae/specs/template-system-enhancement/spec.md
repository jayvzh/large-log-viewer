# 模板系统增强与动态字段支持 Spec

## Why

当前模板系统存在多个关键问题，严重影响用户体验和功能完整性：

1. **FilterPanel 几乎不可见**：由于 `templateStore.getCurrentFields()` 返回空数组（依赖未实现的 `field_mapping`），导致 FilterPanel 组件在大多数情况下不显示
2. **动态字段列不工作**：LogList 的动态列依赖 `extraFields`，但该值始终为空
3. **Extra 字段无法高效过滤**：当前只能在前端客户端过滤，无法利用后端索引
4. **Nginx Access Log 等模板无法正确展示**：打开 access.log 后，IP、状态码等关键字段无法在列表中显示

## What Changes

### 核心修复
- **修复 templateStore 的字段提取逻辑**：使用 `extra_fields` 而非未实现的 `field_mapping`
- **修复 FilterPanel 显示条件**：始终显示或根据更合理的条件显示
- **修复 LogList 动态列**：正确获取并显示模板的额外字段

### 功能增强
- **后端 Extra 字段索引**：为常用 extra 字段创建索引，支持高效过滤
- **后端 Extra 字段过滤 API**：新增支持 extra 字段过滤的命令
- **前端 Extra 字段过滤集成**：FilterPanel 支持 extra 字段的后端过滤

### 架构优化
- **模板优先级支持**：添加 priority 字段，支持模板匹配顺序控制
- **模板解析器缓存**：在 AppState 中缓存 LogTemplateParser 实例

## Impact

- Affected specs: 模板系统、过滤系统、日志列表显示
- Affected code:
  - `src-tauri/src/models/mod.rs` - LogTemplate 添加 priority 字段
  - `src-tauri/src/parser/template.rs` - 解析器缓存
  - `src-tauri/src/database/mod.rs` - Extra 字段索引
  - `src-tauri/src/commands/mod.rs` - Extra 字段过滤命令
  - `src/lib/stores/templateStore.ts` - 字段提取逻辑修复
  - `src/lib/components/FilterPanel.svelte` - 显示条件优化
  - `src/lib/components/LogList.svelte` - 动态列修复
  - `src/routes/+page.svelte` - FilterPanel 显示逻辑

## ADDED Requirements

### Requirement: 模板字段元数据正确传递

系统应正确从模板的 `extra_fields`、`has_level`、`has_timestamp`、`has_source` 字段提取元数据，并传递给前端。

#### Scenario: 使用 Nginx Access Log 模板
- **GIVEN** 用户选择 "Nginx Access Log" 模板
- **WHEN** 打开一个 access.log 文件
- **THEN** LogList 应显示动态列：IP、User、Request、Status、Size、Referer、User_Agent
- **AND** FilterPanel 应显示这些字段作为可筛选选项

### Requirement: Extra 字段后端索引

系统应为模板定义的 extra 字段创建索引，支持高效过滤。

#### Scenario: 按 IP 地址过滤
- **GIVEN** 使用 Nginx Access Log 模板解析的日志
- **WHEN** 用户在 FilterPanel 中添加条件 "ip equals 192.168.1.1"
- **THEN** 系统应使用后端索引快速过滤
- **AND** 过滤结果应在 100ms 内返回（对于 10 万条日志）

### Requirement: 模板优先级

系统应支持模板优先级，控制模板匹配顺序。

#### Scenario: 多模板匹配
- **GIVEN** 存在多个可能匹配的模板
- **WHEN** 用户设置了不同的优先级
- **THEN** 系统应按优先级从高到低尝试匹配

## MODIFIED Requirements

### Requirement: FilterPanel 显示

**原需求**：FilterPanel 仅在有筛选条件或 extraFields 非空时显示

**新需求**：FilterPanel 始终显示，提供统一的筛选入口。当无 extra 字段时，仅显示核心字段筛选选项。

### Requirement: LogList 动态列

**原需求**：LogList 根据 extraFields prop 显示动态列

**新需求**：LogList 自动从 templateStore 获取当前模板的 extra_fields，并显示对应的动态列。列顺序应与模板定义一致。

## REMOVED Requirements

### Requirement: field_mapping 字段使用

**Reason**: `field_mapping` 字段从未实现，应使用 `extra_fields` 替代

**Migration**: 删除 `templateStore.ts` 中对 `field_mapping` 的引用，改用 `extra_fields`
