export interface ModelProviderConfig {
  name: string;
  baseUrl: string;
  wireApi: string;
  requiresOpenaiAuth: boolean;
}

export interface ProjectTrust {
  path: string;
  trustLevel: string;
}

export interface CodexConfig {
  modelProvider: string;
  model: string;
  modelReasoningEffort: string;
  networkAccess: string;
  sandboxWorkspaceWriteNetworkAccess: boolean;
  modelProviders: Record<string, ModelProviderConfig>;
  projects: ProjectTrust[];
  rawToml: string;
}

export interface SavePreview {
  path: string;
  diff: string;
  saved: boolean;
  backupPath: string | null;
  warnings: string[];
}

export interface ApiKeyState {
  exists: boolean;
  maskedValue: string | null;
}

export interface SandboxState {
  sandboxMode: "read-only" | "workspace-write" | "danger-full-access" | string;
  workspaceWriteNetworkAccess: boolean;
}

export interface RulesState {
  path: string;
  content: string;
  warnings: string[];
}

export interface RuleCheckResult {
  decision: string | null;
  matchedRules: string[];
  rawOutput: string;
  parsedOutput: unknown | null;
  error: string | null;
}

export interface SkillMeta {
  name: string;
  description: string;
  path: string;
  skillMdPath: string;
  scripts: string[];
  dependencies: string[];
  commands: string[];
}

export interface SkillDetail {
  path: string;
  content: string;
}

export interface SnapshotFileEntry {
  sourcePath: string;
  snapshotPath: string;
}

export interface SnapshotMeta {
  id: string;
  createdAt: string;
  createdBy: string;
  summary: string;
  files: SnapshotFileEntry[];
}

export interface RollbackResult {
  snapshotId: string;
  restoredFiles: string[];
  backupPaths: string[];
}

export interface NetworkProbeResult {
  dnsOk: boolean;
  tcpOk: boolean;
  category: string;
  summary: string;
  latencyMs: number | null;
  error: string | null;
}
