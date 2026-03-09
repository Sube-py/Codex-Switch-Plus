<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { z } from "zod";
import { useToast } from "primevue/usetoast";

import Button from "primevue/button";
import Card from "primevue/card";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import Divider from "primevue/divider";
import Dropdown from "primevue/dropdown";
import InputText from "primevue/inputtext";
import Tag from "primevue/tag";

import { useI18n } from "../i18n";
import { api } from "../lib/api";
import type { CodexConfig, SavePreview } from "../types";

const toast = useToast();
const { t } = useI18n();

const loading = ref(false);
const submitting = ref(false);
const form = ref<CodexConfig | null>(null);

const selectedProviderKey = ref("");
const apiKeyMasked = ref<string | null>(null);
const apiKeyInput = ref("");

const validationErrors = ref<string[]>([]);
const diffVisible = ref(false);
const pendingPreview = ref<SavePreview | null>(null);
const pendingAction = ref<"config" | "apiKey" | null>(null);

const trustLevelOptions = computed(() => [
  { label: "trusted", value: "trusted" },
  { label: "untrusted", value: "untrusted" },
]);

const reasoningOptions = [
  { label: "low", value: "low" },
  { label: "medium", value: "medium" },
  { label: "high", value: "high" },
];

const networkOptions = [
  { label: "enabled", value: "enabled" },
  { label: "disabled", value: "disabled" },
];

const providerKeys = computed(() => {
  if (!form.value) {
    return [];
  }

  return Object.keys(form.value.modelProviders).map((key) => ({
    label: key,
    value: key,
  }));
});

const selectedProvider = computed(() => {
  if (!form.value || !selectedProviderKey.value) {
    return null;
  }

  return form.value.modelProviders[selectedProviderKey.value] ?? null;
});

function cloneConfig(config: CodexConfig): CodexConfig {
  return {
    ...config,
    modelProviders: Object.fromEntries(
      Object.entries(config.modelProviders).map(([key, value]) => [key, { ...value }]),
    ),
    projects: config.projects.map((project) => ({ ...project })),
  };
}

async function loadData(): Promise<void> {
  loading.value = true;
  try {
    const [config, apiKeyState] = await Promise.all([api.getConfig(), api.getApiKey()]);
    form.value = cloneConfig(config);

    const firstProvider = Object.keys(config.modelProviders)[0] ?? "";
    selectedProviderKey.value = config.modelProvider || firstProvider;

    apiKeyMasked.value = apiKeyState.maskedValue;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.loadFailed"), detail: String(error), life: 4000 });
  } finally {
    loading.value = false;
  }
}

function addProvider(): void {
  if (!form.value) {
    return;
  }

  let index = 1;
  let candidate = `provider-${index}`;
  while (form.value.modelProviders[candidate]) {
    index += 1;
    candidate = `provider-${index}`;
  }

  form.value.modelProviders[candidate] = {
    name: candidate,
    baseUrl: "https://api.openai.com/v1",
    wireApi: "responses",
    requiresOpenaiAuth: true,
  };

  selectedProviderKey.value = candidate;
  if (!form.value.modelProvider) {
    form.value.modelProvider = candidate;
  }
}

function removeProvider(key: string): void {
  if (!form.value) {
    return;
  }

  delete form.value.modelProviders[key];
  const remaining = Object.keys(form.value.modelProviders);
  if (remaining.length === 0) {
    addProvider();
    return;
  }

  if (selectedProviderKey.value === key) {
    selectedProviderKey.value = remaining[0];
  }
  if (form.value.modelProvider === key) {
    form.value.modelProvider = remaining[0];
  }
}

function addProject(): void {
  form.value?.projects.push({
    path: "",
    trustLevel: "untrusted",
  });
}

function removeProject(index: number): void {
  form.value?.projects.splice(index, 1);
}

