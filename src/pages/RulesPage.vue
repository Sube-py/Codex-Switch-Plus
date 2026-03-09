<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useToast } from "primevue/usetoast";

import Button from "primevue/button";
import Card from "primevue/card";
import Dialog from "primevue/dialog";
import Divider from "primevue/divider";
import Dropdown from "primevue/dropdown";
import InputText from "primevue/inputtext";
import SelectButton from "primevue/selectbutton";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";

import { useI18n } from "../i18n";
import { api } from "../lib/api";
import type { RuleCheckResult, SavePreview } from "../types";

type RuleDecision = "allow" | "prompt" | "forbidden";

type EditorMode = "visual" | "raw";

interface VisualRule {
  id: string;
  pattern: string;
  decision: RuleDecision;
  justification: string;
  matchExamples: string;
  notMatchExamples: string;
}

const toast = useToast();
const { t } = useI18n();

const loading = ref(false);
const submitting = ref(false);

const rulesPath = ref("");
const warnings = ref<string[]>([]);
const previewVisible = ref(false);
const preview = ref<SavePreview | null>(null);

const mode = ref<EditorMode>("visual");
const modeOptions = computed(() => [
  { label: t("rules.visualMode"), value: "visual" },
  { label: t("rules.rawMode"), value: "raw" },
]);

const rulesContentRaw = ref("");
const visualRules = ref<VisualRule[]>([]);

const commandInput = ref("gh pr view 7888");
const checkResult = ref<RuleCheckResult | null>(null);
const checking = ref(false);
const runningRuleId = ref<string | null>(null);
const ruleCheckDecisionMap = ref<Record<string, string>>({});
const ruleCheckMessageMap = ref<Record<string, string>>({});

const decisionOptions = [
  { label: "allow", value: "allow" },
  { label: "prompt", value: "prompt" },
  { label: "forbidden", value: "forbidden" },
];

const templates = computed(() => [
  {
    label: t("rules.templatePrompt"),
    value: {
      pattern: "gh pr view",
      decision: "prompt" as RuleDecision,
      justification: "Viewing pull requests should require user confirmation",
      matchExamples: "gh pr view 7888\ngh pr view 7888 --comments",
      notMatchExamples: "gh pr checkout 7888\ngh pr create",
    },
  },
  {
    label: t("rules.templateForbidden"),
    value: {
      pattern: "rm -rf",
      decision: "forbidden" as RuleDecision,
      justification: "Destructive recursive delete is blocked by policy",
      matchExamples: "rm -rf .",
      notMatchExamples: "",
    },
  },
  {
    label: t("rules.templateAllow"),
    value: {
      pattern: "python3 /absolute/path/scripts/check.py",
      decision: "allow" as RuleDecision,
      justification: "",
      matchExamples: "python3 /absolute/path/scripts/check.py",
      notMatchExamples: "python3 /absolute/path/scripts/other.py",
    },
  },
  {
    label: t("rules.templateUnion"),
    value: {
      pattern: "npm test | pnpm test",
      decision: "allow" as RuleDecision,
      justification: "",
      matchExamples: "npm test\npnpm test",
      notMatchExamples: "npm publish\npnpm add axios",
    },
  },
]);

function newRuleId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function createEmptyRule(): VisualRule {
  return {
    id: newRuleId(),
    pattern: "",
    decision: "prompt",
    justification: "",
    matchExamples: "",
    notMatchExamples: "",
  };
}

function parseQuotedStrings(input: string): string[] {
  const values: string[] = [];
  const regex = /"((?:\\.|[^"\\])*)"/g;
  let match: RegExpExecArray | null = regex.exec(input);
  while (match) {
    const raw = match[1] ?? "";
    const unescaped = raw.replace(/\\"/g, '"').replace(/\\\\/g, "\\");
    values.push(unescaped);
    match = regex.exec(input);
  }
  return values;
}

