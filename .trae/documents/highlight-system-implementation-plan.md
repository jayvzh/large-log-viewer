# 高亮系统落地可行性评估与实施方案

## 一、当前数据存储格式分析

### 1.1 LogEntry（存储在 sled 中）

```rust
pub struct LogEntry {
    pub id: u64,
    pub file_id: u64,
    pub line_number: u64,
    pub timestamp: i64,
    pub level: LogLevel,
    pub source: SmallVec<[u8; 64]>,
    pub message: SmallVec<[u8; 128]>,
    pub raw_offset: u64,      // 原始文件偏移量
    pub raw_length: u32,      // 原始行长度
    pub extra: HashMap<String, String>,
    pub is_parsed: bool,
}
```

### 1.2 LogEntryView（发送给前端）

```rust
pub struct LogEntryView {
    pub id: u64,
    pub timestamp: i64,
    pub level: String,
    pub source: String,
    pub message: String,
    pub raw: String,          // 原始行内容（按需加载）
    pub extra: HashMap<String, String>,
    pub is_parsed: bool,
}
```

### 1.3 LogTemplate（当前模板结构）

```rust
pub struct LogTemplate {
    pub name: String,
    pub pattern: String,
    pub field_mapping: HashMap<String, String>,
    pub is_builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub extra_fields: Vec<String>,
    pub has_level: bool,
    pub has_timestamp: bool,
    pub has_source: bool,
    pub priority: u32,
}
```

***

## 二、可行性评估

### 2.1 ✅ field 高亮 - **完全兼容**

* LogEntry 已存储 `level`、`source`、`message`、`extra` 字段

* 无需额外处理，直接基于字段值匹配高亮规则

* 性能最优，O(1) 查找

### 2.2 ✅ token 高亮 - **兼容，需扩展**

**现状问题**：

* LogEntryView 的 `raw` 字段默认为空（`From<&LogEntry>` 实现中 `raw: String::new()`）

* 需要 `raw_offset` 和 `raw_length` 从原始文件读取

**解决方案**：

1. 方案A：在解析时同时存储 raw 内容（增加内存占用）
2. 方案B：按需读取 raw 内容（当前实现方式，见 `get_log_entry` 命令）
3. 方案C：在 Rust 端处理高亮，生成 spans 后发送前端

**推荐方案C**：在 Rust 端处理所有高亮逻辑，生成 `HighlightSpan[]` 发送给前端

### 2.3 ✅ keyword 高亮 - **兼容**

* 使用 Aho-Corasick 算法（Rust 的 `aho-corasick` crate）

* O(n) 扫描，性能优秀

* 需要访问 raw 或 message 内容

### 2.4 ⚠️ regex 高亮 - **兼容但需限制**

* 需要编译正则表达式，有性能开销

* 必须限制数量（建议 ≤ 3 条）

* 需要访问 raw 内容

***

## 三、推荐架构设计

### 3.1 三层分离架构

```
┌─────────────────────────────────────────────────────────────┐
│                     用户选择层                               │
│  current_template: "nginx_access"                          │
│  current_highlight: "nginx_default"  // 可独立切换          │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   日志模板 (Parsing)     │     │   高亮模板 (Highlight)   │
│                         │     │                         │
│  {                      │     │  {                      │
│    "name": "nginx",     │     │    "name": "nginx_def", │
│    "pattern": "...",    │     │    "rules": [           │
│    "mapping": {...},    │     │      { "type": "field", │
│    "default_highlight": │     │        "field": "level",│
│      "nginx_default"    │     │        "style": {...} } │
│  }                      │     │    ]                    │
└─────────────────────────┘     └─────────────────────────┘
```

### 3.2 数据结构设计