function validateCurrentConfig(): boolean {
  validationErrors.value = [];

  if (!form.value) {
    validationErrors.value.push(t("config.configNotLoaded"));
    return false;
  }

  const configSchema = z.object({
    modelProvider: z.string().min(1, t("config.modelProviderEmpty")),
    model: z.string().min(1, t("config.modelEmpty")),
    modelReasoningEffort: z.string().min(1, t("config.reasoningEmpty")),
    networkAccess: z.string().min(1, t("config.networkEmpty")),
  });

  const providerSchema = z.object({
    name: z.string().min(1, t("config.providerNameEmpty")),
    baseUrl: z.string().url(t("config.providerBaseUrlInvalid")),
    wireApi: z.string().min(1, t("config.providerWireApiEmpty")),
    requiresOpenaiAuth: z.boolean(),
  });

  const parsedConfig = configSchema.safeParse({
    modelProvider: form.value.modelProvider,
    model: form.value.model,
    modelReasoningEffort: form.value.modelReasoningEffort,
    networkAccess: form.value.networkAccess,
  });

  if (!parsedConfig.success) {
    validationErrors.value.push(...parsedConfig.error.issues.map((issue) => issue.message));
  }

  const providerEntries = Object.entries(form.value.modelProviders);
  if (providerEntries.length === 0) {
    validationErrors.value.push(t("config.needOneProvider"));
  }

  for (const [providerKey, provider] of providerEntries) {
    const parsedProvider = providerSchema.safeParse(provider);
    if (!parsedProvider.success) {
      validationErrors.value.push(
        ...parsedProvider.error.issues.map((issue) => `[${providerKey}] ${issue.message}`),
      );
    }
  }

  for (const [index, project] of form.value.projects.entries()) {
    if (!project.path.trim()) {
      validationErrors.value.push(t("config.projectPathEmpty", { index: index + 1 }));
    }
  }

  if (!form.value.modelProviders[form.value.modelProvider]) {
    validationErrors.value.push(t("config.providerNotFound"));
  }

  return validationErrors.value.length === 0;
}

async function previewConfigSave(): Promise<void> {
  if (!form.value || !validateCurrentConfig()) {
    return;
  }

  submitting.value = true;
  try {
    const preview = await api.saveConfig(form.value, true);
    pendingAction.value = "config";
    pendingPreview.value = preview;
    diffVisible.value = true;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.previewFailed"), detail: String(error), life: 4000 });
  } finally {
    submitting.value = false;
  }
}

async function previewApiKeySave(): Promise<void> {
  if (!apiKeyInput.value.trim()) {
    toast.add({ severity: "warn", summary: t("common.notice"), detail: t("config.needApiKeyInput"), life: 2500 });
    return;
  }

  submitting.value = true;
  try {
    const preview = await api.setApiKey(apiKeyInput.value.trim(), true);
    pendingAction.value = "apiKey";
    pendingPreview.value = preview;
    diffVisible.value = true;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.previewFailed"), detail: String(error), life: 4000 });
  } finally {
    submitting.value = false;
  }
}

async function confirmApply(): Promise<void> {
  if (!pendingAction.value || !pendingPreview.value || !form.value) {
    return;
  }

  submitting.value = true;
  try {
    if (pendingAction.value === "config") {
      await api.saveConfig(form.value, false);
      toast.add({ severity: "success", summary: t("common.saveSuccess"), detail: t("config.saveConfigSuccess"), life: 2800 });
    }

    if (pendingAction.value === "apiKey") {
      await api.setApiKey(apiKeyInput.value.trim(), false);
      apiKeyInput.value = "";
      toast.add({ severity: "success", summary: t("common.saveSuccess"), detail: t("config.saveApiKeySuccess"), life: 2800 });
    }

    await loadData();
    diffVisible.value = false;
    pendingPreview.value = null;
    pendingAction.value = null;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.saveFailed"), detail: String(error), life: 4000 });
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  void loadData();
});
</script>

