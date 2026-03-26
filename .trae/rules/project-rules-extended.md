---
alwaysApply: false
---
# Extended Rules

## Logging Standard
### 原则
- 必须可观测
- 日志结构化

### 推荐格式
{
  "event": "",
  "module": "",
  "status": "",
  "message": ""
}

### 必须记录
- 错误
- 关键流程
- 外部调用

## Testing Policy
### 必须测试
- 核心业务逻辑
- 边界条件
- 异常情况

### 测试原则
- 测试独立
- 可重复执行

## Dependency Detail
### 原则
- 优先使用标准库
- 最小依赖原则

### 新增依赖必须说明
- 使用目的
- 是否有替代方案
- 引入成本

### 禁止
- 重复功能库
- 不必要大型依赖