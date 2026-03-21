# LogViewer 待办事项（已完成，等待新任务）

本文档记录 README.md "当前实现差距分析" 中未被本次 UI 改进方案覆盖的待办项目。

***

## 待办事项清单

### 第一优先级：核心性能优化（严重）

#### 1. 启用并行解析

**问题描述**：README 声称使用 rayon 多线程并行解析，但实际代码中 `parse_lines_parallel` 函数从未被调用。

**代码位置**：

- [parser/mod.rs:149-157](src-tauri/src/parser/mod.rs#L149-L157) - 定义了并行解析函数
- [commands/mod.rs:100-133](src-tauri/src/commands/mod.rs#L100-L133) - 实际使用单线程循环

**解决方案**：

```rust
// 方案A：批量收集后并行解析
let lines: Vec<_> = reader.read_lines_mmap()?.collect();
let chunks: Vec<_> = lines.chunks(10000).collect();

let results: Vec<Vec<LogEntry>> = chunks.par_iter()
    .map(|chunk| parser.parse_lines_parallel(chunk))
    .collect();

// 方案B：流式并行处理（更优）
use crossbeam_channel::{bounded, Sender, Receiver};

let (sender, receiver) = bounded(10000);
thread::spawn(move || {
    for line in reader.read_lines_mmap()? {
        sender.send(line)?;
    }
});

let entries: Vec<LogEntry> = receiver.par_iter()
    .map(|line| parser.parse_line(...))
    .collect();
```

**预期效果**：4 核 CPU 解析速度提升 3-3.5 倍

***

#### 2. 修复零拷贝实现

**问题描述**：`MmapLineIterator` 在迭代时调用 `to_vec()`，破坏了零拷贝设计。

**代码位置**：[reader/mod.rs:156](src-tauri/src/reader/mod.rs#L156)

**当前代码**：

```rust
let line = mmap[start..end].to_vec();  // ❌ 拷贝到堆内存
Some(MmapLine {
    data: line,
    // ...
})
```

**解决方案**：

```rust
pub struct MmapLine<'a> {
    pub data: &'a [u8],  // ✅ 零拷贝
    pub line_number: u64,
    pub offset: u64,
}

impl<'a> Iterator for MmapLineIterator<'a> {
    type Item = MmapLine<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mmap = self.mmap.as_ref()?;
        // ...
        Some(MmapLine {
            data: &mmap[start..end],  // ✅ 零拷贝
            line_number,
            offset,
        })
    }
}
```

**预期效果**：内存占用降低 40-60%

***

#### 3. 优化数据库查询

**问题描述**：`get_entries` 需要全表扫描才能获取指定偏移量的数据。

**代码位置**：[database/mod.rs:91-114](src-tauri/src/database/mod.rs#L91-L114)

**当前代码**：

```rust
for item in iter {
    if count >= offset + limit {  // ❌ 跳转到第100万条需遍历100万次
        break;
    }
    // ...
}
```

**解决方案**：

```rust
// 添加二级索引
// 主键: log:{file_id}:{entry_id}
// 索引: log_idx:{file_id}:{entry_id:016x} → 空值（用于快速定位）

pub async fn get_entries_fast(&self, file_id: u64, offset: u64, limit: u64) -> Result<Vec<LogEntry>, String> {
    let start_key = format!("log:{}:{:016x}", file_id, offset);
    let iter = db.range(start_key.as_bytes()..);
    // 直接从 offset 开始扫描，无需遍历前面的记录
}
```

**预期效果**：跳转到任意位置响应时间 < 50ms

***

#### 4. 实现全文搜索索引

**问题描述**：搜索只处理前 10000 条记录，且无索引支持。

**代码位置**：[commands/mod.rs:224-246](src-tauri/src/commands/mod.rs#L224-L246)

**当前代码**：

```rust
let entries = state.db.get_entries(file_id, 0, 10000).await?;  // ❌ 只搜索前10000条
let filtered: Vec<_> = entries.iter().filter(|e| {
    e.logger_str().to_lowercase().contains(&query_lower)  // ❌ 内存中线性扫描
}).collect();
```

**解决方案**：

```rust
// 倒排索引结构
// word:{file_id}:{word_hash} → RoaringBitmap (包含该词的日志ID)

pub async fn build_search_index(&self, file_id: u64, entries: &[LogEntry]) -> Result<(), String> {
    let mut word_index: HashMap<String, RoaringBitmap> = HashMap::new();
    
    for entry in entries {
        let words = tokenize(&entry.message);
        for word in words {
            word_index.entry(word)
                .or_default()
                .insert(entry.id as u32);
        }
    }
    
    for (word, bitmap) in word_index {
        let key = format!("word:{}:{}", file_id, hash(&word));
        self.db.insert(key, bitmap.serialize())?;
    }
    
    Ok(())
}
```

**预期效果**：百万级日志搜索响应 < 100ms

***

### 第二优先级：功能增强（中等）

#### 5. 添加时间戳索引

**问题描述**：时间过滤在前端实现，应该在后端利用时间戳索引。

**代码位置**：[logStore.ts:194-217](src/lib/stores/logStore.ts#L194-L217)

**解决方案**：

```rust
// 时间戳索引
// time_idx:{file_id}:{timestamp} → RoaringBitmap

pub async fn filter_by_time(&self, file_id: u64, start: i64, end: i64) -> Result<Vec<u64>, String> {
    let start_key = format!("time_idx:{}:{:016x}", file_id, start);
    let end_key = format!("time_idx:{}:{:016x}", file_id, end);
    
    let mut result = RoaringBitmap::new();
    let iter = self.db.range(start_key.as_bytes()..end_key.as_bytes());
    
    for item in iter {
        let (_, bitmap_data) = item?;
        let bitmap = RoaringBitmap::deserialize(&bitmap_data)?;
        result |= bitmap;
    }
    
    Ok(result.iter().map(|id| id as u64).collect())
}
```

***

#### 6. 实现真正的分页加载

**问题描述**：前端一次性加载 10000 条日志到内存。

**代码位置**：[logStore.ts:135-138](src/lib/stores/logStore.ts#L135-L138)

**当前代码**：

```typescript
const entries = await getEntries(fileId, 0, 10000);  // ❌ 一次性加载
this.logs = entries;
```

**解决方案**：

```typescript
class LogStore {
    private pageSize = 100;
    private loadedPages = new Map<number, LogEntry[]>();
    
    async loadPage(page: number): Promise<LogEntry[]> {
        if (this.loadedPages.has(page)) {
            return this.loadedPages.get(page)!;
        }
        
        const offset = page * this.pageSize;
        const entries = await getEntries(this.currentFile!.id, offset, this.pageSize);
        this.loadedPages.set(page, entries);
        return entries;
    }
    
    async onScroll(scrollTop: number, viewportHeight: number) {
        const startPage = Math.floor(scrollTop / (this.pageSize * ROW_HEIGHT));
        const endPage = Math.ceil((scrollTop + viewportHeight) / (this.pageSize * ROW_HEIGHT));
        
        for (let page = startPage; page <= endPage; page++) {
            if (!this.loadedPages.has(page)) {
                await this.loadPage(page);
            }
        }
    }
}
```

***

#### 7. 后端过滤替代前端过滤

**问题描述**：过滤逻辑在前端，应该移至后端减少数据传输。

**解决方案**：

```rust
#[tauri::command]
pub async fn filter_logs(
    file_id: u64,
    level: Option<String>,
    query: Option<String>,
    time_range: Option<(i64, i64)>,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let mut result_bitmap = get_all_entry_ids(file_id)?;
    
    // 级别过滤（利用 RoaringBitmap）
    if let Some(l) = level {
        let level_bitmap = state.db.get_level_bitmap(file_id, LogLevel::from_str(&l)).await?;
        result_bitmap &= level_bitmap.unwrap_or_default();
    }
    
    // 时间过滤（利用时间索引）
    if let Some((start, end)) = time_range {
        let time_bitmap = state.db.get_time_range_bitmap(file_id, start, end).await?;
        result_bitmap &= time_bitmap;
    }
    
    // 搜索过滤（利用倒排索引）
    if let Some(q) = query {
        let search_bitmap = state.db.search_with_index(file_id, &q).await?;
        result_bitmap &= search_bitmap;
    }
    
    let ids: Vec<u64> = result_bitmap.iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|id| id as u64)
        .collect();
    
    let entries = state.db.get_entries_by_ids(file_id, &ids).await?;
    Ok(entries.iter().map(LogEntryView::from).collect())
}
```

***

### 第三优先级：健壮性增强（轻微）

#### 8. 添加文件大小检查

**问题描述**：无大文件保护机制。

**解决方案**：

```rust
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10GB

pub async fn open_file(path: String, state: State<'_, AppState>) -> Result<FileInfo, String> {
    let metadata = std::fs::metadata(&path)?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} bytes (max: {} bytes)",
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }
    
    // ...
}
```

***

## 优先级排序

```
┌─────────────────────────────────────────────────────────────────┐
│                     待办事项优先级                               │
├─────────────────────────────────────────────────────────────────┤
│ ████████████████████████ 1. 启用并行解析 (严重)                 │
│ ████████████████████████ 2. 修复零拷贝实现 (严重)               │
│ ████████████████████████ 3. 优化数据库查询 (严重)               │
│ ████████████████████████ 4. 实现全文搜索索引 (严重)             │
│ ██████████████░░░░░░░░░░ 5. 添加时间戳索引 (中等)               │
│ ██████████████░░░░░░░░░░ 6. 实现分页加载 (中等)                 │
│ ██████████████░░░░░░░░░░ 7. 后端过滤替代 (中等)                 │
│ ████████░░░░░░░░░░░░░░░░ 8. 文件大小检查 (轻微)                 │
└─────────────────────────────────────────────────────────────────┘
```

***

## 预期性能提升

| 指标          | 当前实现        | 改进后     | 提升幅度  |
| ----------- | ----------- | ------- | ----- |
| 1GB 文件解析时间  | \~45s       | \~12s   | 3.75x |
| 内存占用（1GB文件） | \~1.5GB     | \~600MB | 60%↓  |
| 跳转到第100万条   | \~3s        | <50ms   | 60x   |
| 全文搜索响应      | N/A（只搜前1万条） | <100ms  | ∞     |
| 级别过滤响应      | \~500ms     | <10ms   | 50x   |

***

## 建议实施顺序

1. **第一阶段**（预计 5-8 天）
   - 启用并行解析
   - 修复零拷贝实现
   - 优化数据库查询
2. **第二阶段**（预计 4-6 天）
   - 实现全文搜索索引
   - 添加时间戳索引
3. **第三阶段**（预计 3-5 天）
   - 实现分页加载
   - 后端过滤替代
   - 文件大小检查

**预计总工期**：12-19 天
