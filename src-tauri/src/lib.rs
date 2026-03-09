use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use similar::TextDiff;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use toml::map::Map;
use toml::Value;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelProviderConfig {
    name: String,
    base_url: String,
    wire_api: String,
    requires_openai_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectTrust {
    path: String,
    trust_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexConfig {
    model_provider: String,
    model: String,
    model_reasoning_effort: String,
    network_access: String,
    sandbox_workspace_write_network_access: bool,
    model_providers: BTreeMap<String, ModelProviderConfig>,
    projects: Vec<ProjectTrust>,
    raw_toml: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SavePreview {
    path: String,
    diff: String,
    saved: bool,
    backup_path: Option<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyState {
    exists: bool,
    masked_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SandboxState {
    sandbox_mode: String,
    workspace_write_network_access: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RulesState {
    path: String,
    content: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleCheckResult {
    decision: Option<String>,
    matched_rules: Vec<String>,
    raw_output: String,
    parsed_output: Option<JsonValue>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillMeta {
    name: String,
    description: String,
    path: String,
    skill_md_path: String,
    scripts: Vec<String>,
    dependencies: Vec<String>,
    commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillDetail {
    path: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotFileEntry {
    source_path: String,
    snapshot_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotMeta {
    id: String,
    created_at: String,
    created_by: String,
    summary: String,
    files: Vec<SnapshotFileEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RollbackResult {
    snapshot_id: String,
    restored_files: Vec<String>,
    backup_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkProbeResult {
    dns_ok: bool,
    tcp_ok: bool,
    category: String,
    summary: String,
    latency_ms: Option<u128>,
    error: Option<String>,
}

fn resolve_home_dir() -> Result<PathBuf, String> {
    env::var("HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(PathBuf::from))
        .map_err(|_| "无法解析用户 home 目录".to_string())
}

fn codex_dir() -> Result<PathBuf, String> {
    Ok(resolve_home_dir()?.join(".codex"))
}

fn app_data_dir() -> Result<PathBuf, String> {
    Ok(resolve_home_dir()?.join(".codex-switch-plus"))
}

fn config_path() -> Result<PathBuf, String> {
    Ok(codex_dir()?.join("config.toml"))
}

fn env_path() -> Result<PathBuf, String> {
    Ok(codex_dir()?.join(".env"))
}

fn rules_dir() -> Result<PathBuf, String> {
    Ok(codex_dir()?.join("rules"))
}

fn default_rules_path() -> Result<PathBuf, String> {
    Ok(rules_dir()?.join("default.rules"))
}

fn skills_root() -> Result<PathBuf, String> {
    Ok(codex_dir()?.join("skills"))
}

fn snapshot_root() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("snapshots"))
}

fn logs_root() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("logs"))
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建目录失败 {}: {}", parent.display(), err))?;
    }
    Ok(())
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| format!("创建目录失败 {}: {}", path.display(), err))
}

fn read_text_or_empty(path: &Path) -> Result<String, String> {
    if path.exists() {
        fs::read_to_string(path).map_err(|err| format!("读取文件失败 {}: {}", path.display(), err))
    } else {
        Ok(String::new())
    }
}

fn write_text(path: &Path, content: &str) -> Result<(), String> {
    ensure_parent(path)?;
    fs::write(path, content).map_err(|err| format!("写入文件失败 {}: {}", path.display(), err))
}

fn now_id() -> String {
    Local::now().format("%Y%m%d-%H%M%S-%3f").to_string()
}

fn current_user() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn create_backup(path: &Path, bucket: &str) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let backup_dir = app_data_dir()?.join("backups").join(bucket);
    ensure_dir(&backup_dir)?;

    let name = path
        .file_name()
        .map(|it| it.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup.dat".to_string());

    let backup_path = backup_dir.join(format!("{}-{}.bak", now_id(), name));
    fs::copy(path, &backup_path).map_err(|err| {
        format!(
            "创建备份失败 {} -> {}: {}",
            path.display(),
            backup_path.display(),
            err
        )
    })?;

    Ok(Some(backup_path))
}

fn make_diff(before: &str, after: &str, before_label: &str, after_label: &str) -> String {
    if before == after {
        return "No changes".to_string();
    }

    TextDiff::from_lines(before, after)
        .unified_diff()
        .context_radius(3)
        .header(before_label, after_label)
        .to_string()
}

fn audit(action: &str, detail: JsonValue) -> Result<(), String> {
    let root = logs_root()?;
    ensure_dir(&root)?;
    let log_file = root.join(format!("{}.jsonl", Local::now().format("%Y-%m-%d")));

    ensure_parent(&log_file)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .map_err(|err| format!("打开日志文件失败 {}: {}", log_file.display(), err))?;

    let line = json!({
        "timestamp": Local::now().to_rfc3339(),
        "action": action,
        "detail": detail,
    })
    .to_string();

    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|err| format!("写入日志失败 {}: {}", log_file.display(), err))
}

fn empty_table() -> Value {
    Value::Table(Map::new())
}

fn parse_toml(raw: &str) -> Result<Value, String> {
    if raw.trim().is_empty() {
        Ok(empty_table())
    } else {
        toml::from_str::<Value>(raw).map_err(|err| format!("TOML 解析失败: {}", err))
    }
}

fn read_config_value() -> Result<(String, Value), String> {
    let path = config_path()?;
    let raw = read_text_or_empty(&path)?;
    let value = parse_toml(&raw)?;
    Ok((raw, value))
}

fn as_table_mut(value: &mut Value) -> Result<&mut Map<String, Value>, String> {
    if !value.is_table() {
        *value = empty_table();
    }
    value
        .as_table_mut()
        .ok_or_else(|| "配置根节点不是表结构".to_string())
}

fn ensure_table<'a>(table: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    let needs_init = !matches!(table.get(key), Some(Value::Table(_)));
    if needs_init {
        table.insert(key.to_string(), empty_table());
    }

    table
        .get_mut(key)
        .and_then(Value::as_table_mut)
        .expect("table must exist")
}

fn get_str(value: &Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| default.to_string())
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 4 {
        return "****".to_string();
    }

    let suffix = &key[key.len() - 4..];
    format!("{}{}", "*".repeat(key.len().saturating_sub(4)), suffix)
}

fn parse_api_key_from_env(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("OPENAI_API_KEY=") {
            return None;
        }

        let value = trimmed.trim_start_matches("OPENAI_API_KEY=").trim();
        let unquoted = value.trim_matches('"').trim_matches('\'');
        if unquoted.is_empty() {
            None
        } else {
            Some(unquoted.to_string())
        }
    })
}

fn upsert_api_key_env(content: &str, api_key: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        if line.trim_start().starts_with("OPENAI_API_KEY=") {
            lines.push(format!("OPENAI_API_KEY={}", api_key));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced {
        lines.push(format!("OPENAI_API_KEY={}", api_key));
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn warn_wide_rules(content: &str) -> Vec<String> {
    let block_re = Regex::new(r#"(?s)prefix_rule\s*\((?P<body>.*?)\)"#).expect("regex compile failed");
    let pattern_re = Regex::new(r#"(?s)\bpattern\s*=\s*(?P<pattern>\[[^\]]*\])"#).expect("regex compile failed");
    let decision_re = Regex::new(r#"\bdecision\s*=\s*"(?P<decision>[^"]+)""#).expect("regex compile failed");
    let justification_re =
        Regex::new(r#"\bjustification\s*=\s*"(?P<justification>[^"]+)""#).expect("regex compile failed");
    let match_re = Regex::new(r#"(?s)\bmatch\s*=\s*(?P<match>\[[^\]]*\])"#).expect("regex compile failed");
    let not_match_re =
        Regex::new(r#"(?s)\bnot_match\s*=\s*(?P<not_match>\[[^\]]*\])"#).expect("regex compile failed");
    let token_re = Regex::new(r#""([^"]+)""#).expect("regex compile failed");
    let shell_bins = ["bash", "sh", "zsh", "python", "python3", "node", "npm", "pnpm", "glab", "curl"];
    let shell_wrapper_bins = ["bash", "sh", "zsh", "fish"];

    let mut warnings = Vec::new();
    for block_capture in block_re.captures_iter(content) {
        let Some(whole_block) = block_capture.get(0) else {
            continue;
        };

        let body = block_capture
            .name("body")
            .map(|it| it.as_str())
            .unwrap_or_default();
        let line_no = content[..whole_block.start()].bytes().filter(|byte| *byte == b'\n').count() + 1;

        let Some(pattern_capture) = pattern_re.captures(body) else {
            warnings.push(format!(
                "第 {} 行规则缺少 pattern 字段。参考官方文档：prefix_rule(pattern=[...], ...)",
                line_no
            ));
            continue;
        };

        let pattern_body = pattern_capture
            .name("pattern")
            .map(|it| it.as_str())
            .unwrap_or_default();

        let decision = decision_re
            .captures(body)
            .and_then(|capture| capture.name("decision").map(|it| it.as_str().to_string()))
            .unwrap_or_else(|| "allow".to_string());

        if !matches!(decision.as_str(), "allow" | "prompt" | "forbidden") {
            warnings.push(format!(
                "第 {} 行规则使用了未知 decision=\"{}\"。建议使用 allow/prompt/forbidden。",
                line_no, decision
            ));
        }

        let has_justification = justification_re
            .captures(body)
            .and_then(|capture| capture.name("justification"))
            .map(|it| !it.as_str().trim().is_empty())
            .unwrap_or(false);

        if decision == "forbidden" && !has_justification {
            warnings.push(format!(
                "第 {} 行 forbidden 规则缺少 justification。文档建议为禁止规则添加原因说明。",
                line_no
            ));
        }

        let has_match = match_re.is_match(body);
        let has_not_match = not_match_re.is_match(body);
        if !has_match && !has_not_match {
            warnings.push(format!(
                "第 {} 行规则未提供 match/not_match 示例。建议添加测试样例提升可维护性。",
                line_no
            ));
        }

        let tokens: Vec<String> = token_re
            .captures_iter(pattern_body)
            .filter_map(|capture| capture.get(1).map(|it| it.as_str().to_string()))
            .collect();

        if tokens.is_empty() {
            continue;
        }

        if decision == "allow" {
            if tokens.len() <= 1 {
                warnings.push(format!("第 {} 行规则过宽：仅限制了一级命令 {:?}", line_no, tokens));
                continue;
            }

            if tokens.len() == 2 && shell_bins.contains(&tokens[0].as_str()) && !tokens[1].starts_with('/') {
                warnings.push(format!(
                    "第 {} 行规则可能过宽：shell 命令未限制为绝对脚本路径 {:?}",
                    line_no, tokens
                ));
            }

            if tokens.len() >= 2
                && shell_wrapper_bins.contains(&tokens[0].as_str())
                && matches!(tokens[1].as_str(), "-c" | "-lc")
            {
                warnings.push(format!(
                    "第 {} 行规则使用了 {} {} 包装命令，风险较高。建议直接限制目标可执行命令前缀。",
                    line_no, tokens[0], tokens[1]
                ));
            }
        }
    }

    warnings
}

fn extract_frontmatter_field(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    let first = lines.next()?.trim();
    if first != "---" {
        return None;
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }

        let mut parts = trimmed.splitn(2, ':');
        let field_key = parts.next()?.trim();
        let field_value = parts.next()?.trim();
        if field_key != key {
            continue;
        }

        let value = field_value.trim_matches('"').trim_matches('\'').trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn extract_skill_name(content: &str, fallback: &str) -> String {
    if let Some(name) = extract_frontmatter_field(content, "name") {
        return name;
    }

    content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                Some(trimmed.trim_start_matches('#').trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn extract_skill_description(content: &str) -> String {
    if let Some(description) = extract_frontmatter_field(content, "description") {
        return description;
    }

    let mut in_frontmatter = false;
    let mut frontmatter_seen = false;
    let mut in_code_fence = false;

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();

            if !frontmatter_seen && trimmed == "---" {
                in_frontmatter = !in_frontmatter;
                if !in_frontmatter {
                    frontmatter_seen = true;
                }
                return None;
            }

            if in_frontmatter {
                return None;
            }

            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                return None;
            }

            if in_code_fence {
                return None;
            }

            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') || trimmed.starts_with('*') {
                return None;
            }

            Some(trimmed.to_string())
        })
        .next()
        .unwrap_or_else(|| "No description".to_string())
}

fn collect_dependencies(skill_md: &str, script_files: &[PathBuf]) -> Vec<String> {
    let dep_re = Regex::new(r"\b(glab|python3|python|jq|bash|zsh|sh|node|npm|pnpm|cargo|rustc|codex|curl)\b")
        .expect("regex compile failed");

    let mut deps = BTreeMap::<String, ()>::new();

    for capture in dep_re.captures_iter(skill_md) {
        if let Some(found) = capture.get(1) {
            deps.insert(found.as_str().to_string(), ());
        }
    }

    for file in script_files {
        if let Ok(content) = fs::read_to_string(file) {
            for capture in dep_re.captures_iter(&content) {
                if let Some(found) = capture.get(1) {
                    deps.insert(found.as_str().to_string(), ());
                }
            }
        }
    }

    deps.into_keys().collect()
}

fn validate_sandbox_mode(mode: &str) -> bool {
    matches!(mode, "read-only" | "workspace-write" | "danger-full-access")
}

fn format_matched_rules(parsed: &JsonValue) -> Vec<String> {
    let Some(matched_rules) = parsed.get("matchedRules").and_then(JsonValue::as_array) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for rule in matched_rules {
        if let Some(prefix_match) = rule.get("prefixRuleMatch") {
            let decision = prefix_match
                .get("decision")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown");

            let tokens = prefix_match
                .get("matchedPrefix")
                .and_then(JsonValue::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(JsonValue::as_str)
                        .collect::<Vec<&str>>()
                        .join(" ")
                })
                .unwrap_or_else(|| prefix_match.to_string());

            out.push(format!("{} ({})", tokens, decision));
        } else {
            out.push(rule.to_string());
        }
    }

    out
}

fn codex_config_from_value(raw_toml: String, value: &Value) -> CodexConfig {
    let model_provider = get_str(value, "model_provider", "");
    let model = get_str(value, "model", "");
    let model_reasoning_effort = get_str(value, "model_reasoning_effort", "");
    let network_access = get_str(value, "network_access", "disabled");

    let sandbox_workspace_write_network_access = value
        .get("sandbox_workspace_write")
        .and_then(Value::as_table)
        .and_then(|table| table.get("network_access"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut model_providers = BTreeMap::new();
    if let Some(providers) = value.get("model_providers").and_then(Value::as_table) {
        for (provider_key, provider_value) in providers {
            if let Some(provider_table) = provider_value.as_table() {
                let config = ModelProviderConfig {
                    name: provider_table
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or(provider_key)
                        .to_string(),
                    base_url: provider_table
                        .get("base_url")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    wire_api: provider_table
                        .get("wire_api")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    requires_openai_auth: provider_table
                        .get("requires_openai_auth")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                };
                model_providers.insert(provider_key.to_string(), config);
            }
        }
    }

    let mut projects = Vec::new();
    if let Some(project_table) = value.get("projects").and_then(Value::as_table) {
        for (path, project_value) in project_table {
            let trust_level = project_value
                .as_table()
                .and_then(|project| project.get("trust_level"))
                .and_then(Value::as_str)
                .unwrap_or("untrusted")
                .to_string();

            projects.push(ProjectTrust {
                path: path.to_string(),
                trust_level,
            });
        }
    }

    projects.sort_by(|left, right| left.path.cmp(&right.path));

    CodexConfig {
        model_provider,
        model,
        model_reasoning_effort,
        network_access,
        sandbox_workspace_write_network_access,
        model_providers,
        projects,
        raw_toml,
    }
}

fn preview_or_save(
    path: &Path,
    old_content: &str,
    new_content: &str,
    dry_run: bool,
    backup_bucket: &str,
    warnings: Vec<String>,
    action: &str,
    audit_detail: JsonValue,
) -> Result<SavePreview, String> {
    let diff = make_diff(old_content, new_content, "before", "after");

    if dry_run {
        return Ok(SavePreview {
            path: path.display().to_string(),
            diff,
            saved: false,
            backup_path: None,
            warnings,
        });
    }

    let backup = create_backup(path, backup_bucket)?;
    write_text(path, new_content)?;
    audit(action, audit_detail)?;

    Ok(SavePreview {
        path: path.display().to_string(),
        diff,
        saved: true,
        backup_path: backup.map(|it| it.display().to_string()),
        warnings,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn get_config() -> Result<CodexConfig, String> {
    let (raw, value) = read_config_value()?;
    Ok(codex_config_from_value(raw, &value))
}

#[tauri::command(rename_all = "snake_case")]
fn save_config(config: CodexConfig, dry_run: bool) -> Result<SavePreview, String> {
    let path = config_path()?;
    let (old_raw, mut value) = read_config_value()?;
    let root = as_table_mut(&mut value)?;

    root.insert("model_provider".to_string(), Value::String(config.model_provider.clone()));
    root.insert("model".to_string(), Value::String(config.model.clone()));
    root.insert(
        "model_reasoning_effort".to_string(),
        Value::String(config.model_reasoning_effort.clone()),
    );
    root.insert("network_access".to_string(), Value::String(config.network_access.clone()));

    let sandbox_workspace_write = ensure_table(root, "sandbox_workspace_write");
    sandbox_workspace_write.insert(
        "network_access".to_string(),
        Value::Boolean(config.sandbox_workspace_write_network_access),
    );

    let mut provider_table = Map::new();
    for (provider_name, provider) in &config.model_providers {
        let mut provider_value = Map::new();
        provider_value.insert("name".to_string(), Value::String(provider.name.clone()));
        provider_value.insert("base_url".to_string(), Value::String(provider.base_url.clone()));
        provider_value.insert("wire_api".to_string(), Value::String(provider.wire_api.clone()));
        provider_value.insert(
            "requires_openai_auth".to_string(),
            Value::Boolean(provider.requires_openai_auth),
        );
        provider_table.insert(provider_name.to_string(), Value::Table(provider_value));
    }
    root.insert("model_providers".to_string(), Value::Table(provider_table));

    let mut projects = Map::new();
    for project in &config.projects {
        let mut project_table = Map::new();
        project_table.insert("trust_level".to_string(), Value::String(project.trust_level.clone()));
        projects.insert(project.path.clone(), Value::Table(project_table));
    }
    root.insert("projects".to_string(), Value::Table(projects));

    let new_raw = toml::to_string_pretty(&value).map_err(|err| format!("生成 TOML 失败: {}", err))?;
    preview_or_save(
        &path,
        &old_raw,
        &new_raw,
        dry_run,
        "config",
        Vec::new(),
        "save_config",
        json!({
            "path": path.display().to_string(),
            "dryRun": dry_run,
            "modelProvider": config.model_provider,
            "model": config.model,
        }),
    )
}

#[tauri::command(rename_all = "snake_case")]
fn get_api_key() -> Result<ApiKeyState, String> {
    let path = env_path()?;
    let content = read_text_or_empty(&path)?;
    let key = parse_api_key_from_env(&content);

    Ok(ApiKeyState {
        exists: key.is_some(),
        masked_value: key.as_deref().map(mask_api_key),
    })
}

#[tauri::command(rename_all = "snake_case")]
fn set_api_key(api_key: String, dry_run: bool) -> Result<SavePreview, String> {
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    let path = env_path()?;
    let old_content = read_text_or_empty(&path)?;
    let new_content = upsert_api_key_env(&old_content, api_key.trim());

    preview_or_save(
        &path,
        &old_content,
        &new_content,
        dry_run,
        "api-key",
        Vec::new(),
        "set_api_key",
        json!({
            "path": path.display().to_string(),
            "dryRun": dry_run,
            "masked": mask_api_key(api_key.trim()),
        }),
    )
}

#[tauri::command(rename_all = "snake_case")]
fn get_sandbox() -> Result<SandboxState, String> {
    let (_, value) = read_config_value()?;

    let sandbox_mode = value
        .get("sandbox_mode")
        .and_then(Value::as_str)
        .unwrap_or("workspace-write")
        .to_string();

    let workspace_write_network_access = value
        .get("sandbox_workspace_write")
        .and_then(Value::as_table)
        .and_then(|table| table.get("network_access"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(SandboxState {
        sandbox_mode,
        workspace_write_network_access,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn set_sandbox(
    sandbox_mode: String,
    workspace_write_network_access: bool,
    high_risk_confirmed: bool,
    dry_run: bool,
) -> Result<SavePreview, String> {
    if !validate_sandbox_mode(&sandbox_mode) {
        return Err(format!("非法 sandbox 模式: {}", sandbox_mode));
    }

    if sandbox_mode == "danger-full-access" && !high_risk_confirmed {
        return Err("danger-full-access 需要二次确认".to_string());
    }

    let path = config_path()?;
    let (old_raw, mut value) = read_config_value()?;
    let root = as_table_mut(&mut value)?;

    root.insert("sandbox_mode".to_string(), Value::String(sandbox_mode.clone()));

    let sandbox_workspace_write = ensure_table(root, "sandbox_workspace_write");
    sandbox_workspace_write.insert(
        "network_access".to_string(),
        Value::Boolean(workspace_write_network_access),
    );

    let new_raw = toml::to_string_pretty(&value).map_err(|err| format!("生成 TOML 失败: {}", err))?;

    preview_or_save(
        &path,
        &old_raw,
        &new_raw,
        dry_run,
        "sandbox",
        Vec::new(),
        "set_sandbox",
        json!({
            "path": path.display().to_string(),
            "dryRun": dry_run,
            "sandboxMode": sandbox_mode,
            "workspaceWriteNetworkAccess": workspace_write_network_access,
        }),
    )
}

#[tauri::command(rename_all = "snake_case")]
fn probe_network() -> Result<NetworkProbeResult, String> {
    let started = Instant::now();
    let target = "api.github.com:443";

    let addrs = target
        .to_socket_addrs()
        .map_err(|err| format!("DNS 解析失败: {}", err))?;

    let Some(addr) = addrs.into_iter().next() else {
        return Ok(NetworkProbeResult {
            dns_ok: false,
            tcp_ok: false,
            category: "dns".to_string(),
            summary: "DNS 未返回可用地址".to_string(),
            latency_ms: None,
            error: Some("无可用地址".to_string()),
        });
    };

    match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
        Ok(_) => {
            let latency_ms = started.elapsed().as_millis();
            let result = NetworkProbeResult {
                dns_ok: true,
                tcp_ok: true,
                category: "ok".to_string(),
                summary: format!("网络探测成功，{} 可连通", target),
                latency_ms: Some(latency_ms),
                error: None,
            };
            audit(
                "probe_network",
                json!({"category": result.category, "latencyMs": result.latency_ms}),
            )?;
            Ok(result)
        }
        Err(err) => {
            let result = NetworkProbeResult {
                dns_ok: true,
                tcp_ok: false,
                category: "tcp".to_string(),
                summary: format!("DNS 正常，但 TCP 连接失败：{}", err),
                latency_ms: None,
                error: Some(err.to_string()),
            };
            audit(
                "probe_network",
                json!({"category": result.category, "error": result.error}),
            )?;
            Ok(result)
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_rules() -> Result<RulesState, String> {
    let path = default_rules_path()?;
    let content = read_text_or_empty(&path)?;
    let warnings = warn_wide_rules(&content);

    Ok(RulesState {
        path: path.display().to_string(),
        content,
        warnings,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn save_rules(rules_content: String, dry_run: bool) -> Result<SavePreview, String> {
    let path = default_rules_path()?;
    let old_content = read_text_or_empty(&path)?;

    let mut normalized_content = rules_content;
    if !normalized_content.ends_with('\n') {
        normalized_content.push('\n');
    }

    let warnings = warn_wide_rules(&normalized_content);

    preview_or_save(
        &path,
        &old_content,
        &normalized_content,
        dry_run,
        "rules",
        warnings,
        "save_rules",
        json!({
            "path": path.display().to_string(),
            "dryRun": dry_run,
        }),
    )
}

fn run_execpolicy_check(rules_path: &Path, command: &str) -> Result<RuleCheckResult, String> {
    let command = command.trim();
    if command.is_empty() {
        return Err("命令不能为空".to_string());
    }

    let tokens = shlex::split(command).ok_or_else(|| "命令解析失败，请检查引号".to_string())?;
    if tokens.is_empty() {
        return Err("未解析出可执行命令".to_string());
    }

    let output = Command::new("codex")
        .arg("execpolicy")
        .arg("check")
        .arg("--pretty")
        .arg("--rules")
        .arg(rules_path)
        .arg("--")
        .args(&tokens)
        .output()
        .map_err(|err| format!("执行 codex execpolicy check 失败: {}", err))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let parsed_output = serde_json::from_str::<JsonValue>(&stdout).ok();
    let decision = parsed_output
        .as_ref()
        .and_then(|parsed| parsed.get("decision"))
        .and_then(JsonValue::as_str)
        .map(ToString::to_string);

    let matched_rules = parsed_output
        .as_ref()
        .map(format_matched_rules)
        .unwrap_or_default();

    let error = if output.status.success() {
        if stderr.trim().is_empty() {
            None
        } else {
            Some(stderr)
        }
    } else if stderr.trim().is_empty() {
        Some("execpolicy 返回非零退出码".to_string())
    } else {
        Some(stderr)
    };

    Ok(RuleCheckResult {
        decision,
        matched_rules,
        raw_output: if stdout.trim().is_empty() { "(empty stdout)".to_string() } else { stdout },
        parsed_output,
        error,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn check_rule_command(command: String) -> Result<RuleCheckResult, String> {
    let rules_path = default_rules_path()?;
    let result = run_execpolicy_check(&rules_path, &command)?;

    audit(
        "check_rule_command",
        json!({
            "command": command,
            "decision": result.decision,
            "success": result.error.is_none(),
        }),
    )?;

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
fn check_single_rule(rule_content: String, command: String) -> Result<RuleCheckResult, String> {
    let normalized_rule = rule_content.trim();
    if normalized_rule.is_empty() {
        return Err("规则内容不能为空".to_string());
    }

    let rule_check_dir = env::temp_dir().join("codex-switch-plus");
    ensure_dir(&rule_check_dir)?;

    let temp_rule_path = rule_check_dir.join(format!("rule-check-{}.rules", now_id()));
    let file_content = format!("{}\n", normalized_rule);
    write_text(&temp_rule_path, &file_content)?;

    let check_result = run_execpolicy_check(&temp_rule_path, &command);
    let _ = fs::remove_file(&temp_rule_path);

    let result = check_result?;
    audit(
        "check_single_rule",
        json!({
            "command": command,
            "decision": result.decision,
            "success": result.error.is_none(),
        }),
    )?;

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
fn scan_skills() -> Result<Vec<SkillMeta>, String> {
    let root = skills_root()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        if entry.file_name() != "SKILL.md" {
            continue;
        }

        let skill_md_path = entry.path().to_path_buf();
        let Some(skill_dir) = skill_md_path.parent() else {
            continue;
        };

        let content = fs::read_to_string(&skill_md_path)
            .map_err(|err| format!("读取 SKILL.md 失败 {}: {}", skill_md_path.display(), err))?;

        let mut scripts = Vec::new();
        let mut script_files = Vec::new();
        let scripts_dir = skill_dir.join("scripts");
        if scripts_dir.exists() {
            for script_entry in WalkDir::new(&scripts_dir)
                .follow_links(false)
                .min_depth(1)
                .max_depth(3)
                .into_iter()
                .filter_map(Result::ok)
            {
                if script_entry.file_type().is_file() {
                    let full_path = script_entry.path().to_path_buf();
                    script_files.push(full_path.clone());
                    if let Ok(relative) = full_path.strip_prefix(skill_dir) {
                        scripts.push(relative.display().to_string());
                    }
                }
            }
            scripts.sort();
        }

        let skill_name_fallback = skill_dir
            .file_name()
            .map(|it| it.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown-skill".to_string());
        let name = extract_skill_name(&content, &skill_name_fallback);
        let description = extract_skill_description(&content);
        let dependencies = collect_dependencies(&content, &script_files);
        let commands = dependencies.clone();

        out.push(SkillMeta {
            name,
            description,
            path: skill_dir.display().to_string(),
            skill_md_path: skill_md_path.display().to_string(),
            scripts,
            dependencies,
            commands,
        });
    }

    out.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(out)
}

#[tauri::command(rename_all = "snake_case")]
fn read_skill(skill_md_path: String) -> Result<SkillDetail, String> {
    let root = skills_root()?;
    let root = fs::canonicalize(&root)
        .map_err(|err| format!("技能目录不存在或无权限 {}: {}", root.display(), err))?;

    let skill_path = PathBuf::from(skill_md_path);
    let canonical_skill_path = fs::canonicalize(&skill_path)
        .map_err(|err| format!("无法读取技能文件 {}: {}", skill_path.display(), err))?;

    if !canonical_skill_path.starts_with(&root) {
        return Err("非法路径：仅允许读取 ~/.codex/skills 下的 SKILL.md".to_string());
    }

    let content = fs::read_to_string(&canonical_skill_path)
        .map_err(|err| format!("读取文件失败 {}: {}", canonical_skill_path.display(), err))?;

    Ok(SkillDetail {
        path: canonical_skill_path.display().to_string(),
        content,
    })
}

#[tauri::command(rename_all = "snake_case")]
fn create_snapshot(summary: Option<String>) -> Result<SnapshotMeta, String> {
    let snapshot_root = snapshot_root()?;
    ensure_dir(&snapshot_root)?;

    let id = now_id();
    let snapshot_dir = snapshot_root.join(&id);
    ensure_dir(&snapshot_dir)?;

    let mut files = Vec::new();

    let cfg_path = config_path()?;
    if cfg_path.exists() {
        let dest = snapshot_dir.join("config.toml");
        ensure_parent(&dest)?;
        fs::copy(&cfg_path, &dest).map_err(|err| {
            format!(
                "复制配置文件失败 {} -> {}: {}",
                cfg_path.display(),
                dest.display(),
                err
            )
        })?;
        files.push(SnapshotFileEntry {
            source_path: cfg_path.display().to_string(),
            snapshot_path: "config.toml".to_string(),
        });
    }

    let rules_root = rules_dir()?;
    if rules_root.exists() {
        for entry in fs::read_dir(&rules_root)
            .map_err(|err| format!("读取 rules 目录失败 {}: {}", rules_root.display(), err))?
        {
            let entry = entry.map_err(|err| format!("读取目录项失败: {}", err))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(ext) = path.extension() else {
                continue;
            };
            if ext != "rules" {
                continue;
            }

            let file_name = path
                .file_name()
                .map(|it| it.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown.rules".to_string());

            let relative_snapshot_path = format!("rules/{}", file_name);
            let dest = snapshot_dir.join(&relative_snapshot_path);
            ensure_parent(&dest)?;
            fs::copy(&path, &dest).map_err(|err| {
                format!(
                    "复制规则文件失败 {} -> {}: {}",
                    path.display(),
                    dest.display(),
                    err
                )
            })?;

            files.push(SnapshotFileEntry {
                source_path: path.display().to_string(),
                snapshot_path: relative_snapshot_path,
            });
        }
    }

    let meta = SnapshotMeta {
        id: id.clone(),
        created_at: Local::now().to_rfc3339(),
        created_by: current_user(),
        summary: summary.unwrap_or_else(|| "手动快照".to_string()),
        files,
    };

    let metadata_path = snapshot_dir.join("metadata.json");
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|err| format!("序列化快照元数据失败: {}", err))?;
    write_text(&metadata_path, &meta_json)?;

    audit(
        "create_snapshot",
        json!({
            "snapshotId": id,
            "fileCount": meta.files.len(),
            "summary": meta.summary,
        }),
    )?;

    Ok(meta)
}

#[tauri::command(rename_all = "snake_case")]
fn list_snapshots() -> Result<Vec<SnapshotMeta>, String> {
    let root = snapshot_root()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();

    for entry in fs::read_dir(&root).map_err(|err| format!("读取快照目录失败 {}: {}", root.display(), err))? {
        let entry = entry.map_err(|err| format!("读取快照目录项失败: {}", err))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let metadata_path = path.join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&metadata_path)
            .map_err(|err| format!("读取快照元数据失败 {}: {}", metadata_path.display(), err))?;

        if let Ok(meta) = serde_json::from_str::<SnapshotMeta>(&content) {
            snapshots.push(meta);
        }
    }

    snapshots.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(snapshots)
}

#[tauri::command(rename_all = "snake_case")]
fn rollback_snapshot(snapshot_id: String) -> Result<RollbackResult, String> {
    let root = snapshot_root()?;
    let snapshot_dir = root.join(&snapshot_id);
    let metadata_path = snapshot_dir.join("metadata.json");

    if !metadata_path.exists() {
        return Err(format!("快照不存在: {}", snapshot_id));
    }

    let metadata_content = fs::read_to_string(&metadata_path)
        .map_err(|err| format!("读取快照元数据失败 {}: {}", metadata_path.display(), err))?;
    let metadata: SnapshotMeta = serde_json::from_str(&metadata_content)
        .map_err(|err| format!("解析快照元数据失败 {}: {}", metadata_path.display(), err))?;

    let mut restored_files = Vec::new();
    let mut backup_paths = Vec::new();

    for file in &metadata.files {
        let source_path = PathBuf::from(&file.source_path);
        let snapshot_file_path = snapshot_dir.join(&file.snapshot_path);

        if !snapshot_file_path.exists() {
            continue;
        }

        if let Some(backup) = create_backup(&source_path, "rollback")? {
            backup_paths.push(backup.display().to_string());
        }

        ensure_parent(&source_path)?;
        fs::copy(&snapshot_file_path, &source_path).map_err(|err| {
            format!(
                "回滚文件失败 {} -> {}: {}",
                snapshot_file_path.display(),
                source_path.display(),
                err
            )
        })?;

        restored_files.push(source_path.display().to_string());
    }

    audit(
        "rollback_snapshot",
        json!({
            "snapshotId": snapshot_id,
            "restoredFiles": restored_files,
        }),
    )?;

    Ok(RollbackResult {
        snapshot_id,
        restored_files,
        backup_paths,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_api_key,
            set_api_key,
            get_sandbox,
            set_sandbox,
            probe_network,
            get_rules,
            save_rules,
            check_rule_command,
            check_single_rule,
            scan_skills,
            read_skill,
            create_snapshot,
            list_snapshots,
            rollback_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{
        extract_skill_description, extract_skill_name, mask_api_key, parse_api_key_from_env,
        upsert_api_key_env, warn_wide_rules,
    };

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("abcd"), "****");
        assert_eq!(mask_api_key("abcdefgh"), "****efgh");
    }

    #[test]
    fn test_parse_api_key() {
        let env = "OPENAI_API_KEY=test-key\nOTHER=1\n";
        assert_eq!(parse_api_key_from_env(env), Some("test-key".to_string()));
    }

    #[test]
    fn test_upsert_api_key() {
        let env = "FOO=bar\nOPENAI_API_KEY=old\n";
        let updated = upsert_api_key_env(env, "new");
        assert!(updated.contains("OPENAI_API_KEY=new"));
        assert!(updated.ends_with('\n'));
    }

    #[test]
    fn test_warn_wide_rules() {
        let rules = r#"
            prefix_rule(pattern=["glab"], decision="allow")
            prefix_rule(pattern=["bash", "scripts/run.sh"], decision="allow")
        "#;
        let warnings = warn_wide_rules(rules);
        assert!(warnings.iter().any(|warning| warning.contains("仅限制了一级命令")));
        assert!(warnings.iter().any(|warning| warning.contains("未限制为绝对脚本路径")));
    }

    #[test]
    fn test_warn_forbidden_without_justification() {
        let rules = r#"
            prefix_rule(
                pattern=["rm", "-rf"],
                decision="forbidden"
            )
        "#;
        let warnings = warn_wide_rules(rules);
        assert!(warnings.iter().any(|warning| warning.contains("forbidden 规则缺少 justification")));
    }

    #[test]
    fn test_warn_unknown_decision() {
        let rules = r#"
            prefix_rule(
                pattern=["gh", "pr", "view"],
                decision="deny"
            )
        "#;
        let warnings = warn_wide_rules(rules);
        assert!(warnings.iter().any(|warning| warning.contains("未知 decision")));
    }

    #[test]
    fn test_extract_skill_frontmatter() {
        let content = r#"---
name: test-skill
description: this is test description
---

# Test Skill
"#;
        assert_eq!(extract_skill_name(content, "fallback"), "test-skill");
        assert_eq!(extract_skill_description(content), "this is test description");
    }
}
