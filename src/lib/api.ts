import { invoke } from "@tauri-apps/api/core";
import type {
  ApiKeyState,
  CodexConfig,
  NetworkProbeResult,
  RollbackResult,
  RuleCheckResult,
  RulesState,
  SandboxState,
  SavePreview,
  SkillDetail,
  SkillMeta,
  SnapshotMeta,
} from "../types";

export const api = {
  getConfig() {
    return invoke<CodexConfig>("get_config");
  },
  saveConfig(config: CodexConfig, dryRun: boolean) {
    return invoke<SavePreview>("save_config", {
      config,
      dry_run: dryRun,
    });
  },
  getApiKey() {
    return invoke<ApiKeyState>("get_api_key");
  },
  setApiKey(apiKey: string, dryRun: boolean) {
    return invoke<SavePreview>("set_api_key", {
      api_key: apiKey,
      dry_run: dryRun,
    });
  },
  getSandbox() {
    return invoke<SandboxState>("get_sandbox");
  },
  setSandbox(
    sandboxMode: string,
    workspaceWriteNetworkAccess: boolean,
    highRiskConfirmed: boolean,
    dryRun: boolean,
  ) {
    return invoke<SavePreview>("set_sandbox", {
      sandbox_mode: sandboxMode,
      workspace_write_network_access: workspaceWriteNetworkAccess,
      high_risk_confirmed: highRiskConfirmed,
      dry_run: dryRun,
    });
  },
  probeNetwork() {
    return invoke<NetworkProbeResult>("probe_network");
  },
  getRules() {
    return invoke<RulesState>("get_rules");
  },
  saveRules(rulesContent: string, dryRun: boolean) {
    return invoke<SavePreview>("save_rules", {
      rules_content: rulesContent,
      dry_run: dryRun,
    });
  },
  checkRuleCommand(command: string) {
    return invoke<RuleCheckResult>("check_rule_command", {
      command,
    });
  },
  checkSingleRule(ruleContent: string, command: string) {
    return invoke<RuleCheckResult>("check_single_rule", {
      rule_content: ruleContent,
      command,
    });
  },
  scanSkills() {
    return invoke<SkillMeta[]>("scan_skills");
  },
  readSkill(skillMdPath: string) {
    return invoke<SkillDetail>("read_skill", {
      skill_md_path: skillMdPath,
    });
  },
  createSnapshot(summary: string | null) {
    return invoke<SnapshotMeta>("create_snapshot", {
      summary,
    });
  },
  listSnapshots() {
    return invoke<SnapshotMeta[]>("list_snapshots");
  },
  rollbackSnapshot(snapshotId: string) {
    return invoke<RollbackResult>("rollback_snapshot", {
      snapshot_id: snapshotId,
    });
  },
};