function parsePatternExpression(expr: string): string {
  const compact = expr.replace(/\s+/g, " ").trim();
  if (!compact) {
    return "";
  }

  if (compact.startsWith("[[")) {
    const groupRegex = /\[[^\[\]]*\]/g;
    const groups = (compact.match(groupRegex) ?? [])
      .map((group) => parseQuotedStrings(group).join(" "))
      .filter((segment) => segment.length > 0);

    if (groups.length > 0) {
      return groups.join(" | ");
    }
  }

  return parseQuotedStrings(compact).join(" ");
}

function extractExpression(body: string, key: "pattern" | "match" | "not_match"): string {
  const regex = new RegExp(
    `\\b${key}\\s*=\\s*([\\s\\S]*?)(?=\\n\\s*(?:pattern|decision|justification|match|not_match)\\s*=|$)`,
  );
  const matched = body.match(regex);
  if (!matched) {
    return "";
  }

  return matched[1].trim().replace(/,$/, "").trim();
}

function parseRulesToVisual(content: string): VisualRule[] {
  const blocks = [...content.matchAll(/prefix_rule\s*\(([\s\S]*?)\)\s*/g)];
  if (blocks.length === 0) {
    return [createEmptyRule()];
  }

  const rows = blocks.map((block) => {
    const body = (block[1] ?? "").trim();

    const patternExpr = extractExpression(body, "pattern");
    const decision = ((body.match(/\bdecision\s*=\s*"([^"]+)"/)?.[1] ?? "allow") as RuleDecision);
    const justification = body.match(/\bjustification\s*=\s*"([^"]*)"/)?.[1] ?? "";

    const matchExpr = extractExpression(body, "match");
    const notMatchExpr = extractExpression(body, "not_match");

    return {
      id: newRuleId(),
      pattern: parsePatternExpression(patternExpr),
      decision: decisionOptions.some((option) => option.value === decision) ? decision : "allow",
      justification,
      matchExamples: parseQuotedStrings(matchExpr).join("\n"),
      notMatchExamples: parseQuotedStrings(notMatchExpr).join("\n"),
    } satisfies VisualRule;
  });

  return rows.length > 0 ? rows : [createEmptyRule()];
}

function toTomlQuoted(value: string): string {
  const escaped = value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `"${escaped}"`;
}

function toLines(value: string): string[] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function toTomlStringList(values: string[]): string {
  if (values.length === 0) {
    return "[]";
  }
  return `[${values.map((value) => toTomlQuoted(value)).join(", ")}]`;
}

function patternToTomlExpression(pattern: string): string {
  const segments = pattern
    .split("|")
    .map((segment) => segment.trim())
    .filter((segment) => segment.length > 0);

  if (segments.length === 0) {
    return "[]";
  }

  const tokenGroups = segments.map((segment) => segment.split(/\s+/).filter((token) => token.length > 0));

  if (tokenGroups.length === 1) {
    return toTomlStringList(tokenGroups[0]);
  }

  const groups = tokenGroups.map((tokens) => toTomlStringList(tokens));
  return `[${groups.join(", ")}]`;
}

function ruleToBlock(rule: VisualRule): string {
  const lines: string[] = [
    "prefix_rule(",
    `    pattern = ${patternToTomlExpression(rule.pattern)},`,
    `    decision = ${toTomlQuoted(rule.decision)},`,
  ];

  if (rule.justification.trim()) {
    lines.push(`    justification = ${toTomlQuoted(rule.justification.trim())},`);
  }

  const matches = toLines(rule.matchExamples);
  const notMatches = toLines(rule.notMatchExamples);

  if (matches.length > 0) {
    lines.push(`    match = ${toTomlStringList(matches)},`);
  }

  if (notMatches.length > 0) {
    lines.push(`    not_match = ${toTomlStringList(notMatches)},`);
  }

  const last = lines.length - 1;
  if (lines[last]?.endsWith(",")) {
    lines[last] = lines[last].slice(0, -1);
  }

  lines.push(")");
  return lines.join("\n");
}

