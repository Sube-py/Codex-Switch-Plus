# Codex Switch+

<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="Codex Switch+ Icon" width="96" />
</p>

A desktop control panel for Codex local configuration and policy workflow.

Built with Tauri 2 + Vue 3, Codex Switch+ helps you manage real `~/.codex` files safely with preview diffs, confirmations, backups, and snapshots.

Chinese version: [README.zh-CN.md](./README.zh-CN.md)

## Highlights

- Config center for `~/.codex/config.toml`
- API key management via `~/.codex/.env` (`OPENAI_API_KEY`, masked on read)
- Sandbox and network controls
- Rules visual editor + raw editor + policy playground
- Per-rule check button in editor
- Skills scanner for `~/.codex/skills/**/SKILL.md`
- Markdown preview for `SKILL.md`
- Snapshot create/list/rollback
- Local audit logs
- Built-in i18n: English + Chinese
- PrimeVue UI + Tailwind CSS v4

## Routes

- `/dashboard`
- `/config`
- `/sandbox`
- `/rules`
- `/skills`
- `/snapshots`

## Tech Stack

- Frontend: Vue 3, Vue Router, Pinia, PrimeVue, Tailwind CSS v4, Zod
- Desktop backend: Tauri 2 (Rust)
- Markdown rendering: `marked` + `dompurify`

## Prerequisites

- Node.js 18+
- pnpm 9+
- Rust stable toolchain
- Tauri 2 system prerequisites for your OS
- `codex` CLI available in `PATH` (required for rules check/playground)

## Quick Start

```bash
pnpm install
```

Run web dev (Vite):

```bash
pnpm dev
```

Run desktop app (Tauri):

```bash
pnpm tauri dev
```

Build frontend:

```bash
pnpm build
```

Build desktop bundle:

```bash
pnpm tauri build
```

## Data Paths and Safety Model

This app targets real local Codex files by default.

- Config: `~/.codex/config.toml`
- API key env: `~/.codex/.env`
- Rules: `~/.codex/rules/default.rules`
- Skills root: `~/.codex/skills`

App-owned data:

- Snapshots: `~/.codex-switch-plus/snapshots`
- Backups: `~/.codex-switch-plus/backups`
- Audit logs: `~/.codex-switch-plus/logs`

Write flow:

1. Generate preview diff
2. User confirms
3. Save file
4. Auto backup + audit log

## Core Backend Commands (Tauri Invoke)

- `get_config` / `save_config`
- `get_api_key` / `set_api_key`
- `get_sandbox` / `set_sandbox`
- `probe_network`
- `get_rules` / `save_rules`
- `check_rule_command` / `check_single_rule`
- `scan_skills` / `read_skill`
- `create_snapshot` / `list_snapshots` / `rollback_snapshot`

## Notes

- `danger-full-access` requires explicit confirmation.
- Rule checks call:

```bash
codex execpolicy check --pretty --rules <rules-file> -- <command...>
```

- Skill "open folder/file" uses Tauri opener permissions (`opener:allow-open-path`).

## Troubleshooting

- `codex` command not found:
  - Install Codex CLI and ensure it is in your shell `PATH`.
- Rules playground fails:
  - Verify your rules syntax and local Codex CLI version.
- Cannot open skill path:
  - Check OS-level file access permissions for the app.

## Contributor Guide

### Development Workflow

1. Install dependencies:

```bash
pnpm install
```

2. Start desktop development:

```bash
pnpm tauri dev
```

3. If you only need frontend iteration:

```bash
pnpm dev
```

### Quality Checks Before PR

Run these checks locally before opening a PR:

```bash
pnpm build
```

Optional Rust tests:

```bash
cd src-tauri
cargo test --lib
```

### Project Structure

- `src/`: Vue app (pages, router, UI logic, i18n, API invoke wrapper)
- `src-tauri/`: Rust commands and desktop capabilities
- `docs/`: product docs and planning notes

### PR Checklist

- Keep changes scoped to one goal.
- Add/update i18n keys for user-facing text (`src/i18n/index.ts`).
- Keep write operations safe (preview + confirm flow unchanged).
- Verify both dark mode and light mode UI readability.
- Ensure both Chinese and English experiences remain usable.

### Commit Message

- Use clear English commit messages.
- Keep subject concise and action-oriented.
