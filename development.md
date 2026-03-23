# 开发文档

本文档包含 LogViewer 的技术架构、模板系统详解、开发进度和待改进事项。

---

## 一、技术架构

### 1.1 技术栈

| 层级 | 技术 |
|------|------|
| **桌面框架** | Tauri 2.x（Rust + WebView） |
| **前端框架** | Svelte 5 + SvelteKit |
| **构建工具** | Vite + pnpm |
| **数据库** | sled 嵌入式 KV 数据库 |
| **索引引擎** | RoaringBitmap 压缩位图 |
| **文件读取** | memmap2 内存映射 |
| **并行处理** | rayon + tokio |
| **日志解析** | regex 多模式匹配 |

### 1.2 项目结构

```
large-log-viewer/
├── src/                          # Svelte 前端源码
│   ├── routes/
│   │   └── +page.svelte          # 主页面
│   ├── lib/
│   │   ├── components/           # UI 组件
│   │   │   ├── FileBar.svelte    # 工具栏
│   │   │   ├── LogList.svelte    # 虚拟滚动日志列表
│   │   │   ├── LogDetail.svelte  # 多格式详情查看器
│   │   │   ├── TemplateManagerModal.svelte  # 模板管理器
│   │   │   ├── TemplateEditorModal.svelte   # 模板编辑器
│   │   │   └── ...
│   │   ├── stores/               # 状态管理
│   │   │   ├── templateStore.ts  # 模板状态管理
│   │   │   └── ...
│   │   └── api/                  # Tauri API 封装
│   └── app.html                  # HTML 模板
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── main.rs               # 应用入口
│   │   ├── models/               # 数据模型
│   │   ├── database/             # sled 数据库操作
│   │   ├── parser/               # 日志解析器
│   │   │   ├── template.rs       # 模板解析器核心
│   │   │   └── ...
│   │   ├── template_store.rs     # 模板持久化存储
│   │   ├── reader/               # 文件读取器
│   │   └── commands/             # Tauri 命令
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
└── package.json                  # Node.js 依赖
```

### 1.3 数据存储

| 数据类型 | 存储位置 | 格式 |
|---------|---------|------|
| **模板定义** | 文件系统 `data/templates.json` + 数据库 `template:{name}` | JSON 序列化 |
| **日志条目** | Sled 数据库 `logs.db` | 二进制序列化 |
| **级别索引** | Sled 数据库 `level_bitmap:{file_id}:{level}` | RoaringBitmap |
| **时间索引** | Sled 数据库 `time_index:{file_id}` | BTreeMap |
| **搜索索引** | Sled 数据库 `search_index:{file_id}:{word_hash}` | RoaringBitmap |
| **Extra 字段索引** | Sled 数据库 `extra_idx:{file_id}:{field_name}:{value_hash}` | RoaringBitmap |

---

## 二、模板系统详解

### 2.1 架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         数据流向图                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   原始日志行 ──→ LogTemplateParser.parse_line() ──→ LogEvent        │
│       │                    │                           │             │
│       │                    ▼                           ▼             │
│       │           正则匹配捕获组              字段分类存储           │
│       │                    │                           │             │
│       │                    ▼                           ▼             │
│       │           captures_to_event()         ┌───────┴───────┐     │
│       │                    │                  │               │     │
│       │                    │             核心字段          Extra    │
│       │                    │           (timestamp,        HashMap   │
│       │                    │            level,                      │
│       │                    │            source,                     │
│       │                    │            message)                    │
│       │                    │                  │               │     │
│       ▼                    ▼                  ▼               ▼     │
│   LogEntry ◄───────────────────────────────────────────────────     │
│       │                                                              │
│       ▼                                                              │
│   Sled 数据库持久化                                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 模板数据结构

文件: `src-tauri/src/models/mod.rs`

```rust
pub struct LogTemplate {
    pub name: String,                    // 模板名称（唯一标识）
    pub pattern: String,                 // 正则表达式模式
    pub field_mapping: HashMap<String, String>,  // 字段映射（预留，未使用）
    pub is_builtin: bool,                // 是否内置模板（不可修改/删除）
    pub created_at: i64,                 // 创建时间戳
    pub updated_at: i64,                 // 更新时间戳
    pub extra_fields: Vec<String>,       // 预提取的额外字段名列表
    pub has_level: bool,                 // 是否包含级别字段
    pub has_timestamp: bool,             // 是否包含时间戳字段
    pub has_source: bool,                // 是否包含来源字段
}
```