function visualRulesToContent(rows: VisualRule[]): string {
  const blocks = rows
    .filter((rule) => rule.pattern.trim().length > 0)
    .map((rule) => ruleToBlock(rule));

  if (blocks.length === 0) {
    return "";
  }

  return `${blocks.join("\n\n")}\n`;
}

function parsePatternSegments(pattern: string): string[][] {
  return pattern
    .split("|")
    .map((segment) => segment.trim())
    .filter((segment) => segment.length > 0)
    .map((segment) => segment.split(/\s+/).filter((token) => token.length > 0));
}

function getRuleWarnings(rule: VisualRule): string[] {
  const rowWarnings: string[] = [];
  const segments = parsePatternSegments(rule.pattern);

  if (segments.length === 0) {
    rowWarnings.push(t("rules.warningPatternEmpty"));
    return rowWarnings;
  }

  if (!["allow", "prompt", "forbidden"].includes(rule.decision)) {
    rowWarnings.push(t("rules.warningDecisionInvalid"));
  }

  if (rule.decision === "forbidden" && !rule.justification.trim()) {
    rowWarnings.push(t("rules.warningNeedJustification"));
  }

  if (!rule.matchExamples.trim() && !rule.notMatchExamples.trim()) {
    rowWarnings.push(t("rules.warningNeedExamples"));
  }

  if (rule.decision === "allow") {
    const shellBins = new Set(["bash", "sh", "zsh", "python", "python3", "node", "npm", "pnpm", "glab", "curl"]);
    const shellWrapperBins = new Set(["bash", "sh", "zsh", "fish"]);

    for (const tokens of segments) {
      if (tokens.length <= 1) {
        rowWarnings.push(t("rules.warningAllowTooWide", { tokens: tokens.join(" ") }));
      }

      if (tokens.length === 2 && shellBins.has(tokens[0]) && !tokens[1].startsWith("/")) {
        rowWarnings.push(t("rules.warningNeedAbsolutePath", { tokens: tokens.join(" ") }));
      }

      if (tokens.length >= 2 && shellWrapperBins.has(tokens[0]) && ["-c", "-lc"].includes(tokens[1])) {
        rowWarnings.push(t("rules.warningWrapperRisk", { cmd: tokens[0], flag: tokens[1] }));
      }
    }
  }

  return rowWarnings;
}

function inferCheckCommand(rule: VisualRule): string {
  const matches = toLines(rule.matchExamples);
  if (matches.length > 0) {
    return matches[0];
  }

  const segments = parsePatternSegments(rule.pattern);
  if (segments.length > 0) {
    return segments[0].join(" ");
  }

  return "";
}

function decisionSeverity(decision: string): "success" | "warn" | "danger" | "info" {
  if (decision === "allow") {
    return "success";
  }
  if (decision === "prompt") {
    return "warn";
  }
  if (decision === "forbidden" || decision === "error") {
    return "danger";
  }
  return "info";
}

function addRule(): void {
  visualRules.value.push(createEmptyRule());
}

function duplicateRule(rule: VisualRule): void {
  visualRules.value.push({
    ...rule,
    id: newRuleId(),
  });
}

function removeRule(index: number): void {
  visualRules.value.splice(index, 1);
  if (visualRules.value.length === 0) {
    addRule();
  }
}

function addTemplateRule(templateIndex: number): void {
  const template = templates.value[templateIndex];
  if (!template) {
    return;
  }

  visualRules.value.push({
    ...template.value,
    id: newRuleId(),
  });
}

function toRawFromVisual(): void {
  rulesContentRaw.value = visualRulesToContent(visualRules.value);
  toast.add({ severity: "info", summary: t("rules.syncToRaw"), detail: t("rules.syncDone"), life: 2000 });
}

