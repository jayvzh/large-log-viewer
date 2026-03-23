## 高亮系统（模板驱动 + token化 ）

***

# 一、设计目标（必须满足）

你的高亮系统必须做到：

- ✅ **模板驱动**（不同日志不同规则）
- ✅ **零运行时复杂正则**（避免卡顿）
- ✅ **支持字段 / token / 关键词**
- ✅ **支持用户自定义**
- ✅ **前端纯渲染（无计算）**

***

# 二、核心设计思路（很关键）

> ❗高亮 ≠ 再跑一遍 regex\
> ✅ 高亮 = 利用“解析结果 + 轻量规则”

***

# 三、DSL 结构（推荐最终版🔥）

用 JSON（便于你现有模板系统兼容）

***

## 🎯 完整示例

```
{
  "highlight": {
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
            "DEBUG": "gray"
          }
        }
      },
      {
        "type": "field",
        "field": "status",
        "style": {
          "map": {
            "200": "green",
            "301": "cyan",
            "404": "yellow",
            "500": "red bold"
          }
        }
      },
      {
        "type": "token",
        "pattern": "ID:\\d+",
        "style": "purple"
      },
      {
        "type": "keyword",
        "words": ["failed", "error", "exception"],
        "style": "red bold"
      },
      {
        "type": "keyword",
        "words": ["success", "ok"],
        "style": "green"
      },
      {
        "type": "regex",
        "pattern": "\\b\\d{1,3}(\\.\\d{1,3}){3}\\b",
        "style": "cyan"
      }
    ]
  }
}

```

***

# 四、规则类型说明（核心）

***

## 1️⃣ field（最重要，性能最高）

👉 基于你解析后的字段

```
{
  "type": "field",
  "field": "level",
  "style": {
    "map": {
      "ERROR": "red"
    }
  }
}

```

👉 Rust阶段直接标记，无需 regex

***

## 2️⃣ token（推荐）

👉 针对 message 内“结构片段”

```
{
  "type": "token",
  "pattern": "ID:\\d+",
  "style": "purple"
}

```

👉 Rust 编译 regex，一次扫描

***

## 3️⃣ keyword（高性价比🔥）

```
{
  "type": "keyword",
  "words": ["failed", "timeout"],
  "style": "red"
}

```

👉 实现建议：

- 用 Aho-Corasick（Rust 超快）
- O(n) 扫描

***

## 4️⃣ regex（慎用）

```
{
  "type": "regex",
  "pattern": "xxx",
  "style": "..."
}

```

👉 用于复杂情况\
👉 数量必须限制（否则卡）

***

# 五、样式 DSL（简单但够用）

***

## 推荐格式：

```
"style": "red bold"

```

***

## 支持：

属性

示例

颜色

red / green / blue / yellow / cyan / purple / gray

粗体

bold

斜体

italic

下划线

underline

***

## 前端映射：

```
const styleMap = {
  "red": "color-red",
  "bold": "font-bold"
};

```

***

# 六、Rust 数据结构设计（关键）

***

```
struct HighlightRule {
    rule_type: RuleType,
    field: Option<String>,
    pattern: Option<Regex>,
    words: Option<Vec<String>>,
    style: Style,
}

enum RuleType {
    Field,
    Token,
    Keyword,
    Regex,
}

```

***

## 输出给前端：

```
struct HighlightSpan {
    start: usize,
    end: usize,
    class: String,
}

```

***

# 七、渲染流程（高性能核心🔥）

***

## Rust：

1. 解析日志 → LogEvent
2. 应用 highlight rules
3. 生成 spans

***

## Svelte：

```
{#each spans as span}
  <span class={span.class}>
    {text.slice(span.start, span.end)}
  </span>
{/each}

```

***

# 八、性能优化（非常关键）

***

## ❗规则优先级（必须有）

```
field > keyword > token > regex

```

👉 先 cheap，再 expensive

***

## ❗缓存策略

- 每行解析一次
- highlight 结果缓存

***

## ❗限制 regex 数量

```
<= 3 条 regex

```

***

# 九、模板集成方式

你的模板现在应该变成：

```
{
  "name": "nginx_access",
  "pattern": "...",
  "mapping": {...},
  "highlight": { ... }   ← 新增
}

```

***

# 十、用户体验增强（建议你加）

***

## 🎯 1. UI配置高亮

```
高亮规则：
[+ 添加规则]
类型：keyword
关键词：[failed, timeout]
颜色：[红色]

```

***

## 🎯 2. 实时预览（非常重要）

👉 输入一行日志 → 立即高亮

***

## 🎯 3. 一键预设

```
通用日志高亮
Web日志高亮
安全日志高亮

```

***

# 十一、进阶（你以后一定会做）

***

## 🔥 条件高亮（高级DSL）

```
{
  "type": "conditional",
  "when": {
    "field": "status",
    "value": "500"
  },
  "style": "red bold"
}

```

***

## 🔥 多字段组合

```
{
  "when": {
    "level": "ERROR",
    "source": "auth"
  }
}

```

***

# 十二、一句话总结

> ✅ 高亮系统 = 模板DSL + Rust预处理 + 前端纯渲染\
> ❌ 绝不能在前端做复杂计算

***

