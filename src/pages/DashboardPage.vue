<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useToast } from "primevue/usetoast";

import Button from "primevue/button";
import Card from "primevue/card";
import Tag from "primevue/tag";

import { useI18n } from "../i18n";
import { api } from "../lib/api";

const router = useRouter();
const toast = useToast();
const { t } = useI18n();

const loading = ref(false);
const provider = ref("-");
const model = ref("-");
const sandboxMode = ref("-");
const workspaceNetwork = ref(false);
const rulesCount = ref(0);
const skillsCount = ref(0);
const snapshotsCount = ref(0);

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    const [config, sandbox, rules, skills, snapshots] = await Promise.all([
      api.getConfig(),
      api.getSandbox(),
      api.getRules(),
      api.scanSkills(),
      api.listSnapshots(),
    ]);

    provider.value = config.modelProvider || "-";
    model.value = config.model || "-";
    sandboxMode.value = sandbox.sandboxMode;
    workspaceNetwork.value = sandbox.workspaceWriteNetworkAccess;
    rulesCount.value = rules.content
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0).length;
    skillsCount.value = skills.length;
    snapshotsCount.value = snapshots.length;
  } catch (error) {
    toast.add({
      severity: "error",
      summary: t("common.loadFailed"),
      detail: String(error),
      life: 3500,
    });
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <section>
    <header class="mb-6 flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("dashboard.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("dashboard.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="refresh" />
    </header>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-3">
      <Card>
        <template #title>{{ t("dashboard.modelConfig") }}</template>
        <template #content>
          <p class="text-sm text-slate-500">{{ t("dashboard.provider") }}</p>
          <p class="mb-3 text-lg font-semibold">{{ provider }}</p>
          <p class="text-sm text-slate-500">{{ t("dashboard.model") }}</p>
          <p class="text-base font-medium">{{ model }}</p>
        </template>
      </Card>

      <Card>
        <template #title>{{ t("dashboard.sandboxNetwork") }}</template>
        <template #content>
          <div class="mb-3 flex items-center gap-2">
            <Tag :value="sandboxMode" severity="info" />
          </div>
          <p class="text-sm text-slate-500">{{ t("dashboard.workspaceWriteNetwork") }}</p>
          <Tag :value="workspaceNetwork ? t('dashboard.enabled') : t('dashboard.disabled')" :severity="workspaceNetwork ? 'success' : 'warn'" />
        </template>
      </Card>

      <Card>
        <template #title>{{ t("dashboard.resources") }}</template>
        <template #content>
          <div class="grid grid-cols-3 gap-2 text-center">
            <div class="rounded-lg bg-slate-50 p-2">
              <p class="text-xl font-semibold">{{ rulesCount }}</p>
              <p class="text-xs text-slate-500">{{ t("dashboard.rulesCount") }}</p>
            </div>
            <div class="rounded-lg bg-slate-50 p-2">
              <p class="text-xl font-semibold">{{ skillsCount }}</p>
              <p class="text-xs text-slate-500">{{ t("dashboard.skillsCount") }}</p>
            </div>
            <div class="rounded-lg bg-slate-50 p-2">
              <p class="text-xl font-semibold">{{ snapshotsCount }}</p>
              <p class="text-xs text-slate-500">{{ t("dashboard.snapshotsCount") }}</p>
            </div>
          </div>
        </template>
      </Card>
    </div>

    <div class="mt-6 flex flex-wrap gap-2">
      <Button :label="t('dashboard.gotoConfig')" icon="pi pi-cog" severity="contrast" @click="router.push('/config')" />
      <Button :label="t('dashboard.gotoRules')" icon="pi pi-sliders-h" outlined @click="router.push('/rules')" />
      <Button :label="t('dashboard.gotoSkills')" icon="pi pi-bolt" outlined @click="router.push('/skills')" />
    </div>
  </section>
</template>
