<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useConfirm } from "primevue/useconfirm";
import { useToast } from "primevue/usetoast";

import Button from "primevue/button";
import Card from "primevue/card";
import Dialog from "primevue/dialog";
import Divider from "primevue/divider";
import InputSwitch from "primevue/inputswitch";
import SelectButton from "primevue/selectbutton";
import Tag from "primevue/tag";

import { useI18n } from "../i18n";
import { api } from "../lib/api";
import type { NetworkProbeResult, SavePreview, SandboxState } from "../types";

const toast = useToast();
const confirm = useConfirm();
const { t } = useI18n();

const loading = ref(false);
const submitting = ref(false);
const state = ref<SandboxState>({
  sandboxMode: "workspace-write",
  workspaceWriteNetworkAccess: false,
});

const probeResult = ref<NetworkProbeResult | null>(null);
const probeLoading = ref(false);

const preview = ref<SavePreview | null>(null);
const previewVisible = ref(false);

const modeOptions = [
  { label: "read-only", value: "read-only" },
  { label: "workspace-write", value: "workspace-write" },
  { label: "danger-full-access", value: "danger-full-access" },
];

async function loadSandboxState(): Promise<void> {
  loading.value = true;
  try {
    state.value = await api.getSandbox();
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.loadFailed"), detail: String(error), life: 3500 });
  } finally {
    loading.value = false;
  }
}

async function previewSave(): Promise<void> {
  submitting.value = true;
  try {
    preview.value = await api.setSandbox(
      state.value.sandboxMode,
      state.value.workspaceWriteNetworkAccess,
      true,
      true,
    );
    previewVisible.value = true;
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.previewFailed"), detail: String(error), life: 3500 });
  } finally {
    submitting.value = false;
  }
}

async function applySave(highRiskConfirmed: boolean): Promise<void> {
  submitting.value = true;
  try {
    await api.setSandbox(
      state.value.sandboxMode,
      state.value.workspaceWriteNetworkAccess,
      highRiskConfirmed,
      false,
    );
    toast.add({ severity: "success", summary: t("common.saveSuccess"), detail: t("sandbox.saveSuccess"), life: 2800 });
    previewVisible.value = false;
    await loadSandboxState();
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.saveFailed"), detail: String(error), life: 3500 });
  } finally {
    submitting.value = false;
  }
}

function confirmApply(): void {
  if (state.value.sandboxMode !== "danger-full-access") {
    void applySave(false);
    return;
  }

  confirm.require({
    message: t("sandbox.dangerMessage"),
    header: t("sandbox.dangerHeader"),
    icon: "pi pi-exclamation-triangle",
    rejectLabel: t("common.cancel"),
    acceptLabel: t("sandbox.dangerAccept"),
    acceptClass: "p-button-danger",
    accept: () => {
      void applySave(true);
    },
  });
}

async function runProbe(): Promise<void> {
  probeLoading.value = true;
  try {
    probeResult.value = await api.probeNetwork();
    toast.add({ severity: "info", summary: t("sandbox.probeDone"), detail: probeResult.value.summary, life: 2800 });
  } catch (error) {
    toast.add({ severity: "error", summary: t("sandbox.probeFailed"), detail: String(error), life: 3500 });
  } finally {
    probeLoading.value = false;
  }
}

onMounted(() => {
  void loadSandboxState();
});
</script>

<template>
  <section class="space-y-5">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("sandbox.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("sandbox.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="loadSandboxState" />
    </header>

    <Card>
      <template #title>{{ t("sandbox.modeSwitch") }}</template>
      <template #content>
        <div class="space-y-4">
          <div>
            <p class="mb-2 text-sm text-slate-600">{{ t("sandbox.sandboxMode") }}</p>
            <SelectButton v-model="state.sandboxMode" :options="modeOptions" option-label="label" option-value="value" />
          </div>

          <label class="inline-flex items-center gap-3 rounded-lg border border-slate-200 px-3 py-2">
            <InputSwitch v-model="state.workspaceWriteNetworkAccess" />
            <span>{{ t("sandbox.workspaceWriteNetwork") }}</span>
            <Tag :value="state.workspaceWriteNetworkAccess ? t('dashboard.enabled') : t('dashboard.disabled')" :severity="state.workspaceWriteNetworkAccess ? 'success' : 'warn'" />
          </label>

          <div class="rounded-lg border border-slate-200 bg-slate-50 p-3 text-sm text-slate-700">
            {{ t("sandbox.commonPitfall") }}
          </div>

          <div class="flex justify-end">
            <Button
              icon="pi pi-eye"
              :label="t('sandbox.previewAndSave')"
              severity="contrast"
              :loading="submitting"
              @click="previewSave"
            />
          </div>
        </div>
      </template>
    </Card>

    <Card>
      <template #title>{{ t("sandbox.probeTitle") }}</template>
      <template #content>
        <div class="flex flex-wrap items-center gap-3">
          <Button icon="pi pi-wifi" :label="t('sandbox.runProbe')" :loading="probeLoading" @click="runProbe" />
          <span class="text-sm text-slate-500">{{ t("sandbox.probeTarget") }}</span>
        </div>

        <div v-if="probeResult" class="mt-4 rounded-lg border border-slate-200 p-3 text-sm">
          <p><strong>{{ t("sandbox.result") }}:</strong>{{ probeResult.summary }}</p>
          <p><strong>{{ t("sandbox.category") }}:</strong>{{ probeResult.category }}</p>
          <p><strong>{{ t("sandbox.dns") }}:</strong>{{ probeResult.dnsOk ? t("common.ok") : t("common.fail") }}</p>
          <p><strong>{{ t("sandbox.tcp") }}:</strong>{{ probeResult.tcpOk ? t("common.ok") : t("common.fail") }}</p>
          <p v-if="probeResult.latencyMs !== null"><strong>{{ t("sandbox.latency") }}:</strong>{{ probeResult.latencyMs }} ms</p>
          <p v-if="probeResult.error"><strong>{{ t("sandbox.error") }}:</strong>{{ probeResult.error }}</p>
        </div>
      </template>
    </Card>

    <Dialog v-model:visible="previewVisible" modal :header="t('common.savePreview')" :style="{ width: '68rem', maxWidth: '95vw' }">
      <div v-if="preview" class="space-y-3">
        <p class="text-sm text-slate-500">{{ t("common.targetFile") }}：{{ preview.path }}</p>
        <Divider />
        <pre class="max-h-[55vh] overflow-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-xs text-emerald-200">{{ preview.diff }}</pre>
      </div>

      <template #footer>
        <Button :label="t('common.cancel')" text @click="previewVisible = false" />
        <Button
          :label="t('common.confirmSave')"
          severity="contrast"
          :loading="submitting"
          @click="confirmApply"
        />
      </template>
    </Dialog>
  </section>
</template>
