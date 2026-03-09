# Codex Switch+（中文说明）

<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="Codex Switch+ 图标" width="96" />
</p>

English version: [README.md](./README.md)

## 项目简介

Codex Switch+ 是一个用于管理本机 Codex 配置与规则工作流的桌面工具。  
技术栈为 Tauri 2 + Vue 3，默认操作真实 `~/.codex`，并通过 diff 预览、二次确认、自动备份与快照降低误操作风险。

## 主要能力

- 配置中心：管理 `~/.codex/config.toml`
- API Key 管理：写入 `~/.codex/.env` 的 `OPENAI_API_KEY`，读取脱敏
- Sandbox 与网络开关
- Rules 可视化编辑 + 原始编辑 + playground 调试
- Rules 单条规则运行检查
- Skills 扫描（`~/.codex/skills/**/SKILL.md`）与 Markdown 预览
- 快照创建 / 列表 / 回滚
- 本地审计日志
- 中英文国际化

## 页面路由

- `/dashboard`
- `/config`
- `/sandbox`
- `/rules`
- `/skills`
- `/snapshots`

## 技术栈

- 前端：Vue 3、Vue Router、Pinia、PrimeVue、Tailwind CSS v4、Zod
- 桌面后端：Tauri 2（Rust）
- Markdown 渲染：`marked` + `dompurify`

## 环境要求

- Node.js 18+
- pnpm 9+
- Rust stable toolchain
- 对应操作系统的 Tauri 2 依赖
- `codex` CLI 在 `PATH` 中（Rules 调试依赖）

## 快速开始

安装依赖：

```bash
pnpm install
```

仅前端开发：

```bash
pnpm dev
```

桌面端开发（推荐）：

```bash
pnpm tauri dev
```

构建前端：

```bash
pnpm build
```

构建桌面安装包：

```bash
pnpm tauri build
```

## 数据路径与安全模型

默认读写真实 Codex 目录：

- 配置：`~/.codex/config.toml`
- API Key 文件：`~/.codex/.env`
- Rules：`~/.codex/rules/default.rules`
- Skills 根目录：`~/.codex/skills`

应用自有目录：

- 快照：`~/.codex-switch-plus/snapshots`
- 备份：`~/.codex-switch-plus/backups`
- 审计日志：`~/.codex-switch-plus/logs`

写入流程：

1. 生成 diff 预览
2. 用户确认
3. 写入文件
4. 自动备份 + 审计记录

## 后端命令（Tauri Invoke）

- `get_config` / `save_config`
- `get_api_key` / `set_api_key`
- `get_sandbox` / `set_sandbox`
- `probe_network`
- `get_rules` / `save_rules`
- `check_rule_command` / `check_single_rule`
- `scan_skills` / `read_skill`
- `create_snapshot` / `list_snapshots` / `rollback_snapshot`

## 注意事项

- `danger-full-access` 必须显式二次确认。
- Rules 调试使用命令：

```bash
codex execpolicy check --pretty --rules <rules-file> -- <command...>
```

- Skills 的“打开目录/文件”依赖 Tauri opener 权限（`opener:allow-open-path`）。

## 常见问题

- 找不到 `codex` 命令：
  - 安装 Codex CLI，并确认其在 shell 的 `PATH` 中。
- Rules playground 运行失败：
  - 检查 rules 语法和本机 Codex CLI 版本兼容性。
- 无法打开技能目录或文件：
  - 检查系统文件访问权限。

## 贡献者说明

### 开发流程

1. 安装依赖：

```bash
pnpm install
```

2. 启动桌面开发：

```bash
pnpm tauri dev
```

3. 只做前端迭代可使用：

```bash
pnpm dev
```

### PR 前检查

提交前至少执行：

```bash
pnpm build
```

涉及 Rust 后端改动时建议执行：

```bash
cd src-tauri
cargo test --lib
```

### 目录约定

- `src/`：Vue 页面、路由、i18n、前端逻辑
- `src-tauri/`：Rust 命令与桌面能力配置
- `docs/`：产品与规划文档

### PR 清单

- 单个 PR 聚焦单一目标
- 用户可见文案必须同步维护 i18n key（`src/i18n/index.ts`）
- 不破坏“预览 diff + 确认 + 备份”的安全写入流程
- 确认明亮/暗黑模式可读性
- 确认中英文切换可用

### Commit Message

- commit message 使用英文
- 标题简洁、动词导向
