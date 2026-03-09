# Codex Switch+ 产品需求文档（PRD）

## 1. 背景与目标

当前 Codex 的关键配置分散在多个文件与命令中（如 `~/.codex/config.toml`、`~/.codex/rules/default.rules`、`~/.codex/skills/**`），维护成本高，且缺少可视化调试能力（尤其是 rules 命中验证、sandbox 网络配置、skills 状态管理）。

目标是构建一个桌面端工具（暂定名：**Codex Switch+**），技术栈为 **Tauri + Vue + TypeScript + Tailwind CSS + PrimeVue**，对标 cc-switch 的易用性，并在 Codex 场景下提供更强的配置与调试能力。

## 2. 产品定位

- 面向对象：重度使用 Codex CLI/Codex App 的开发者与团队。
- 核心价值：
  - 一站式管理 Codex 配置（API Key、URL、模型、network_access、rules、skills）。
  - 提供可视化规则调试能力，减少“为什么命令没跑起来”的排障时间。
  - 降低手改配置文件与命令行操作风险。

## 3. 范围

### 3.1 In Scope（一期）

1. Codex 基础配置管理
2. Sandbox / 网络配置管理
3. Rules 可视化编辑与调试
4. Skills 浏览与管理
5. 配置快照（导出/回滚）

### 3.2 Out of Scope（一期不做）

1. 远程多机同步（云端账号体系）
2. 复杂团队权限系统（RBAC）
3. 自动化任务编排器（仅做查看入口，不做完整编排）

## 4. 用户故事

1. 作为开发者，我希望在 UI 中配置 Codex API Key 与 Base URL，避免手动编辑 `config.toml`。
2. 作为开发者，我希望显式开启 `workspace-write` 的 `network_access`，避免自动化里 `glab`/HTTP 请求失败。
3. 作为开发者，我希望可视化编辑 `rules`，并一键验证命令是否命中 allow。
4. 作为开发者，我希望看到本机 skills 列表、元信息和脚本入口，快速确认技能是否可用。
5. 作为开发者，我希望在改配置前后可创建快照并回滚，降低误操作风险。

## 5. 功能需求

## 5.1 配置中心（Config Center）

### 功能点

- 读取并编辑 `~/.codex/config.toml`
- 支持字段：
  - `model_provider`
  - `model`
  - `model_reasoning_effort`
  - `network_access`
  - `[model_providers.<name>]` 下的 `base_url`、`wire_api`、`requires_openai_auth`
  - `[sandbox_workspace_write].network_access`
  - `[projects."<path>"].trust_level`
- API Key 管理：
  - 支持读取/写入 `OPENAI_API_KEY`（优先写 OS Keychain，降级写 `.env` 可选）
  - UI 中始终脱敏显示

### 交互要求

- 表单校验（URL 格式、枚举值合法性）
- “预览 diff” 后再保存
- 保存后自动备份旧文件

## 5.2 Sandbox 与网络配置（Codex 特有）

### 功能点

- 可视化切换 sandbox 模式：
  - `read-only`
  - `workspace-write`
  - `danger-full-access`（高风险确认）
- 可视化开关 `workspace-write` 的 `network_access`
- 提示常见坑：
  - “rules 已放行但 network_access 关闭，联网命令仍失败”

### 验证能力

- 内置“网络探测”按钮（执行安全探测命令并展示结果）
- 展示最近一次失败原因摘要（DNS、权限、rules miss、命令不存在）

## 5.3 Rules 管理（核心差异化）

### 功能点

- 读取/编辑 `~/.codex/rules/default.rules`
- 提供规则模板：
  - `prefix_rule(...) allow/deny`
  - 常用命令模板（`glab`、`python3 script`、`bash script`）
- 规则分组展示（按命令前缀、来源、最近命中次数）

### 调试能力（必须）

