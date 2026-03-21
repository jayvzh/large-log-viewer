# LogViewer - 高性能日志查看器

一款专为处理大型日志文件设计的高性能桌面应用，支持几百MB至几GB的日志文件快速加载、分类、搜索和查看。

## 功能特性

- **大文件支持**：使用内存映射文件（mmap）零拷贝技术，轻松处理GB级日志文件
- **快速解析**：多线程并行解析，1GB文件解析时间 < 15秒
- **智能分类**：自动识别日志级别（FATAL/ERROR/WARN/INFO/DEBUG/TRACE），支持快速过滤
- **高效搜索**：基于布隆过滤器和位图索引的搜索引擎，响应时间 < 100ms
- **虚拟滚动**：支持千万级日志条目的流畅滚动和快速跳转
- **多格式查看**：支持文本、HTML、JSON、XML 四种查看模式
- **时间筛选**：按时间范围快速筛选日志
- **拖拽加载**：支持拖拽文件到窗口直接打开

## 技术栈

### 后端 (Rust)
- **桌面框架**: Tauri 2.x
- **数据库**: sled 嵌入式数据库
- **文件读取**: memmap2 内存映射
- **索引引擎**: RoaringBitmap 位图索引
- **并行处理**: rayon + tokio
- **日志解析**: regex 多模式匹配

### 前端 (TypeScript)
- **框架**: Svelte 5 + SvelteKit
- **语言**: TypeScript
- **构建工具**: Vite
- **状态管理**: Svelte Stores

## 性能指标

| 指标 | 目标值 |
|------|--------|
| 1GB 文件加载时间 | < 15 秒 |
| 内存占用 | < 文件大小的 0.8 倍 |
| 筛选/搜索响应时间 | < 100ms |
| 最大支持日志条目 | 1000 万条 |
| 跳转响应（1→100万条） | < 50ms |
| 打包体积 | < 10MB |

## 系统要求

- Windows 10/11 (x64)
- 无需额外运行时依赖

## 安装使用

### 从发布包安装

下载最新的 `.exe` 安装包，双击安装即可。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-repo/large-log-viewer.git
cd large-log-viewer/log-viewer

# 安装依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建发布版本
pnpm tauri build
```

## 项目结构

```
log-viewer/
├── src/                      # Svelte 前端源码
│   ├── routes/
│   │   └── +page.svelte      # 主页面
│   ├── lib/
│   │   ├── components/
│   │   │   ├── LogList.svelte    # 虚拟滚动日志列表
│   │   │   └── LogDetail.svelte  # 多格式详情查看器
│   │   ├── stores/
│   │   │   └── logStore.ts       # 状态管理
│   │   └── api/
│   │       └── tauri.ts          # Tauri API 封装
│   └── app.html             # HTML 模板
├── src-tauri/               # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs           # 库入口
│   │   ├── main.rs          # 应用入口
│   │   ├── models/          # 数据模型
│   │   ├── database/        # sled 数据库操作
│   │   ├── parser/          # 日志解析器
│   │   ├── reader/          # 文件读取器
│   │   └── commands/        # Tauri 命令
│   ├── Cargo.toml           # Rust 依赖
│   └── tauri.conf.json      # Tauri 配置
└── package.json             # Node.js 依赖
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `F5` | 刷新日志 |
| `Ctrl+O` | 打开文件 |
| `Ctrl+F` | 聚焦搜索框 |
| `Escape` | 重置筛选条件 |
| `Ctrl+C` | 复制选中日志内容 |

## 支持的日志格式

应用自动识别以下常见日志格式：

1. **标准格式**: `2024-01-15 10:30:45.123 INFO  [module] message`
2. **Log4j格式**: `2024-01-15 10:30:45,123 INFO  [module] message`
3. **ISO格式**: `2024-01-15T10:30:45.123Z INFO module - message`
4. **紧凑格式**: `2024-01-15 10:30:45 INFO module: message`

## 日志级别颜色

| 级别 | 颜色 |
|------|------|
| FATAL | #f44747 (红色) |
| ERROR | #f44747 (红色) |
| WARN | #d7ba7d (黄色) |
| INFO | #569cd6 (蓝色) |
| DEBUG | #808080 (灰色) |
| TRACE | #606060 (深灰) |
| OTHER | #cccccc (浅灰) |

## 开发

### 环境要求

- Node.js 18+
- pnpm 8+
- Rust 1.70+
- Visual Studio Build Tools (Windows)

### 运行测试

```bash
# Rust 后端测试
cd src-tauri
cargo test

# 前端类型检查
pnpm check
```

## 许可证

MIT License

## 致谢

本项目使用以下开源技术构建：
- [Tauri](https://tauri.app/) - 构建更小、更快、更安全的桌面应用
- [Svelte](https://svelte.dev/) - 编译时优化的前端框架
- [sled](https://github.com/spacejam/sled) - 高性能嵌入式数据库
- [RoaringBitmap](https://roaringbitmap.org/) - 高效压缩位图