#### 3.2.1 高亮模板 (HighlightProfile)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightProfile {
    pub name: String,
    pub rules: Vec<HighlightRule>,
    pub is_builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HighlightRule {
    Field {
        field: String,
        style: FieldStyle,
    },
    Token {
        pattern: String,
        style: StyleDef,
    },
    Keyword {
        words: Vec<String>,
        style: StyleDef,
    },
    Regex {
        pattern: String,
        style: StyleDef,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldStyle {
    pub map: HashMap<String, String>,  // value -> style
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDef {
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}
```

#### 3.2.2 高亮输出 (HighlightSpan)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub class: String,  // CSS class name
}
```

#### 3.2.3 扩展 LogEntryView

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryView {
    pub id: u64,
    pub timestamp: i64,
    pub level: String,
    pub source: String,
    pub message: String,
    pub raw: String,
    pub extra: HashMap<String, String>,
    pub is_parsed: bool,
    // 新增高亮 spans
    #[serde(default)]
    pub highlight_spans: Vec<HighlightSpan>,
}
```

### 3.3 用户选择存储

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSelection {
    pub current_template: Option<String>,
    pub current_highlight: Option<String>,
}
```

***

## 四、UI 设计方案

### 4.1 ControlBar 扩展

```
┌──────────────────────────────────────────────────────────────────┐
│ [模板下拉框 ▼] [高亮下拉框 ▼] | 筛选 | 搜索 | 导出              │
└──────────────────────────────────────────────────────────────────┘
```

### 4.2 模板管理页面改造

```
┌──────────────────────────────────────────────────────────────────┐
│  [日志模板] [高亮模板]  ← 新增标签页切换                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  日志模板页面（保持现有功能）                                      │
│                                                                  │
│  高亮模板页面（新增）：                                            │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 内置高亮模板                                                │  │
│  │  • 通用日志高亮                                             │  │
│  │  • Web 日志高亮                                             │  │
│  │  • 安全日志高亮                                             │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ 用户高亮模板                                                │  │
│  │  • nginx_default                      [编辑] [删除] [导出] │  │
│  │  • custom_debug                       [编辑] [删除] [导出] │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  [+ 新建] [导入]                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 4.3 高亮模板编辑器

```
┌──────────────────────────────────────────────────────────────────┐
│  编辑高亮模板                                           [×]     │
├──────────────────────────────────────────────────────────────────┤
│  名称: [nginx_default         ]                                  │
│                                                                  │
│  高亮规则:                                                        │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 类型: [field ▼]  字段: [level ▼]                           │  │
│  │ 样式映射:                                                   │  │
│  │   ERROR → [red ▼] [✓] 粗体                                 │  │
│  │   WARN  → [yellow ▼] [ ] 粗体                              │  │
│  │   INFO  → [blue ▼] [ ] 粗体                                │  │
│  │   [+ 添加映射]                                              │  │
│  │                                          [删除规则]         │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ 类型: [keyword ▼]  关键词: [failed, error, exception]      │  │
│  │ 样式: [red ▼] [✓] 粗体 [ ] 斜体 [ ] 下划线                 │  │
│  │                                          [删除规则]         │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ [+ 添加规则]                                                │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  实时预览:                                                        │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 2024-01-15 10:30:45 [ERROR] Connection failed to server    │  │
│  │                    ^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^         │  │
│  │                    红色      红色粗体                        │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│                              [取消]  [保存]                       │
└──────────────────────────────────────────────────────────────────┘
```

***

## 五、预设高亮模板

### 5.1 通用日志高亮 (general\_default)

```json
{
  "name": "general_default",
  "rules": [
    {
      "type": "field",
      "field": "level",
      "style": {
        "map": {
          "FATAL": "red bold",
          "ERROR": "red",
          "WARN": "yellow",
          "INFO": "blue",
          "DEBUG": "gray",
          "TRACE": "gray"
        }
      }
    },
    {
      "type": "keyword",
      "words": ["failed", "error", "exception", "timeout", "refused"],
      "style": "red"
    },
    {
      "type": "keyword",
      "words": ["success", "ok", "completed", "started"],
      "style": "green"
    }
  ]
}
```

### 5.2 Web 日志高亮 (web\_default)

```json
{
  "name": "web_default",
  "rules": [
    {
      "type": "field",
      "field": "level",
      "style": {
        "map": {
          "FATAL": "red bold",
          "ERROR": "red",
          "WARN": "yellow",
          "INFO": "blue"
        }
      }
    },
    {
      "type": "field",
      "field": "status",
      "style": {
        "map": {
          "200": "green",
          "201": "green",
          "301": "cyan",
          "302": "cyan",
          "400": "yellow",
          "404": "yellow",
          "500": "red bold",
          "502": "red bold",
          "503": "red bold"
        }
      }
    },
    {
      "type": "regex",
      "pattern": "\\b\\d{1,3}(\\.\\d{1,3}){3}\\b",
      "style": "cyan"
    }
  ]
}
```

### 5.3 安全日志高亮 (security\_default)

```json
{
  "name": "security_default",
  "rules": [
    {
      "type": "keyword",
      "words": ["failed", "invalid", "unauthorized", "denied", "blocked"],
      "style": "red bold"
    },
    {
      "type": "keyword",
      "words": ["login", "authentication", "password", "session"],
      "style": "yellow"
    },
    {
      "type": "regex",
      "pattern": "\\b\\d{1,3}(\\.\\d{1,3}){3}\\b",
      "style": "cyan"
    }
  ]
}
```

***

## 六、实施步骤

### 阶段一：后端基础架构（预计 2-3 天）

1. **新增数据模型**

   * [ ] 创建 `HighlightProfile` 结构体

   * [ ] 创建 `HighlightRule` 枚举

   * [ ] 创建 `HighlightSpan` 结构体

   * [ ] 扩展 `LogEntryView` 添加 `highlight_spans` 字段

2. **新增存储管理**

   * [ ] 创建 `HighlightStore` 管理高亮模板持久化

   * [ ] 存储路径：`{用户数据目录}/LogViewer/highlights.json`

3. **新增 Tauri 命令**

   * [ ] `get_highlight_profiles()` - 获取所有高亮模板

   * [ ] `create_highlight_profile()` - 创建高亮模板

   * [ ] `update_highlight_profile()` - 更新高亮模板

   * [ ] `delete_highlight_profile()` - 删除高亮模板

   * [ ] `export_highlight_profiles()` - 导出高亮模板

   * [ ] `import_highlight_profiles()` - 导入高亮模板

### 阶段二：高亮引擎实现（预计 2-3 天）

1. **高亮处理引擎**

   * [ ] 实现 `HighlightEngine` 结构体

   * [ ] 实现 field 规则处理（直接字段匹配）

   * [ ] 实现 keyword 规则处理（Aho-Corasick）

   * [ ] 实现 token 规则处理（正则匹配）

   * [ ] 实现 regex 规则处理（限制数量）

   * [ ] 规则优先级处理：field > keyword > token > regex

2. **集成到日志查询流程**

   * [ ] 修改 `get_log_entries` 命令，应用高亮规则

   * [ ] 生成 `HighlightSpan` 数组

   * [ ] 添加高亮结果缓存

### 阶段三：前端 UI 实现（预计 3-4 天）

1. **模板管理页面改造**

   * [ ] 添加标签页切换组件

   * [ ] 创建 `HighlightManagerModal.svelte` 组件

   * [ ] 创建 `HighlightEditorModal.svelte` 组件

   * [ ] 实现高亮规则编辑 UI

   * [ ] 实现实时预览功能

2. **ControlBar 扩展**

   * [ ] 添加高亮模板下拉框

   * [ ] 实现模板和高亮的独立切换

3. **前端 Store 扩展**

   * [ ] 扩展 `templateStore` 添加高亮模板状态

   * [ ] 或创建独立的 `highlightStore`

4. **日志渲染改造**

   * [ ] 修改 `LogList.svelte` 支持高亮渲染

   * [ ] 实现 CSS 类映射

### 阶段四：集成与优化（预计 1-2 天）

1. **日志模板关联**

   * [ ] 扩展 `LogTemplate` 添加 `default_highlight` 字段

   * [ ] 实现模板创建时选择关联高亮模板

2. **预设模板**

   * [ ] 添加内置高亮模板

   * [ ] 关联内置日志模板与高亮模板

3. **性能优化**

   * [ ] 高亮结果缓存

   * [ ] 虚拟滚动优化

   * [ ] 限制 regex 规则数量

***

## 七、技术要点

### 7.1 性能优化策略

1. **规则优先级**：field > keyword > token > regex
2. **缓存策略**：每行解析一次，高亮结果缓存
3. **限制 regex 数量**：≤ 3 条
4. **Aho-Corasick 算法**：keyword 匹配 O(n)

### 7.2 CSS 类映射

```css
.hl-red { color: #f44747; }
.hl-green { color: #4ec9b0; }
.hl-blue { color: #569cd6; }
.hl-yellow { color: #dcdcaa; }
.hl-cyan { color: #4fc1ff; }
.hl-purple { color: #c586c0; }
.hl-gray { color: #808080; }
.hl-bold { font-weight: bold; }
.hl-italic { font-style: italic; }
.hl-underline { text-decoration: underline; }
```

### 7.3 前端渲染

```svelte
{#each highlightSpans as span}
  <span class={span.class}>
    {message.slice(span.start, span.end)}
  </span>
{/each}
```

***

## 八、总结

### 8.1 可行性结论

| 功能         | 可行性    | 说明              |
| ---------- | ------ | --------------- |
| field 高亮   | ✅ 完全兼容 | 直接使用现有字段        |
| token 高亮   | ✅ 兼容   | 需访问 raw 内容      |
| keyword 高亮 | ✅ 兼容   | 使用 Aho-Corasick |
| regex 高亮   | ⚠️ 需限制 | 限制数量 ≤ 3        |

### 8.2 架构优势

1. **三层分离**：日志模板、高亮模板、用户选择独立管理
2. **灵活切换**：高亮模板可独立于日志模板切换
3. **易于扩展**：新增规则类型只需扩展枚举
4. **性能优先**：Rust 端处理，前端纯渲染

### 8.3 风险与缓解

| 风险         | 缓解措施          |
| ---------- | ------------- |
| regex 性能问题 | 限制数量，编译缓存     |
| 内存占用增加     | 高亮结果按需计算，缓存有限 |
| UI 复杂度增加   | 分阶段实施，优先核心功能  |