- 内置 playground：
  - 输入一条命令
  - 调用 `codex execpolicy check --pretty --rules <file> -- <cmd...>`
  - 返回命中规则、decision、未命中原因
- 支持“从失败命令自动生成候选规则”（需用户确认）

### 安全要求

- 对过宽规则给出警告（如 `["glab"]`、`["bash"]` 级别）
- 支持一键回滚到上一个规则快照

## 5.4 Skills 管理

### 功能点

- 扫描 `~/.codex/skills/**/SKILL.md`
- 展示字段：
  - `name`
  - `description`
  - 路径
  - 脚本入口（`scripts/`）
- 操作能力：
  - 打开 skill 目录
  - 查看/编辑 `SKILL.md`
  - 触发 quick validate（如存在校验脚本）

### 增强能力

- 显示 skill 对应命令依赖（如 `glab`、`python3`、`jq`）
- 一键生成对应 rules 建议（最小权限）

## 5.5 快照与回滚

### 功能点

- 针对以下文件做版本快照：
  - `config.toml`
  - `rules/*.rules`
- 快照元信息：
  - 时间
  - 修改人（本机用户名）
  - 变更摘要
- 支持按快照回滚并二次确认

## 6. 非功能需求

1. 跨平台：macOS 优先，兼容 Windows/Linux（Tauri）
2. 安全性：
   - API Key 不明文落盘（优先系统密钥链）
   - 高危操作二次确认（danger-full-access、宽规则）
3. 可观测性：
   - 本地操作日志
   - 调试结果可复制
4. 性能：
   - 首屏 < 2s（本地配置场景）
   - rules 调试单次反馈 < 500ms（不含外部网络）

## 7. 信息架构与页面

1. 仪表盘（Dashboard）
2. 配置中心（Config）
3. Sandbox/网络（Sandbox & Network）
4. Rules（列表 + 编辑 + 调试）
5. Skills（列表 + 详情 + 依赖）
6. 快照中心（Snapshots）

## 8. 技术方案（建议）

## 8.1 前端

- Vue 3 + TypeScript + Vite
- UI：PrimeVue
- 样式：Tailwind CSS
- 状态管理：Pinia
- 表单校验：Zod + VeeValidate（可选）

## 8.2 Tauri 后端（Rust）

- 文件操作（读写 `~/.codex/**`）
- 调用命令：
  - `codex execpolicy check`
  - `codex --help` / `codex exec --help`（环境探测）
- 安全封装：
  - 命令白名单执行
  - 路径访问控制

## 8.3 数据结构（前端）

- `CodexConfig`
- `RuleEntry`
- `SkillMeta`
- `SnapshotMeta`
- `RuleCheckResult`

## 9. 验收标准（MVP）

1. 能在 UI 中修改并保存 `config.toml`，且自动生成备份。
2. 能配置并持久化 `workspace-write` 的 `network_access`。
3. 能可视化编辑 `default.rules` 并通过 `execpolicy check` 完成调试。
4. 能扫描并展示本机 skills 列表，读取 `SKILL.md` 元信息。
5. 能创建、查看、回滚快照。
6. 对 API Key 全程脱敏显示，且支持系统密钥链存储。

## 10. 里程碑

### M1（1-2 周）

- 配置中心 + Sandbox 网络配置
- Rules 基础编辑

### M2（第 3 周）

- Rules 调试器（execpolicy check）
- Skills 列表与详情

### M3（第 4 周）

- 快照与回滚
- 安全增强与体验打磨

## 11. 风险与对策

1. Codex CLI 版本差异导致命令输出不一致
   - 对策：做版本探测与适配层
2. 错误规则导致命令被拦截
   - 对策：规则调试器 + 一键回滚
3. 用户误开高危 sandbox
   - 对策：高风险确认弹窗 + 警示文案

## 12. 命名建议

- 产品名：**Codex Switch+**
- Slogan：**One panel to control Codex config, rules, skills, and debugging**
