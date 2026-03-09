import { computed, ref } from "vue";

type Locale = "zh-CN" | "en-US";
type MessageValue = string | MessageDict;

interface MessageDict {
  [key: string]: MessageValue;
}

const STORAGE_KEY = "codex-switch-plus.locale";

const messages: Record<Locale, MessageDict> = {
  "zh-CN": {
    common: {
      refresh: "刷新",
      cancel: "取消",
      confirmSave: "确认保存",
      notice: "提示",
      ok: "OK",
      fail: "失败",
      saveSuccess: "保存成功",
      saveFailed: "保存失败",
      loadFailed: "加载失败",
      previewFailed: "预览失败",
      check: "检查",
      noData: "暂无数据",
      targetFile: "目标文件",
      savePreview: "保存前预览",
      riskTips: "风险提示",
    },
    app: {
      title: "Codex Switch+",
      slogan: "One panel to control Codex",
      language: "语言",
      languageZh: "中文",
      languageEn: "English",
      nav: {
        dashboard: "仪表盘",
        config: "配置中心",
        sandbox: "Sandbox/网络",
        rules: "Rules",
        skills: "Skills",
        snapshots: "快照中心",
      },
    },
    dashboard: {
      title: "仪表盘",
      subtitle: "Codex 核心状态总览",
      modelConfig: "模型配置",
      provider: "Provider",
      model: "Model",
      sandboxNetwork: "Sandbox / 网络",
      workspaceWriteNetwork: "workspace-write network_access",
      resources: "资源数量",
      enabled: "已开启",
      disabled: "已关闭",
      rulesCount: "Rules 行数",
      skillsCount: "Skills",
      snapshotsCount: "Snapshots",
      gotoConfig: "去配置中心",
      gotoRules: "去 Rules 调试",
      gotoSkills: "去 Skills",
    },
    config: {
      title: "配置中心",
      subtitle: "编辑 ~/.codex/config.toml 与 OPENAI_API_KEY",
      baseConfig: "基础配置",
      loading: "正在加载...",
      addProvider: "新增 Provider",
      currentProviderKey: "当前 Provider Key",
      delete: "删除",
      projects: "projects",
      addProject: "新增项目",
      validationErrors: "校验错误",
      previewAndSaveConfig: "预览 diff 并保存配置",
      apiKey: "API Key",
      currentMasked: "当前值（脱敏）",
      notSet: "未设置",
      newApiKey: "新的 OPENAI_API_KEY",
      previewAndSave: "预览并保存",
      saveConfigSuccess: "配置已写入 config.toml",
      saveApiKeySuccess: "API Key 已更新",
      needApiKeyInput: "请输入新的 API Key",
      apiKeyPlaceholder: "请输入 sk-...",
      modelProviderEmpty: "model_provider 不能为空",
      modelEmpty: "model 不能为空",
      reasoningEmpty: "model_reasoning_effort 不能为空",
      networkEmpty: "network_access 不能为空",
      providerNameEmpty: "provider.name 不能为空",
      providerBaseUrlInvalid: "provider.base_url 必须是合法 URL",
      providerWireApiEmpty: "provider.wire_api 不能为空",
      configNotLoaded: "配置尚未加载",
      needOneProvider: "至少需要一个 model provider",
      projectPathEmpty: "项目 #{index} path 不能为空",
      providerNotFound: "model_provider 必须对应一个已存在的 provider key",
    },
    sandbox: {
      title: "Sandbox / 网络",
      subtitle: "切换 sandbox 模式，验证 network_access 可用性",
      modeSwitch: "模式切换",
      sandboxMode: "sandbox_mode",
      workspaceWriteNetwork: "workspace-write.network_access",
      commonPitfall: "常见坑提示：即使 rules 已放行，若 network_access 关闭，联网命令依然会失败。",
      previewAndSave: "预览 diff 并保存",
      probeTitle: "网络探测",
      runProbe: "执行探测",
      probeTarget: "检测 DNS + TCP 443（目标：api.github.com）",
      probeDone: "探测完成",
      probeFailed: "探测失败",
      saveSuccess: "Sandbox 配置已更新",
      dangerHeader: "高风险操作确认",
      dangerMessage: "你将启用 danger-full-access。该模式可绕过大部分隔离限制，确认继续？",
      dangerAccept: "确认启用",
      result: "结果",
      category: "分类",
      dns: "DNS",
      tcp: "TCP",
      latency: "延迟",
      error: "错误",
    },
    rules: {
      title: "Rules 管理",
      subtitle: "逐条可视化编辑规则，decision 使用下拉选择，支持 hover 风险告警",
      visualMode: "可视化逐条编辑",
      rawMode: "原始文本编辑",
      visualModeTag: "可视化模式",
      rawModeTag: "原始文本模式",
      editorMode: "编辑模式",
      filePath: "文件路径",
      perRuleEditor: "逐条规则编辑器",
      addRule: "新增规则",
      syncToRaw: "同步到原始文本",
      rawToVisual: "用原始文本覆盖可视化",
      insertTemplate: "插入模板规则",
      patternLabel: "pattern（支持 union：cmd a | cmd b）",
      patternPlaceholder: "例如: gh pr view | glab issue view",
      decisionLabel: "decision",
      justificationLabel: "justification",
      justificationPlaceholder: "forbidden / prompt 建议填写原因",
      matchLabel: "match（每行一条）",
      notMatchLabel: "not_match（每行一条）",
      generatedPreview: "可视化模式生成预览",
      rawEditor: "原始规则文本",
      globalWarnings: "全局风险提示",
      noWarnings: "暂无全局告警",
      previewAndSave: "预览 diff 并保存",
      playground: "Rules Playground",
      commandPlaceholder: "输入命令，例如 gh pr view 7888",
      runCheck: "执行 check",
      matchedRules: "matched rules",
      noMatchedRules: "无命中规则",
      ruleTag: "Rule #{index}",
      checkRuleTooltip: "检查当前规则（优先使用 match 第一行）",
      syncDone: "可视化内容已同步到原始文本",
      parsedDone: "原始文本已转换为逐条规则",
      needCommand: "请输入要测试的命令",
      checkFailed: "调试失败",
      cannotCheck: "无法检查",
      needPatternForCheck: "请先填写 pattern，或提供至少一条 match 示例",
      ruleCheckFailed: "规则检查失败",
      ruleCheckPassed: "规则检查通过",
      checkMessage: "命令: {command}，decision: {decision}",
      saveDetail: "default.rules 已更新",
      warningPatternEmpty: "pattern 为空，这条规则不会生效。",
      warningDecisionInvalid: "decision 仅支持 allow / prompt / forbidden。",
      warningNeedJustification: "forbidden 建议填写 justification，说明阻断原因。",
      warningNeedExamples: "建议至少填写 match 或 not_match 示例，便于维护和调试。",
      warningAllowTooWide: "allow 规则过宽：仅限制了一级命令 ({tokens})",
      warningNeedAbsolutePath: "shell 命令建议限制为绝对路径脚本：{tokens}",
      warningWrapperRisk: "检测到 {cmd} {flag} 包装执行，风险较高。",
      templatePrompt: "Prompt 审批规则",
      templateForbidden: "Forbidden 阻断规则",
      templateAllow: "Allow 最小权限规则",
      templateUnion: "Union 模式规则",
      templatePrefix: "模板: {label}",
      empty: "(empty)",
    },
    skills: {
      title: "Skills 管理",
      subtitle: "扫描 ~/.codex/skills/**/SKILL.md 并展示元信息与依赖",
      name: "名称",
      description: "描述",
      dependenciesHeader: "依赖",
      openDir: "打开目录",
      openSkillFile: "打开 SKILL.md",
      scripts: "脚本入口",
      dependencies: "命令依赖",
      noScripts: "未发现 scripts/ 入口",
      chooseSkill: "请选择一个 Skill",
      loadFailed: "加载失败",
      readFailed: "读取失败",
      openDirFailed: "打开目录失败",
      openFileFailed: "打开文件失败",
      noContent: "暂无内容",
      noDescription: "(No description)",
    },
    snapshots: {
      title: "快照中心",
      subtitle: "管理 ~/.codex-switch-plus/snapshots，支持创建与回滚",
      createSnapshot: "创建快照",
      create: "创建",
      createSuccess: "快照已写入本地目录",
      createFailed: "创建失败",
      list: "快照列表",
      summaryLabel: "变更摘要（可选）",
      summaryPlaceholder: "例如：切换 provider + 新增 rules",
      rollback: "回滚",
      rollbackDone: "回滚完成",
      rollbackDoneDetail: "恢复 {count} 个文件",
      rollbackFailed: "回滚失败",
      rollbackHeader: "回滚确认",
      rollbackMessage: "确认回滚到快照 {id}？将覆盖当前 config/rules，并自动备份现状。",
      id: "ID",
      createdAt: "创建时间",
      createdBy: "创建人",
      summary: "摘要",
      fileCount: "文件数",
      actions: "操作",
    },
  },
  "en-US": {
    common: {
      refresh: "Refresh",
      cancel: "Cancel",
      confirmSave: "Confirm Save",
      notice: "Notice",
      ok: "OK",
      fail: "FAIL",
      saveSuccess: "Saved",
      saveFailed: "Save Failed",
      loadFailed: "Load Failed",
      previewFailed: "Preview Failed",
      check: "Check",
      noData: "No data",
      targetFile: "Target File",
      savePreview: "Save Preview",
      riskTips: "Risk Warnings",
    },
    app: {
      title: "Codex Switch+",
      slogan: "One panel to control Codex",
      language: "Language",
      languageZh: "中文",
      languageEn: "English",
      nav: {
        dashboard: "Dashboard",
        config: "Config",
        sandbox: "Sandbox/Network",
        rules: "Rules",
        skills: "Skills",
        snapshots: "Snapshots",
      },
    },
    dashboard: {
      title: "Dashboard",
      subtitle: "Overview of Codex core status",
      modelConfig: "Model Config",
      provider: "Provider",
      model: "Model",
      sandboxNetwork: "Sandbox / Network",
      workspaceWriteNetwork: "workspace-write network_access",
      resources: "Resources",
      enabled: "Enabled",
      disabled: "Disabled",
      rulesCount: "Rules Lines",
      skillsCount: "Skills",
      snapshotsCount: "Snapshots",
      gotoConfig: "Go to Config",
      gotoRules: "Go to Rules",
      gotoSkills: "Go to Skills",
    },
    config: {
      title: "Config",
      subtitle: "Edit ~/.codex/config.toml and OPENAI_API_KEY",
      baseConfig: "Base Config",
      loading: "Loading...",
      addProvider: "Add Provider",
      currentProviderKey: "Current Provider Key",
      delete: "Delete",
      projects: "projects",
      addProject: "Add Project",
      validationErrors: "Validation Errors",
      previewAndSaveConfig: "Preview Diff & Save Config",
      apiKey: "API Key",
      currentMasked: "Current (masked)",
      notSet: "Not set",
      newApiKey: "New OPENAI_API_KEY",
      previewAndSave: "Preview & Save",
      saveConfigSuccess: "Config saved to config.toml",
      saveApiKeySuccess: "API Key updated",
      needApiKeyInput: "Please input a new API Key",
      apiKeyPlaceholder: "Type sk-...",
      modelProviderEmpty: "model_provider is required",
      modelEmpty: "model is required",
      reasoningEmpty: "model_reasoning_effort is required",
      networkEmpty: "network_access is required",
      providerNameEmpty: "provider.name is required",
      providerBaseUrlInvalid: "provider.base_url must be a valid URL",
      providerWireApiEmpty: "provider.wire_api is required",
      configNotLoaded: "Config is not loaded",
      needOneProvider: "At least one model provider is required",
      projectPathEmpty: "Project #{index} path is required",
      providerNotFound: "model_provider must match an existing provider key",
    },
    sandbox: {
      title: "Sandbox / Network",
      subtitle: "Switch sandbox mode and verify network_access",
      modeSwitch: "Mode",
      sandboxMode: "sandbox_mode",
      workspaceWriteNetwork: "workspace-write.network_access",
      commonPitfall: "Common pitfall: even if rules allow a command, it still fails when network_access is disabled.",
      previewAndSave: "Preview Diff & Save",
      probeTitle: "Network Probe",
      runProbe: "Run Probe",
      probeTarget: "Probe DNS + TCP 443 (target: api.github.com)",
      probeDone: "Probe Completed",
      probeFailed: "Probe Failed",
      saveSuccess: "Sandbox config updated",
      dangerHeader: "High-risk Confirmation",
      dangerMessage: "You are enabling danger-full-access. This bypasses most sandbox boundaries. Continue?",
      dangerAccept: "Enable",
      result: "Result",
      category: "Category",
      dns: "DNS",
      tcp: "TCP",
      latency: "Latency",
      error: "Error",
    },
    rules: {
      title: "Rules",
      subtitle: "Edit rules line-by-line with decision dropdown and hover warnings",
      visualMode: "Visual Rule Editor",
      rawMode: "Raw Text Editor",
      visualModeTag: "Visual Mode",
      rawModeTag: "Raw Mode",
      editorMode: "Editor Mode",
      filePath: "File Path",
      perRuleEditor: "Per-rule Editor",
      addRule: "Add Rule",
      syncToRaw: "Sync to Raw",
      rawToVisual: "Parse Raw to Visual",
      insertTemplate: "Insert Template",
      patternLabel: "pattern (union supported: cmd a | cmd b)",
      patternPlaceholder: "e.g. gh pr view | glab issue view",
      decisionLabel: "decision",
      justificationLabel: "justification",
      justificationPlaceholder: "recommended for forbidden/prompt",
      matchLabel: "match (one per line)",
      notMatchLabel: "not_match (one per line)",
      generatedPreview: "Generated Preview",
      rawEditor: "Raw Rule Text",
      globalWarnings: "Global Warnings",
      noWarnings: "No global warnings",
      previewAndSave: "Preview Diff & Save",
      playground: "Rules Playground",
      commandPlaceholder: "Type a command, e.g. gh pr view 7888",
      runCheck: "Run Check",
      matchedRules: "matched rules",
      noMatchedRules: "No matched rules",
      ruleTag: "Rule #{index}",
      checkRuleTooltip: "Check this rule (prefer first match line)",
      syncDone: "Visual content synced to raw text",
      parsedDone: "Raw text parsed to visual rules",
      needCommand: "Please input a command",
      checkFailed: "Check Failed",
      cannotCheck: "Cannot Check",
      needPatternForCheck: "Please fill pattern or add at least one match example",
      ruleCheckFailed: "Rule Check Failed",
      ruleCheckPassed: "Rule Check Passed",
      checkMessage: "Command: {command}, decision: {decision}",
      saveDetail: "default.rules has been updated",
      warningPatternEmpty: "pattern is empty; this rule will not work.",
      warningDecisionInvalid: "decision must be allow / prompt / forbidden.",
      warningNeedJustification: "forbidden should include a justification.",
      warningNeedExamples: "Add match or not_match examples for maintainability.",
      warningAllowTooWide: "allow rule is too broad: only top-level command is constrained ({tokens})",
      warningNeedAbsolutePath: "shell command should use absolute script path: {tokens}",
      warningWrapperRisk: "{cmd} {flag} wrapper execution detected; this is high risk.",
      templatePrompt: "Prompt Rule",
      templateForbidden: "Forbidden Rule",
      templateAllow: "Minimal Allow Rule",
      templateUnion: "Union Pattern Rule",
      templatePrefix: "Template: {label}",
      empty: "(empty)",
    },
    skills: {
      title: "Skills",
      subtitle: "Scan ~/.codex/skills/**/SKILL.md and show metadata",
      name: "Name",
      description: "Description",
      dependenciesHeader: "Dependencies",
      openDir: "Open Folder",
      openSkillFile: "Open SKILL.md",
      scripts: "Scripts",
      dependencies: "Dependencies",
      noScripts: "No scripts/ entries",
      chooseSkill: "Choose a skill",
      loadFailed: "Load Failed",
      readFailed: "Read Failed",
      openDirFailed: "Open Folder Failed",
      openFileFailed: "Open File Failed",
      noContent: "No content",
      noDescription: "(No description)",
    },
    snapshots: {
      title: "Snapshots",
      subtitle: "Manage ~/.codex-switch-plus/snapshots with create and rollback",
      createSnapshot: "Create Snapshot",
      create: "Create",
      createSuccess: "Snapshot has been created",
      createFailed: "Create Failed",
      list: "Snapshot List",
      summaryLabel: "Summary (optional)",
      summaryPlaceholder: "e.g. switch provider + update rules",
      rollback: "Rollback",
      rollbackDone: "Rollback Completed",
      rollbackDoneDetail: "Restored {count} files",
      rollbackFailed: "Rollback Failed",
      rollbackHeader: "Rollback Confirmation",
      rollbackMessage: "Rollback to snapshot {id}? Current config/rules will be overwritten and backed up.",
      id: "ID",
      createdAt: "Created At",
      createdBy: "Created By",
      summary: "Summary",
      fileCount: "Files",
      actions: "Actions",
    },
  },
};