function parseRawToVisual(): void {
  visualRules.value = parseRulesToVisual(rulesContentRaw.value);
  toast.add({ severity: "info", summary: t("rules.rawToVisual"), detail: t("rules.parsedDone"), life: 2000 });
}

const visualContentPreview = computed(() => visualRulesToContent(visualRules.value));

function currentEditorContent(): string {
  if (mode.value === "raw") {
    return rulesContentRaw.value;
  }

  return visualContentPreview.value;
}

async function loadRules(): Promise<void> {
  loading.value = true;
  try {
    const rules = await api.getRules();
    rulesPath.value = rules.path;
    warnings.value = rules.warnings;
    rulesContentRaw.value = rules.content;
    visualRules.value = parseRulesToVisual(rules.content);
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.loadFailed"), detail: String(error), life: 3500 });
  } finally {
    loading.value = false;
  }
}

async function previewSaveRules(): Promise<void> {
  submitting.value = true;
  try {
    const content = currentEditorContent();
    preview.value = await api.saveRules(content, true);
    warnings.value = preview.value.warnings;
    previewVisible.value = true;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.previewFailed"), detail: String(error), life: 3500 });
  } finally {
    submitting.value = false;
  }
}

async function confirmSaveRules(): Promise<void> {
  submitting.value = true;
  try {
    const content = currentEditorContent();
    const result = await api.saveRules(content, false);
    warnings.value = result.warnings;
    previewVisible.value = false;
    toast.add({ severity: "success", summary: t("common.saveSuccess"), detail: t("rules.saveDetail"), life: 2800 });
    await loadRules();
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.saveFailed"), detail: String(error), life: 3500 });
  } finally {
    submitting.value = false;
  }
}

async function runCheck(): Promise<void> {
  if (!commandInput.value.trim()) {
    toast.add({ severity: "warn", summary: t("common.notice"), detail: t("rules.needCommand"), life: 2500 });
    return;
  }

  checking.value = true;
  try {
    checkResult.value = await api.checkRuleCommand(commandInput.value.trim());
  } catch (error) {
    toast.add({ severity: "error", summary: t("rules.checkFailed"), detail: String(error), life: 3500 });
  } finally {
    checking.value = false;
  }
}

async function runRuleCheck(rule: VisualRule): Promise<void> {
  const command = inferCheckCommand(rule);
  if (!command) {
    toast.add({
      severity: "warn",
      summary: t("rules.cannotCheck"),
      detail: t("rules.needPatternForCheck"),
      life: 2500,
    });
    return;
  }

  runningRuleId.value = rule.id;
  try {
    const result = await api.checkSingleRule(ruleToBlock(rule), command);
    const decision = result.error ? "error" : (result.decision ?? "unknown");
    const message = result.error ?? t("rules.checkMessage", { command, decision: result.decision ?? "unknown" });

    ruleCheckDecisionMap.value[rule.id] = decision;
    ruleCheckMessageMap.value[rule.id] = message;

    toast.add({
      severity: result.error ? "error" : "success",
      summary: result.error ? t("rules.ruleCheckFailed") : t("rules.ruleCheckPassed"),
      detail: message,
      life: 3000,
    });
  } catch (error) {
    const message = String(error);
    ruleCheckDecisionMap.value[rule.id] = "error";
    ruleCheckMessageMap.value[rule.id] = message;
    toast.add({
      severity: "error",
      summary: t("rules.ruleCheckFailed"),
      detail: message,
      life: 3500,
    });
  } finally {
    runningRuleId.value = null;
  }
}

onMounted(() => {
  void loadRules();
});
</script>