### 2.3 模板语法

模板使用 **Rust regex crate** 的命名捕获组语法：

```regex
(?P<字段名>正则表达式)
```

**示例**：
```regex
^(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})\s+(?P<level>\w+)\s+(?P<message>.+)$
```

### 2.4 核心字段名及别名映射

| 主字段名 | 支持的别名 | 用途 |
|---------|-----------|------|
| `timestamp` | `time`, `ts` | 日志时间戳 |
| `level` | `lvl`, `severity` | 日志级别 |
| `source` | `logger`, `src` | 日志来源 |
| `message` | `msg`, `content`, `summary` | 日志消息内容 |

**其他任何字段名** → 自动存入 `extra` HashMap

### 2.5 解析流程

1. **解析入口**: `parse_log` 命令接收文件ID和可选模板名
2. **核心解析**: `LogTemplateParser.parse_line()` 遍历所有编译好的模板，按优先级顺序尝试匹配
3. **字段提取**: `captures_to_event()` 将捕获组分类为核心字段和 extra 字段
4. **数据存储**: LogEvent 转换为 LogEntry 后存入 Sled 数据库

### 2.6 内置模板

| 模板名 | 用途 | Extra 字段 |
|-------|------|-----------|
| Linux Syslog | 系统日志 | `hostname` |
| Linux Auth Log | 认证日志 | `hostname` |
| Nginx Access Log | Nginx访问日志 | `ip`, `user`, `request`, `status`, `size`, `referer`, `user_agent` |

### 2.7 自定义模板示例

**Java Log4j 格式**：
```regex
^(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3})\s+(?P<level>\w+)\s+\[(?P<source>[^\]]+)\]\s+(?P<message>.+)$
```

**匹配示例**：
```
2024-01-15 10:30:45,123 INFO  [com.example.Service] User login success
```

**提取结果**：
- `timestamp`: 2024-01-15 10:30:45,123
- `level`: INFO
- `source`: com.example.Service
- `message`: User login success

### 2.8 模板自动检测

打开日志文件时，系统会：
1. 读取文件前 100 行
2. 遍历所有模板计算匹配率
3. 选择匹配率 > 50% 且最高的模板
4. 自动应用该模板解析整个文件

### 2.9 创建自定义模板

1. 点击工具栏的「模板管理」按钮
2. 点击「新建模板」
3. 输入模板名称和正则表达式
4. 在测试区域粘贴示例日志行验证匹配效果
5. 设置优先级（数值越大优先级越高）
6. 保存后即可使用

### 2.10 AI 辅助生成

模板编辑器提供 AI 辅助功能，点击「AI 辅助」按钮可复制提示词模板，粘贴到 AI 对话框中并附上日志示例，即可快速生成正则表达式。

### 2.11 当前方案局限性

| 类别 | 缺点 | 影响 |
|-----|------|------|
| 架构 | 单模板匹配（只使用第一个匹配成功的模板） | 混合格式日志后续模板不生效 |
| 架构 | field_mapping 预留字段从未实现 | 功能不完整 |
| 性能 | 顺序匹配遍历所有模板直到成功 | 模板多时性能下降 |
| 功能 | 无嵌套字段支持 | 无法解析结构化日志 |
| 功能 | 无类型推断 | 所有字段都是 String |
| 功能 | 无多行日志支持 | Java 堆栈日志等无法正确处理 |
| 体验 | 正则语法门槛高 | 普通用户难以创建模板 |

---

## 三、开发进度

### 3.1 已完成功能

#### 核心功能
- [x] 内存映射文件读取（mmap）
- [x] 零拷贝行迭代器（`MmapLine<'a>` 使用生命周期引用）
- [x] 多线程并行解析（`parse_lines_parallel`）
- [x] UTF-8 BOM 自动跳过
- [x] 多编码支持（UTF-8、UTF-16 LE/BE、ANSI）

#### 索引与存储
- [x] sled 嵌入式数据库存储
- [x] RoaringBitmap 级别索引
- [x] 倒排索引（词频索引）用于搜索
- [x] 时间索引（`time_idx:{file_id}:{timestamp}`）

