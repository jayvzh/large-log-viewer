# LogViewer仿制软件开发文档

## 1. 项目概述

### 1.1 项目背景
开发一个高性能的日志文件分析工具，专门用于处理大型日志文件（几百MB至几GB），提供快速分类、搜索和查看功能，提高开发人员和运维人员的日志分析效率。

### 1.2 核心目标
- 实现高效的超大日志文件加载和解析
- 提供直观的分类标签过滤机制
- 支持模糊搜索和精确时间筛选
- 保持界面简洁且功能完整

## 2. 功能需求规格

### 2.1 核心功能模块

#### 2.1.1 文件加载模块
- 支持拖拽加载日志文件
- 支持文件对话框选择
- 支持加载多种格式日志文件（.log, .txt等）
- 大文件分块读取与渐进式加载
- 加载进度显示

#### 2.1.2 日志解析模块
- 自动识别常见日志格式
- 支持自定义日志格式配置
- 按行解析并提取时间戳、级别、记录者、摘要信息
- 高效内存管理，避免大文件加载时的内存溢出

#### 2.1.3 分类过滤模块
- 按日志级别分类：致命、错误、警告、信息、调试、跟踪、其它
- "所有"标签显示全部日志
- 实时统计各分类数量
- 支持多分类组合筛选

#### 2.1.4 搜索模块
- 模糊搜索（支持正则表达式）
- 按时间范围筛选
- 搜索条件实时生效
- 搜索结果高亮显示

#### 2.1.5 日志查看模块
- 多格式内容查看：文本、浏览器（HTML）、JSON、XML
- 日志详情显示
- 支持内容复制
- 语法高亮（针对不同格式）

#### 2.1.6 刷新与重置
- F5快捷键刷新
- 刷新按钮
- 重置所有筛选条件

### 2.2 性能要求
- 加载1GB日志文件时间不超过30秒
- 内存占用不超过文件大小的1.5倍
- 筛选和搜索操作响应时间小于1秒（对于已加载文件）
- 支持同时打开多个日志文件

## 3. 界面设计规范

### 3.1 整体布局
```
┌─────────────────────────────────────────────────────────┐
│ LogViewer V1.3.1609.1250 By [开发者信息]                 │
├─────────────────────────────────────────────────────────┤
│ 时间 [下拉菜单]     模糊搜索条件...[搜索框]               │
├─────────────────────────────────────────────────────────┤
│ 所有(0) 致命(0) 错误(0) 警告(0) 信息(0) 调试(0) 跟踪(0) │
├─────────────────────────────────────────────────────────┤
│ 日志记录列表                                            │
│ ┌──────┬──────┬──────┬─────────────────────────────┐    │
│ │ 时间 │ 级别 │记录者│ 摘要                        │    │
│ ├──────┼──────┼──────┼─────────────────────────────┤    │
│ │      │      │      │                             │    │
│ │      │      │      │                             │    │
│ └──────┴──────┴──────┴─────────────────────────────┘    │
├─────────────────────────────────────────────────────────┤
│ 日志内容查看                                            │
│ [文本] [浏览器] [JSON] [XML]                           │
│ ┌──────────────────────────────────────────────────┐    │
│ │                                                  │    │
│ │                                                  │    │
│ │                                                  │    │
│ └──────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────┤
│ 请将日志文件拖入窗口即可加载日志，按F5可刷新...         │
│ [刷新] 列表数: 0                                       │
└─────────────────────────────────────────────────────────┘
```

### 3.2 界面组件详细说明