<template>
  <section class="space-y-5">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("config.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("config.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="loadData" />
    </header>

    <Card>
      <template #title>{{ t("config.baseConfig") }}</template>
      <template #content>
        <div v-if="!form" class="text-sm text-slate-500">{{ t("config.loading") }}</div>
        <div v-else class="space-y-4">
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <label class="flex flex-col gap-2">
              <span class="text-sm text-slate-600">model_provider</span>
              <Dropdown v-model="form.modelProvider" :options="providerKeys" option-label="label" option-value="value" />
            </label>

            <label class="flex flex-col gap-2">
              <span class="text-sm text-slate-600">model</span>
              <InputText v-model="form.model" />
            </label>

            <label class="flex flex-col gap-2">
              <span class="text-sm text-slate-600">model_reasoning_effort</span>
              <Dropdown
                v-model="form.modelReasoningEffort"
                :options="reasoningOptions"
                option-label="label"
                option-value="value"
              />
            </label>

            <label class="flex flex-col gap-2">
              <span class="text-sm text-slate-600">network_access</span>
              <Dropdown v-model="form.networkAccess" :options="networkOptions" option-label="label" option-value="value" />
            </label>
          </div>

          <div class="rounded-xl border border-slate-200 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h3 class="font-medium">model_providers</h3>
              <Button icon="pi pi-plus" :label="t('config.addProvider')" text @click="addProvider" />
            </div>

            <div class="mb-3 flex flex-wrap gap-2">
              <Tag
                v-for="item in providerKeys"
                :key="item.value"
                :value="item.label"
                class="cursor-pointer"
                :severity="selectedProviderKey === item.value ? 'contrast' : 'secondary'"
                @click="selectedProviderKey = item.value"
              />
            </div>

            <div v-if="selectedProvider" class="space-y-3">
              <div class="flex items-center justify-between gap-2">
                <label class="flex-1">
                  <span class="mb-1 block text-sm text-slate-600">{{ t("config.currentProviderKey") }}</span>
                  <InputText v-model="selectedProviderKey" disabled />
                </label>
                <Button
                  icon="pi pi-trash"
                  :label="t('config.delete')"
                  severity="danger"
                  outlined
                  :disabled="providerKeys.length <= 1"
                  @click="removeProvider(selectedProviderKey)"
                />
              </div>

              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">name</span>
                <InputText v-model="selectedProvider.name" />
              </label>

              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">base_url</span>
                <InputText v-model="selectedProvider.baseUrl" />
              </label>

              <label class="flex flex-col gap-2">
                <span class="text-sm text-slate-600">wire_api</span>
                <InputText v-model="selectedProvider.wireApi" />
              </label>

              <label class="inline-flex items-center gap-2">
                <Checkbox v-model="selectedProvider.requiresOpenaiAuth" binary />
                <span class="text-sm text-slate-700">requires_openai_auth</span>
              </label>
            </div>
          </div>

          <div class="rounded-xl border border-slate-200 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h3 class="font-medium">{{ t("config.projects") }}</h3>
              <Button icon="pi pi-plus" :label="t('config.addProject')" text @click="addProject" />
            </div>

            <div class="space-y-2">
              <div
                v-for="(project, index) in form.projects"
                :key="`project-${index}`"
                class="grid grid-cols-1 gap-2 rounded-lg border border-slate-200 p-3 md:grid-cols-[1fr_180px_100px]"
              >
                <InputText v-model="project.path" placeholder="/absolute/path" />
                <Dropdown v-model="project.trustLevel" :options="trustLevelOptions" option-label="label" option-value="value" />
                <Button icon="pi pi-trash" severity="danger" outlined @click="removeProject(index)" />
              </div>
            </div>
          </div>

          <div v-if="validationErrors.length > 0" class="rounded-lg border border-red-200 bg-red-50 p-3">
            <p class="mb-1 text-sm font-medium text-red-700">{{ t("config.validationErrors") }}</p>
            <ul class="list-disc pl-5 text-sm text-red-700">
              <li v-for="(error, index) in validationErrors" :key="`${error}-${index}`">{{ error }}</li>
            </ul>
          </div>

          <div class="flex justify-end">
            <Button
              icon="pi pi-eye"
              :label="t('config.previewAndSaveConfig')"
              :loading="submitting"
              severity="contrast"
              @click="previewConfigSave"
            />
          </div>
        </div>
      </template>
    </Card>

    <Card>
      <template #title>{{ t("config.apiKey") }}</template>
      <template #content>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-[1fr_320px_auto] md:items-end">
          <div>
            <p class="text-sm text-slate-500">{{ t("config.currentMasked") }}</p>
            <p class="mt-1 font-mono text-sm">{{ apiKeyMasked ?? t("config.notSet") }}</p>
          </div>
          <label class="flex flex-col gap-2">
            <span class="text-sm text-slate-600">{{ t("config.newApiKey") }}</span>
            <InputText v-model="apiKeyInput" type="password" :placeholder="t('config.apiKeyPlaceholder')" />
          </label>
          <Button icon="pi pi-key" :label="t('config.previewAndSave')" :loading="submitting" @click="previewApiKeySave" />
        </div>
      </template>
    </Card>

    <Dialog v-model:visible="diffVisible" modal :header="t('common.savePreview')" :style="{ width: '70rem', maxWidth: '95vw' }">
      <div class="space-y-3" v-if="pendingPreview">
        <p class="text-sm text-slate-500">{{ t("common.targetFile") }}：{{ pendingPreview.path }}</p>

        <div v-if="pendingPreview.warnings.length > 0" class="rounded-lg border border-amber-200 bg-amber-50 p-3">
          <p class="mb-1 text-sm font-medium text-amber-800">{{ t("common.riskTips") }}</p>
          <ul class="list-disc pl-5 text-sm text-amber-800">
            <li v-for="(warning, index) in pendingPreview.warnings" :key="`${warning}-${index}`">{{ warning }}</li>
          </ul>
        </div>

        <Divider />
        <pre class="max-h-[55vh] overflow-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-xs text-emerald-200">{{ pendingPreview.diff }}</pre>
      </div>

      <template #footer>
        <Button :label="t('common.cancel')" text @click="diffVisible = false" />
        <Button :label="t('common.confirmSave')" severity="contrast" :loading="submitting" @click="confirmApply" />
      </template>
    </Dialog>
  </section>
</template>
