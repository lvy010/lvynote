前端

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