<template>
  <section class="space-y-5">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("rules.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("rules.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="loadRules" />
    </header>

    <Card>
      <template #title>{{ t("rules.editorMode") }}</template>
      <template #content>
        <div class="mb-3 flex flex-wrap items-center gap-2">
          <SelectButton v-model="mode" :options="modeOptions" option-label="label" option-value="value" />
          <Tag :value="mode === 'visual' ? t('rules.visualModeTag') : t('rules.rawModeTag')" severity="info" />
        </div>

        <div class="text-sm text-slate-500">{{ t("rules.filePath") }}: {{ rulesPath }}</div>
      </template>
    </Card>

    <Card v-if="mode === 'visual'">
      <template #title>{{ t("rules.perRuleEditor") }}</template>
      <template #content>
        <div class="mb-3 flex flex-wrap gap-2">
          <Button icon="pi pi-plus" :label="t('rules.addRule')" @click="addRule" />
          <Button icon="pi pi-clone" :label="t('rules.syncToRaw')" outlined @click="toRawFromVisual" />
          <Button icon="pi pi-file-edit" :label="t('rules.rawToVisual')" outlined @click="parseRawToVisual" />
          <Dropdown
            :options="templates.map((item, index) => ({ label: t('rules.templatePrefix', { label: item.label }), value: index }))"
            option-label="label"
            option-value="value"
            :placeholder="t('rules.insertTemplate')"
            class="min-w-[240px]"
            @update:model-value="addTemplateRule"
          />
        </div>

        <div class="space-y-3">
          <div
            v-for="(rule, index) in visualRules"
            :key="rule.id"
            class="rounded-xl border border-slate-200 p-3"
          >
            <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
              <div class="flex items-center gap-2">
                <Tag :value="t('rules.ruleTag', { index: index + 1 })" severity="secondary" />
                <i
                  v-if="getRuleWarnings(rule).length > 0"
                  v-tooltip.top="getRuleWarnings(rule).join('\n')"
                  class="pi pi-exclamation-triangle cursor-help text-amber-500"
                />
                <Tag
                  v-if="ruleCheckDecisionMap[rule.id]"
                  v-tooltip.top="ruleCheckMessageMap[rule.id] || ''"
                  :value="`${t('common.check')}: ${ruleCheckDecisionMap[rule.id]}`"
                  :severity="decisionSeverity(ruleCheckDecisionMap[rule.id])"
                />
              </div>

              <div class="flex gap-2">
                <Button
                  icon="pi pi-play"
                  text
                  rounded
                  :loading="runningRuleId === rule.id"
                  v-tooltip.top="t('rules.checkRuleTooltip')"
                  @click="runRuleCheck(rule)"
                />
                <Button icon="pi pi-copy" text rounded @click="duplicateRule(rule)" />
                <Button icon="pi pi-trash" text rounded severity="danger" @click="removeRule(index)" />
              </div>
            </div>

            <div class="grid grid-cols-1 gap-3 md:grid-cols-[1fr_220px]">
              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">{{ t("rules.patternLabel") }}</span>
                <InputText v-model="rule.pattern" :placeholder="t('rules.patternPlaceholder')" />
              </label>

              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">{{ t("rules.decisionLabel") }}</span>
                <Dropdown v-model="rule.decision" :options="decisionOptions" option-label="label" option-value="value" />
              </label>
            </div>

            <label class="mt-3 flex flex-col gap-2">
              <span class="text-sm text-slate-600">{{ t("rules.justificationLabel") }}</span>
              <InputText v-model="rule.justification" :placeholder="t('rules.justificationPlaceholder')" />
            </label>

            <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2">
              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">{{ t("rules.matchLabel") }}</span>
                <Textarea v-model="rule.matchExamples" rows="4" auto-resize class="w-full" />
              </label>

              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">{{ t("rules.notMatchLabel") }}</span>
                <Textarea v-model="rule.notMatchExamples" rows="4" auto-resize class="w-full" />
              </label>
            </div>
          </div>
        </div>

        <div class="mt-4 rounded-lg border border-slate-200 bg-slate-950 p-3">
          <p class="mb-2 text-xs text-slate-400">{{ t("rules.generatedPreview") }}</p>
          <pre class="max-h-[240px] overflow-auto text-xs text-emerald-200">{{ visualContentPreview || t("rules.empty") }}</pre>
        </div>
      </template>
    </Card>

    <Card v-else>
      <template #title>{{ t("rules.rawEditor") }}</template>
      <template #content>
        <Textarea v-model="rulesContentRaw" auto-resize rows="16" class="w-full font-mono text-sm" />
      </template>
    </Card>

    <Card>
      <template #title>{{ t("rules.globalWarnings") }}</template>
      <template #content>
        <div v-if="warnings.length > 0" class="rounded-lg border border-amber-200 bg-amber-50 p-3">
          <ul class="list-disc pl-5 text-sm text-amber-700">
            <li v-for="(warning, index) in warnings" :key="`${warning}-${index}`">{{ warning }}</li>
          </ul>
        </div>
        <p v-else class="text-sm text-slate-500">{{ t("rules.noWarnings") }}</p>

        <div class="mt-4 flex justify-end">
          <Button
            icon="pi pi-eye"
            :label="t('rules.previewAndSave')"
            severity="contrast"
            :loading="submitting"
            @click="previewSaveRules"
          />
        </div>
      </template>
    </Card>

    <Card>
      <template #title>{{ t("rules.playground") }}</template>
      <template #content>
        <div class="flex flex-wrap gap-2">
          <InputText v-model="commandInput" class="min-w-[420px] flex-1" :placeholder="t('rules.commandPlaceholder')" />
          <Button icon="pi pi-play" :label="t('rules.runCheck')" :loading="checking" @click="runCheck" />
        </div>

        <div v-if="checkResult" class="mt-4 space-y-3">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm text-slate-500">{{ t("rules.decisionLabel") }}:</span>
            <Tag :value="checkResult.decision ?? 'unknown'" :severity="checkResult.decision === 'allow' ? 'success' : 'danger'" />
          </div>

          <div>
            <p class="text-sm text-slate-600">{{ t("rules.matchedRules") }}</p>
            <ul v-if="checkResult.matchedRules.length > 0" class="list-disc pl-5 text-sm">
              <li v-for="(rule, index) in checkResult.matchedRules" :key="`${rule}-${index}`">{{ rule }}</li>
            </ul>
            <p v-else class="text-sm text-slate-500">{{ t("rules.noMatchedRules") }}</p>
          </div>

          <div v-if="checkResult.error" class="rounded border border-red-200 bg-red-50 p-2 text-sm text-red-700">
            {{ checkResult.error }}
          </div>

          <pre class="max-h-[320px] overflow-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-xs text-emerald-200">{{ checkResult.rawOutput }}</pre>
        </div>
      </template>
    </Card>

    <Dialog v-model:visible="previewVisible" modal :header="t('common.savePreview')" :style="{ width: '72rem', maxWidth: '96vw' }">
      <div v-if="preview" class="space-y-3">
        <p class="text-sm text-slate-500">{{ t("common.targetFile") }}: {{ preview.path }}</p>

        <div v-if="preview.warnings.length > 0" class="rounded-lg border border-amber-200 bg-amber-50 p-3">
          <p class="mb-1 text-sm font-medium text-amber-800">{{ t("common.riskTips") }}</p>
          <ul class="list-disc pl-5 text-sm text-amber-700">
            <li v-for="(warning, index) in preview.warnings" :key="`${warning}-${index}`">{{ warning }}</li>
          </ul>
        </div>

        <Divider />
        <pre class="max-h-[55vh] overflow-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-xs text-emerald-200">{{ preview.diff }}</pre>
      </div>

      <template #footer>
        <Button :label="t('common.cancel')" text @click="previewVisible = false" />
        <Button :label="t('common.confirmSave')" severity="contrast" :loading="submitting" @click="confirmSaveRules" />
      </template>
    </Dialog>
  </section>
</template>