#### 3.2.1 标题栏
- 显示软件名称、版本号和开发者信息
- 固定高度：40px
- 背景色：深灰色(#2d2d30)
- 字体颜色：浅灰色(#cccccc)

#### 3.2.2 控制栏
- 时间下拉菜单：提供时间筛选选项（今天、昨天、最近7天、自定义范围等）
- 模糊搜索框：带水印提示的搜索输入框
- 高度：50px

#### 3.2.3 分类标签栏
- 8个分类标签按钮：所有、致命、错误、警告、信息、调试、跟踪、其它
- 每个标签显示对应级别的日志数量
- 激活状态高亮显示
- 高度：40px

#### 3.2.4 日志列表区域
- 四列显示：时间、级别、记录者、摘要
- 支持按列排序
- 虚拟滚动技术（处理大量日志条目）
- 行高亮效果（悬停、选中）
- 双击行可在详情区查看完整内容

#### 3.2.5 日志详情区域
- 标签页切换：文本、浏览器、JSON、XML
- 文本视图：纯文本显示，支持查找
- 浏览器视图：渲染HTML内容
- JSON视图：格式化显示，支持展开/折叠
- XML视图：格式化显示，支持语法高亮

#### 3.2.6 状态栏
- 操作提示信息
- 刷新按钮
- 日志条目统计显示
- 高度：30px

### 3.3 颜色方案
- 主色调：深灰(#2d2d30)
- 辅助色：蓝色(#007acc)
- 背景色：深灰(#1e1e1e)
- 文字颜色：浅灰(#cccccc)
- 高亮色：橙色(#ce9178)
- 错误级别色：红色(#f44747)
- 警告级别色：黄色(#d7ba7d)
- 信息级别色：蓝色(#569cd6)

## 4. 技术实现方案

### 4.1 技术栈选择

#### 前端框架
- **方案1**：Electron + React/Vue + TypeScript（推荐）
- **方案2**：WPF + C#（Windows原生）
- **方案3**：Qt + C++（跨平台原生）

#### 推荐技术栈：Electron + React + TypeScript
- 优势：跨平台、开发效率高、社区活跃
- 组件库：Ant Design 或 Material-UI
- 状态管理：Redux 或 MobX
- 打包工具：electron-builder

### 4.2 核心模块实现

#### 4.2.1 大文件读取模块
```javascript
// 伪代码示例
class LogFileReader {
  constructor(filePath, chunkSize = 1024 * 1024) {
    this.filePath = filePath;
    this.chunkSize = chunkSize; // 1MB每块
    this.encoding = 'utf-8';
  }
  
  async *readByLine() {
    // 使用流式读取，避免内存溢出
    const stream = fs.createReadStream(this.filePath, {
      encoding: this.encoding,
      highWaterMark: this.chunkSize
    });
    
    let remaining = '';
    for await (const chunk of stream) {
      const lines = (remaining + chunk).split(/\r?\n/);
      remaining = lines.pop(); // 最后一行可能不完整
      
      for (const line of lines) {
        if (line.trim()) yield line;
      }
    }
    
    if (remaining) yield remaining;
  }
  
  async parseLogLine(line) {
    // 解析单行日志，提取时间、级别、记录者、摘要
    const patterns = [
      // 常见日志格式正则表达式
      /(?<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}.\d{3})\s+\[?(?<level>\w+)\]?\s+(?<logger>\S+)\s+-\s+(?<summary>.+)/,
      // 更多格式...
    ];
    
    for (const pattern of patterns) {
      const match = line.match(pattern);
      if (match) {
        return {
          timestamp: match.groups.timestamp,
          level: match.groups.level,
          logger: match.groups.logger,
          summary: match.groups.summary,
          raw: line
        };
      }
    }
    
    return {
      timestamp: new Date().toISOString(),
      level: 'OTHER',
      logger: 'Unknown',
      summary: line,
      raw: line
    };
  }
}
```

#### 4.2.2 日志分类与过滤模块
```javascript
class LogFilter {
  constructor(logs) {
    this.logs = logs;
    this.filteredLogs = [...logs];
  }
  
  filterByLevel(level) {
    if (level === 'ALL') {
      this.filteredLogs = [...this.logs];
    } else {
      this.filteredLogs = this.logs.filter(log => 
        log.level.toUpperCase() === level.toUpperCase()
      );
    }
    return this;
  }
  
  filterByTimeRange(startTime, endTime) {
    this.filteredLogs = this.filteredLogs.filter(log => {
      const logTime = new Date(log.timestamp).getTime();
      return logTime >= startTime && logTime <= endTime;
    });
    return this;
  }
  
  search(keyword) {
    if (!keyword.trim()) return this;
    
    this.filteredLogs = this.filteredLogs.filter(log => {
      return log.raw.toLowerCase().includes(keyword.toLowerCase()) ||
             log.summary.toLowerCase().includes(keyword.toLowerCase());
    });
    return this;
  }
  
  getResults() {
    return this.filteredLogs;
  }
}
```

#### 4.2.3 虚拟滚动列表组件
```jsx
// React虚拟列表组件示例
import { FixedSizeList as List } from 'react-window';

const LogList = ({ logs, height, onRowClick }) => {
  const Row = ({ index, style }) => {
    const log = logs[index];
    const levelColor = getLevelColor(log.level);
    
    return (
      <div 
        style={style} 
        className="log-row"
        onClick={() => onRowClick(log)}
      >
        <div className="log-time">{formatTime(log.timestamp)}</div>
        <div className="log-level" style={{color: levelColor}}>
          {log.level}
        </div>
        <div className="log-logger">{log.logger}</div>
        <div className="log-summary">{log.summary}</div>
      </div>
    );
  };
  
  return (
    <List
      height={height}
      itemCount={logs.length}
      itemSize={35}
      width="100%"
    >
      {Row}
    </List>
  );
};
```

### 4.3 性能优化策略

1. **流式读取**：使用Node.js流API分块读取大文件
2. **虚拟滚动**：只渲染可视区域内的日志条目
3. **Web Worker**：将日志解析放在后台线程，避免界面阻塞
4. **索引缓存**：为已解析的日志文件创建索引，加速后续筛选
5. **内存管理**：实现LRU缓存机制，释放不常用的数据
6. **增量加载**：先加载部分日志，用户滚动时再加载更多

## 5. 开发计划

### 5.1 阶段划分

#### 第一阶段：基础框架（2周）
- 搭建Electron + React开发环境
- 实现主窗口基本布局
- 完成文件拖拽加载功能
- 实现基本的日志解析

#### 第二阶段：核心功能（3周）
- 完善日志解析器，支持多种格式
- 实现分类标签和计数功能
- 开发搜索和筛选功能
- 实现虚拟滚动列表

#### 第三阶段：增强功能（2周）
- 实现多格式内容查看器（文本、浏览器、JSON、XML）
- 添加时间筛选功能
- 实现刷新和重置功能
- 优化性能，添加加载进度指示

#### 第四阶段：测试优化（1周）
- 性能测试与优化
- 用户体验测试
- Bug修复
- 文档编写

### 5.2 里程碑
- M1：完成基础框架，可加载和显示日志
- M2：实现分类、搜索、筛选功能
- M3：完成所有界面功能
- M4：性能优化和测试完成
- M5：发布第一个可用版本

## 6. 测试方案

### 6.1 功能测试
- 文件加载测试（不同大小、格式）
- 分类筛选准确性测试
- 搜索功能测试
- 时间筛选测试
- 多标签内容显示测试

### 6.2 性能测试
- 大文件加载性能测试（100MB, 500MB, 1GB, 2GB）
- 内存占用测试
- 筛选和搜索响应时间测试
- 长时间运行稳定性测试

### 6.3 兼容性测试
- Windows 10/11
- macOS
- Linux（主要发行版）

## 7. 部署与发布

### 7.1 打包配置
```json
// electron-builder配置示例
{
  "appId": "com.yourcompany.logviewer",
  "productName": "LogViewer",
  "directories": {
    "output": "dist"
  },
  "files": [
    "build/**/*",
    "node_modules/**/*"
  ],
  "mac": {
    "category": "public.app-category.developer-tools"
  },
  "win": {
    "target": "nsis"
  },
  "linux": {
    "target": "AppImage"
  }
}
```

### 7.2 发布渠道
- GitHub Releases
- 官方网站下载
- Chocolatey（Windows包管理）
- Homebrew（macOS包管理）

## 8. 后续优化方向

### 8.1 功能增强
- 支持多标签页同时查看多个日志文件
- 日志导出功能（Excel, CSV, PDF）
- 自定义日志解析规则
- 插件系统支持
- 日志分析和统计报表

### 8.2 用户体验优化
- 自定义主题
- 快捷键自定义
- 布局自定义
- 搜索结果导出

### 8.3 技术优化
- 使用WebAssembly加速日志解析
- 实现日志文件的增量更新监控
- 添加日志数据可视化图表
- 云端日志分析功能

## 9. 注意事项

1. **内存管理**：处理大文件时必须注意内存使用，避免崩溃
2. **编码支持**：确保支持多种文本编码（UTF-8, GBK, GB2312等）
3. **错误处理**：完善的错误处理机制，确保软件稳定性
4. **用户体验**：大文件加载时提供进度反馈和取消选项
5. **可访问性**：支持键盘操作和屏幕阅读器

## 10. 附录

### 10.1 常见日志格式示例
```
// 标准日志格式
2023-10-01 12:30:45.123 INFO  com.example.Service - 用户登录成功 userId=123

// 错误日志
2023-10-01 12:31:15.456 ERROR com.example.Dao - 数据库连接失败

// 多行日志
2023-10-01 12:32:00.789 DEBUG com.example.Processor - 开始处理任务
任务参数: {...}
处理步骤1: ...
处理步骤2: ...
```

### 10.2 参考资源
- Electron官方文档：https://www.electronjs.org/
- React虚拟列表：https://github.com/bvaughn/react-window
- 大文件处理：Node.js Stream API
- 日志解析库：logform、winston等

---

这份文档提供了从功能设计到技术实现的完整方案。根据您的技术偏好，可以选择不同的技术栈实现。建议从最小可行版本开始，逐步迭代完善功能。如果需要更详细的技术实现代码或特定模块的设计，请随时提出。