# Orion Server Buck2 日志系统 - 多目标构建支持重构

增强对单个 CL 内多目标构建的支持。当前系统已具备基础的多构建能力，但缺乏目标级别的元数据管理和状态追踪，导致调试困难和前端展示受限。

## 2. 背景与现状分析

### 2.1 当前系统架构

Orion Server 当前采用三层数据模型： [1](#0-0) 

**现有架构：**
```
CL (Change List)
 └── Task (构建任务，包含 task_name, template 等元数据)
      └── Builds[] (一个 Task 可以包含多个 Build)
           └── Log File (每个 Build 对应一个日志文件)
```

### 2.2 识别的问题

#### **问题 1：Target 信息扁平化**

当前 `builds` 表中的 `target` 字段仅为字符串类型，无法存储目标级别的元数据（如状态、错误摘要、执行统计）： [4](#0-3) 

#### **问题 2：状态管理粒度不足**

状态枚举仅在 API 层临时计算，未在数据层持久化目标级状态： [5](#0-4) [6](#0-5) 

#### **问题 3：前端无法按目标分组展示**

前端 Checks 组件需要手动遍历所有 builds 来确定构建状态，无法直接按 target 分组： [7](#0-6) 

### 2.3 技术栈确认

- **后端框架：** Axum (Rust)
- **ORM：** SeaORM
- **数据库：** PostgreSQL
- **日志存储：** 文件系统 (BUILD_LOG_DIR/{build_id})
- **实时推送：** Server-Sent Events (SSE) [8](#0-7) 

---

## 3. 目标

本次重构旨在：

1. **引入独立的 `targets` 表**，管理目标级元数据
2. **增强状态追踪能力**，支持目标级状态持久化
3. **优化 API 设计**，支持按目标查询和前端分组展示
4. **保持向后兼容**，现有 API 端点继续工作

---

## 4. 数据模型重构

### 4.1 新增 `targets` 表

在 `orion-server/src/model/` 下创建 `targets.rs`： [9](#0-8) 

**表结构设计：**

| 列名            | 类型        | 约束         | 说明                                                         |
| --------------- | ----------- | ------------ | ------------------------------------------------------------ |
| `id`            | UUID        | PRIMARY KEY  | 目标记录唯一标识                                             |
| `task_id`       | UUID        | NOT NULL, FK | 关联到 tasks 表                                              |
| `target_path`   | VARCHAR     | NOT NULL     | Buck2 目标路径（如 `//app:server`）                          |
| `state`         | VARCHAR     | NOT NULL     | 目标状态：`Pending`, `Building`, `Completed`, `Failed`, `Interrupted` |
| `start_at`      | TIMESTAMPTZ | NULL         | 目标构建开始时间                                             |
| `end_at`        | TIMESTAMPTZ | NULL         | 目标构建结束时间                                             |
| `error_summary` | TEXT        | NULL         | 失败时的错误摘要（从日志提取）                               |
| `created_at`    | TIMESTAMPTZ | NOT NULL     | 记录创建时间                                                 |

**关系设计：**
- `targets.task_id` → `tasks.id` (ON DELETE CASCADE)
- `builds.target_id` → `targets.id` (ON DELETE CASCADE)

**索引：**
```sql
CREATE INDEX idx_targets_task_id ON targets (task_id);
CREATE INDEX idx_targets_state ON targets (state);
CREATE INDEX idx_targets_created_at ON targets (created_at);
```

### 4.2 修改 `builds` 表

**变更：**
1. 将 `target` 字段（VARCHAR）替换为 `target_id` (UUID)
2. 添加外键约束：`builds.target_id` → `targets.id`

**迁移策略：**
```sql
-- 步骤 1：添加新列
ALTER TABLE builds ADD COLUMN target_id UUID;

-- 步骤 2：数据迁移（为现有 builds 创建对应的 targets 记录）
-- 步骤 3：设置外键约束
ALTER TABLE builds ADD CONSTRAINT fk_builds_target_id 
    FOREIGN KEY (target_id) REFERENCES targets(id) ON DELETE CASCADE;

-- 步骤 4：删除旧列（在验证后）
ALTER TABLE builds DROP COLUMN target;
```

### 4.3 SeaORM Entity 定义

参考现有 model 的实现风格： [10](#0-9) 

新的 `targets.rs` 需要实现：
- `Model` 结构体
- `Entity` 和 `Column` 枚举
- `Relation` 定义（与 `tasks` 和 `builds` 的关联）
- 辅助方法（`create_target`, `insert_target`, `update_state` 等）

---

## 5. API 重构

### 5.1 新增端点

#### **5.1.1 获取 Task 的所有 Targets**

**端点：** `GET /tasks/{task_id}/targets`

**响应示例：**
```json
{
  "task_id": "01933e5e-...",
  "targets": [
    {
      "id": "01933e5f-...",
      "target_path": "//app:server",
      "state": "Completed",
      "start_at": "2026-01-13T15:30:00Z",
      "end_at": "2026-01-13T15:35:00Z",
      "error_summary": null,
      "builds": [
        {
          "id": "01933e60-...",
          "exit_code": 0,
          "output_file": "./logs/01933e60-..."
        }
      ]
    }
  ]
}
```

#### **5.1.2 获取 Target 日志**

**端点：** `GET /targets/{target_id}/logs`

**查询参数：**
- `type`: `full` | `segment`
- `offset`: 起始行号（segment 模式）
- `limit`: 返回行数（segment 模式）

**实现：** 复用现有的日志读取逻辑： [11](#0-10) 

### 5.2 增强现有端点

#### **5.2.1 `/tasks/{cl}` 返回结构调整**

当前实现： [12](#0-11) 

**调整后的响应结构：**
```json
{
  "task_id": "01933e5e-...",
  "cl_id": 12345,
  "task_name": "Buck2 Build",
  "created_at": "2026-01-13T15:30:00Z",
  "targets": [
    {
      "id": "01933e5f-...",
      "target_path": "//app:server",
      "state": "Completed",
      "builds": [...]
    }
  ]
}
```

**变更要点：**
- 在 DTO 中添加 `targets` 字段（按 target 分组的 builds）
- 保持 `build_list` 字段以保证向后兼容

### 5.3 向后兼容性

现有端点行为保持不变： [13](#0-12) [14](#0-13) 

---

## 6. 实现策略

### 6.1 数据库迁移

参考现有的迁移模式： [15](#0-14) 

**新建迁移文件：** `orion-server/migration/m20260113_create_targets_table.rs`

### 6.2 调度器集成

当前调度器在分派任务时创建 build 记录： [16](#0-15) 

**调整点：**
1. 在 `dispatch_task` 中先创建 `targets` 记录
2. 创建 `builds` 时关联 `target_id`
3. 构建完成时更新 `targets.state` 和 `targets.end_at`

### 6.3 状态更新逻辑

在 WebSocket 消息处理中添加目标状态更新： [17](#0-16) 

**增强点：**
```rust
WSMessage::BuildComplete { id, exit_code, .. } => {
    // 更新 builds 表
    builds::Entity::update(...)
    
    // 新增：更新 targets 表状态
    targets::Entity::update_state(target_id, determine_target_state(exit_code))
}
```

### 6.4 错误摘要提取

复用现有的日志解析逻辑： [18](#0-17) 

**增强：** 将提取的错误摘要写入 `targets.error_summary` 字段

---

## 7. 前端适配

### 7.1 Checks 组件调整

当前组件按 build 遍历： [19](#0-18) 

**调整方案：**
1. 后端返回时已按 target 分组
2. 前端按 `task.targets` 渲染，每个 target 显示状态徽章
3. 点击 target 展开其下的所有 builds

### 7.2 新增 TypeScript 类型

在 `moon/packages/types/generated.ts` 中添加：

```typescript
export interface TargetDTO {
  id: string;
  target_path: string;
  state: 'Pending' | 'Building' | 'Completed' | 'Failed' | 'Interrupted';
  start_at: string;
  end_at?: string;
  error_summary?: string;
  builds: BuildDTO[];
}

export interface TaskInfoDTO {
  task_id: string;
  cl_id: number;
  task_name?: string;
  created_at: string;
  targets: TargetDTO[];  // 新增
  build_list: BuildDTO[]; // 保留（向后兼容）
}
```

---

## 9. 性能考量

### 9.1 查询优化

当前的 `/tasks/{cl}` 查询： [20](#0-19) 

**优化方案：**
- 使用 SeaORM 的 `find_with_related` 一次性加载 `tasks -> targets -> builds`
- 添加数据库索引（如上 4.1 节所示）

### 9.2 日志文件结构保持不变

保持现有的日志文件结构（一个 build 一个文件）： [21](#0-20) 

**原因：**
- 避免复杂的日志文件分割/合并逻辑
- 保持日志 SSE 推送的简单性
- Target 层面的日志聚合在 API 层实现

---

## 10. 风险与缓解

| 风险                      | 影响 | 概率 | 缓解措施                                                     |
| ------------------------- | ---- | ---- | ------------------------------------------------------------ |
| 数据迁移失败              | 高   | 低   | 1. 在测试环境完整验证<br>2. 保留旧数据备份<br>3. 支持回滚    |
| 性能回归                  | 中   | 中   | 1. 提前进行性能基准测试<br>2. 添加数据库索引<br>3. 使用 explain analyze 优化查询 |
| 前端兼容性问题            | 中   | 低   | 1. 保留 `build_list` 字段<br>2. 渐进式发布                   |
| Orion Server 重启丢失状态 | 低   | 低   | 状态已持久化到数据库，无此风险                               |

