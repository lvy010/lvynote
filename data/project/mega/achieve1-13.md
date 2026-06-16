概要

- 新增模型 `targets`（目标级元数据），`builds` 增加 `target_id` 外键并去掉旧的 `target` 字符串，`tasks` 关联目标；相应的实体、关系与帮助方法已补齐。
- 调度与实时处理：派发/重试时会为目标建/查记录并标记为 Building，完成/中断时写回目标状态、结束时间与错误摘要；健康检查失联也会把目标标记为 Interrupted。
- API 扩展：`/tasks/{task_id}/targets` 返回按目标分组的任务详情；`/targets/{target_id}/logs` 支持 full/segment 读日志；`/tasks/{cl}` 响应中新增 `targets` 分组，同时保留 `build_list` 兼容；BuildDTO 现在包含 `target_id`，`target` 字段填充目标路径。
- 开放接口文档已同步（OpenAPI 路径/Schema），新增迁移文件 `orion-server/migration/create_targets_table.rs`（创建 targets 表、builds 追加 target_id、回填/索引/外键，需 pgcrypto 的 `gen_random_uuid()` 或自备 UUID 函数）。

前端对接

- 目标状态枚举：`Pending | Building | Completed | Failed | Interrupted`。
- 任务详情（`GET /tasks/{cl}` 与 `GET /tasks/{task_id}/targets`）：
  - 字段：`task_id`, `cl_id`, `task_name?`, `template?`, `created_at`, `build_list: BuildDTO[]`（兼容老逻辑）, `targets: TargetDTO[]`。
  - TargetDTO：`id`, `target_path`, `state`, `start_at?`, `end_at?`, `error_summary?`, `builds: BuildDTO[]`。
  - BuildDTO：`id`, `task_id`, `target_id`, `exit_code?`, `start_at`, `end_at?`, `repo`, `target`(路径), `args?`, `output_file`, `created_at`, `retry_count`, `status`, `cause_by?`。
- 日志：
  - `GET /targets/{target_id}/logs`：
    - Query：`type=full|segment`（默认 full），`offset`，`limit`（segment 模式行数，默认 200）。
    - 响应：`{ data: string[], len: number, build_id: string }`，404 表示目标或构建不存在。
  - 旧接口 `/task-history-output`、`/task-output/{id}` 保持不变。
- 任务列表 `/tasks/{cl}` 现已按目标分组返回，前端渲染时优先使用 `targets` 分组；旧的 `build_list` 仍可用作兼容路径。
- 创建/重试任务：
  - BuildRequest 新增可选字段 `target`（别名 `target_path`，如 `//app:server`），缺省时后台默认 `//...`。
  - 返回的 build 结果与旧版一致。

数据库迁移提示

- 路径：`orion-server/migration/m20260113_create_targets_table.rs`，包含建表、回填、索引与外键；使用了 `gen_random_uuid()`，若未启用 pgcrypto 请替换为可用的 UUID 生成函数或手工填充。

测试

- 自动化测试；本地 `cargo check` 与手工验证新接口。