const locale = ref<Locale>(resolveInitialLocale());

function resolveInitialLocale(): Locale {
  if (typeof window === "undefined") {
    return "zh-CN";
  }

  const fromStorage = window.localStorage.getItem(STORAGE_KEY);
  if (fromStorage === "zh-CN" || fromStorage === "en-US") {
    return fromStorage;
  }

  const nav = window.navigator.language.toLowerCase();
  if (nav.startsWith("en")) {
    return "en-US";
  }

  return "zh-CN";
}

function setLocale(nextLocale: Locale): void {
  locale.value = nextLocale;
  if (typeof window !== "undefined") {
    window.localStorage.setItem(STORAGE_KEY, nextLocale);
  }
}

function lookup(dict: MessageDict, key: string): MessageValue | undefined {
  const segments = key.split(".");
  let current: MessageValue | undefined = dict;
  for (const segment of segments) {
    if (!current || typeof current === "string") {
      return undefined;
    }
    current = current[segment];
  }
  return current;
}

function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) {
    return template;
  }

  return Object.entries(params).reduce((acc, [key, value]) => {
    return acc.replace(new RegExp(`\\{${key}\\}`, "g"), String(value));
  }, template);
}

function t(key: string, params?: Record<string, string | number>): string {
  const active = messages[locale.value];
  const fallback = messages["zh-CN"];
  const found = lookup(active, key) ?? lookup(fallback, key);
  if (typeof found !== "string") {
    return key;
  }
  return interpolate(found, params);
}

const localeOptions = computed(() => [
  { label: t("app.languageZh"), value: "zh-CN" as Locale },
  { label: t("app.languageEn"), value: "en-US" as Locale },
]);

export function useI18n() {
  return {
    locale,
    localeOptions,
    setLocale,
    t,
  };
}
