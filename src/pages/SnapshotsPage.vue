<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useConfirm } from "primevue/useconfirm";
import { useToast } from "primevue/usetoast";

import Button from "primevue/button";
import Card from "primevue/card";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import InputText from "primevue/inputtext";
import Tag from "primevue/tag";

import { useI18n } from "../i18n";
import { api } from "../lib/api";
import type { SnapshotMeta } from "../types";

const toast = useToast();
const confirm = useConfirm();
const { t } = useI18n();

const loading = ref(false);
const creating = ref(false);
const rollingBackId = ref("");

const summary = ref("");
const snapshots = ref<SnapshotMeta[]>([]);

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

async function loadSnapshots(): Promise<void> {
  loading.value = true;
  try {
    snapshots.value = await api.listSnapshots();
  } catch (error) {
    toast.add({ severity: "error", summary: t("common.loadFailed"), detail: String(error), life: 3500 });
  } finally {
    loading.value = false;
  }
}

async function createSnapshot(): Promise<void> {
  creating.value = true;
  try {
    await api.createSnapshot(summary.value.trim() || null);
    summary.value = "";
    toast.add({ severity: "success", summary: t("common.saveSuccess"), detail: t("snapshots.createSuccess"), life: 2800 });
    await loadSnapshots();
  } catch (error) {
    toast.add({ severity: "error", summary: t("snapshots.createFailed"), detail: String(error), life: 3500 });
  } finally {
    creating.value = false;
  }
}

function askRollback(snapshot: SnapshotMeta): void {
  confirm.require({
    message: t("snapshots.rollbackMessage", { id: snapshot.id }),
    header: t("snapshots.rollbackHeader"),
    icon: "pi pi-history",
    acceptLabel: t("snapshots.rollback"),
    rejectLabel: t("common.cancel"),
    acceptClass: "p-button-danger",
    accept: () => {
      void rollback(snapshot.id);
    },
  });
}

async function rollback(snapshotId: string): Promise<void> {
  rollingBackId.value = snapshotId;
  try {
    const result = await api.rollbackSnapshot(snapshotId);
    toast.add({
      severity: "success",
      summary: t("snapshots.rollbackDone"),
      detail: t("snapshots.rollbackDoneDetail", { count: result.restoredFiles.length }),
      life: 3000,
    });
    await loadSnapshots();
  } catch (error) {
    toast.add({ severity: "error", summary: t("snapshots.rollbackFailed"), detail: String(error), life: 3500 });
  } finally {
    rollingBackId.value = "";
  }
}

onMounted(() => {
  void loadSnapshots();
});
</script>

<template>
  <section class="space-y-5">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("snapshots.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("snapshots.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="loadSnapshots" />
    </header>

    <Card>
      <template #title>{{ t("snapshots.createSnapshot") }}</template>
      <template #content>
        <div class="grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto] md:items-end">
          <label class="flex flex-col gap-2">
            <span class="text-sm text-slate-600">{{ t("snapshots.summaryLabel") }}</span>
            <InputText v-model="summary" :placeholder="t('snapshots.summaryPlaceholder')" />
          </label>
          <Button icon="pi pi-save" :label="t('snapshots.create')" :loading="creating" severity="contrast" @click="createSnapshot" />
        </div>
      </template>
    </Card>

    <Card>
      <template #title>{{ t("snapshots.list") }}</template>
      <template #content>
        <DataTable :value="snapshots" :loading="loading" striped-rows>
          <Column field="id" :header="t('snapshots.id')" style="width: 220px" />
          <Column field="createdAt" :header="t('snapshots.createdAt')" style="width: 220px">
            <template #body="slotProps">
              {{ formatDate(slotProps.data.createdAt) }}
            </template>
          </Column>
          <Column field="createdBy" :header="t('snapshots.createdBy')" style="width: 140px" />
          <Column field="summary" :header="t('snapshots.summary')" />
          <Column :header="t('snapshots.fileCount')" style="width: 100px">
            <template #body="slotProps">
              <Tag :value="String(slotProps.data.files.length)" severity="info" />
            </template>
          </Column>
          <Column :header="t('snapshots.actions')" style="width: 150px">
            <template #body="slotProps">
              <Button
                icon="pi pi-history"
                :label="t('snapshots.rollback')"
                severity="danger"
                outlined
                :loading="rollingBackId === slotProps.data.id"
                @click="askRollback(slotProps.data)"
              />
            </template>
          </Column>
        </DataTable>
      </template>
    </Card>
  </section>
</template>