#### 搜索功能
- [x] 全量日志搜索（使用倒排索引）
- [x] 级别过滤（使用 RoaringBitmap）
- [x] 时间范围过滤（使用时间索引）
- [x] 三种搜索模式（模糊、精确、正则）

#### 前端
- [x] 虚拟滚动
- [x] 分页加载（`loadedPages` Map）
- [x] 编码选择器
- [x] 模板选择器
- [x] 自定义模板支持

#### v0.3.0 新增
- [x] 多实例支持（移除单实例限制，进程ID隔离数据库）
- [x] 模板存储优化（JSON 文件存储，多实例共享）
- [x] 模板检测优化（只读取前 100 行）
- [x] 日志导出功能（支持纯文本和 JSON 格式）
- [x] 移除调试日志（清理 `eprintln!` 输出）
- [x] 时间过滤确认（正确使用后端时间索引）

#### v0.4.0 新增
- [x] Extra 字段索引系统（RoaringBitmap 索引，支持高效过滤）
- [x] 模板优先级支持（priority 字段，按优先级匹配）
- [x] 模板解析器缓存（避免重复编译正则）
- [x] FilterPanel 增强（折叠/展开、清除按钮）
- [x] 修复 FilterPanel 不显示问题（使用 extra_fields 替代 field_mapping）
- [x] 修复 LogList 动态列不显示问题
- [x] Extra 字段后端过滤（利用索引高效查询）

### 3.2 待改进事项

#### 性能优化（中等优先级）

- [ ] **数据库查询优化**
  - 当前：`get_entries` 使用 `scan_prefix` 遍历
  - 改进：使用 `range` 查询直接定位到 offset
  - 预期效果：大偏移量跳转更快

- [ ] **前端缓存策略优化**
  - 当前：固定页大小 100，预加载 5 页
  - 改进：根据可用内存动态调整缓存大小
  - 预期效果：更好的内存利用率

#### 功能增强（低优先级）

- [ ] **文件大小限制**
  - 添加最大文件大小检查（如 10GB）
  - 大文件警告提示

- [ ] **进度反馈优化**
  - 当前：每处理一批更新一次
  - 改进：基于时间和数量的双重策略，更平滑的进度显示

#### 代码质量（低优先级）

- [ ] **错误处理改进**
  - 部分错误信息不够友好
  - 添加错误码和国际化支持

- [ ] **单元测试覆盖**
  - 后端核心逻辑测试
  - 前端组件测试

---

## 四、改进方向

### 4.1 短期改进（低成本）

- [x] 实现模板优先级排序（已实现）
- [x] 复用 LogTemplateParser 实例（已实现）
- [x] 修复字段提取逻辑（已实现）

### 4.2 中期改进（中等成本）

- [x] Extra 字段索引（已实现）
- [ ] 多模板匹配模式（支持指定多个模板，每行尝试所有模板取最佳匹配）
- [ ] 类型推断（自动检测数值字段，支持 status >= 400 等过滤）
- [ ] 多行日志支持（添加多行模式配置，支持以时间戳开头的多行合并）

### 4.3 长期改进（高成本）

- [ ] 可视化模板构建器（拖拽式字段选择，自动生成正则）
- [ ] 支持 Grok 模式（兼容 Logstash/Elasticsearch 生态）
- [ ] 结构化日志支持（JSON 解析器，嵌套字段访问）
- [ ] 自定义转换函数（字段值映射、时间格式转换、正则替换）

---

## 五、技术债务

- [ ] 部分组件样式可以提取为公共 CSS
- [ ] 模板编辑器可以增加正则表达式语法高亮

---

## 六、相关文件清单

| 文件路径 | 功能描述 |
|---------|---------|
| `src-tauri/src/parser/template.rs` | 核心模板解析器实现 |
| `src-tauri/src/models/mod.rs` | 数据模型定义 |
| `src-tauri/src/template_store.rs` | 模板持久化存储 |
| `src-tauri/src/commands/mod.rs` | Tauri 命令接口 |
| `src-tauri/src/database/mod.rs` | 数据库存储模板 |
| `src/lib/stores/templateStore.ts` | 前端状态管理 |
| `src/lib/types/template.ts` | 前端类型定义 |
| `src/lib/components/TemplateEditorModal.svelte` | 模板编辑器UI |

---

*最后更新: 2026-03-23*
