<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { openPath } from "@tauri-apps/plugin-opener";
import { useToast } from "primevue/usetoast";
import DOMPurify from "dompurify";
import { marked } from "marked";

import Button from "primevue/button";
import Card from "primevue/card";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import ProgressSpinner from "primevue/progressspinner";
import Splitter from "primevue/splitter";
import SplitterPanel from "primevue/splitterpanel";
import Tag from "primevue/tag";

import { useI18n } from "../i18n";
import { api } from "../lib/api";
import type { SkillMeta } from "../types";

const toast = useToast();
const { t } = useI18n();

const loading = ref(false);
const detailLoading = ref(false);
const skills = ref<SkillMeta[]>([]);
const selectedSkill = ref<SkillMeta | null>(null);
const selectedContent = ref("");

const renderedMarkdown = computed(() => {
  if (!selectedContent.value.trim()) {
    return `<p>${t("skills.noContent")}</p>`;
  }

  const html = marked.parse(selectedContent.value) as string;
  return DOMPurify.sanitize(html);
});

async function loadSkills(): Promise<void> {
  loading.value = true;
  try {
    skills.value = await api.scanSkills();
    if (skills.value.length > 0 && !selectedSkill.value) {
      selectedSkill.value = skills.value[0];
    }
  } catch (error) {
    toast.add({ severity: "error", summary: t("skills.loadFailed"), detail: String(error), life: 3500 });
  } finally {
    loading.value = false;
  }
}

async function loadSkillDetail(skill: SkillMeta): Promise<void> {
  detailLoading.value = true;
  try {
    const detail = await api.readSkill(skill.skillMdPath);
    selectedContent.value = detail.content;
  } catch (error) {
    selectedContent.value = "";
    toast.add({ severity: "error", summary: t("skills.readFailed"), detail: String(error), life: 3500 });
  } finally {
    detailLoading.value = false;
  }
}

watch(
  selectedSkill,
  (skill) => {
    if (skill) {
      void loadSkillDetail(skill);
    }
  },
  { immediate: true },
);

async function openDirectory(): Promise<void> {
  if (!selectedSkill.value) {
    return;
  }

  try {
    await openPath(selectedSkill.value.path);
  } catch (error) {
    toast.add({ severity: "error", summary: t("skills.openDirFailed"), detail: String(error), life: 3000 });
  }
}

async function openSkillFile(): Promise<void> {
  if (!selectedSkill.value) {
    return;
  }

  try {
    await openPath(selectedSkill.value.skillMdPath);
  } catch (error) {
    toast.add({ severity: "error", summary: t("skills.openFileFailed"), detail: String(error), life: 3000 });
  }
}

onMounted(() => {
  void loadSkills();
});
</script>

<template>
  <section class="space-y-5">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("skills.title") }}</h2>
        <p class="mt-1 text-sm text-slate-500">{{ t("skills.subtitle") }}</p>
      </div>
      <Button icon="pi pi-refresh" :label="t('common.refresh')" :loading="loading" @click="loadSkills" />
    </header>

    <Card>
      <template #content>
        <Splitter class="min-h-[620px] rounded-xl border border-slate-200">
          <SplitterPanel :size="44" class="p-3">
            <DataTable
              v-model:selection="selectedSkill"
              :value="skills"
              data-key="skillMdPath"
              selection-mode="single"
              :loading="loading"
              striped-rows
              scrollable
              scroll-height="560px"
            >
              <Column field="name" :header="t('skills.name')" style="min-width: 180px" />
              <Column field="description" :header="t('skills.description')" style="min-width: 320px">
                <template #body="slotProps">
                  <span class="line-clamp-2 text-sm">{{ slotProps.data.description || t("skills.noDescription") }}</span>
                </template>
              </Column>
              <Column :header="t('skills.dependenciesHeader')" style="min-width: 200px">
                <template #body="slotProps">
                  <div class="flex flex-wrap gap-1">
                    <Tag
                      v-for="dep in slotProps.data.dependencies"
                      :key="dep"
                      :value="dep"
                      severity="secondary"
                    />
                  </div>
                </template>
              </Column>
            </DataTable>
          </SplitterPanel>

          <SplitterPanel :size="56" class="p-3">
            <div v-if="selectedSkill" class="flex h-full flex-col gap-3">
              <div class="flex flex-wrap items-center justify-between gap-2">
                <div>
                  <h3 class="text-lg font-semibold">{{ selectedSkill.name }}</h3>
                  <p class="mt-1 text-sm text-slate-500">{{ selectedSkill.path }}</p>
                  <p class="mt-1 text-sm text-slate-600">{{ selectedSkill.description }}</p>
                </div>

                <div class="flex gap-2">
                  <Button icon="pi pi-folder-open" :label="t('skills.openDir')" outlined @click="openDirectory" />
                  <Button icon="pi pi-file" :label="t('skills.openSkillFile')" outlined @click="openSkillFile" />
                </div>
              </div>

              <div>
                <p class="mb-1 text-sm font-medium text-slate-700">{{ t("skills.scripts") }}</p>
                <div class="flex flex-wrap gap-2">
                  <Tag v-for="script in selectedSkill.scripts" :key="script" :value="script" />
                  <span v-if="selectedSkill.scripts.length === 0" class="text-sm text-slate-500">{{ t("skills.noScripts") }}</span>
                </div>
              </div>

              <div>
                <p class="mb-1 text-sm font-medium text-slate-700">{{ t("skills.dependencies") }}</p>
                <div class="flex flex-wrap gap-2">
                  <Tag
                    v-for="dependency in selectedSkill.dependencies"
                    :key="dependency"
                    :value="dependency"
                    severity="info"
                  />
                </div>
              </div>

              <div class="skill-preview-container relative flex-1 overflow-hidden rounded-lg border border-slate-200">
                <div
                  v-if="detailLoading"
                  class="absolute inset-0 z-10 flex items-center justify-center bg-white/70 dark:bg-slate-950/60"
                >
                  <ProgressSpinner style="width: 40px; height: 40px" stroke-width="6" />
                </div>

                <article class="skill-markdown h-full overflow-auto p-4" v-html="renderedMarkdown" />
              </div>
            </div>

            <div v-else class="flex h-full items-center justify-center text-slate-500">{{ t("skills.chooseSkill") }}</div>
          </SplitterPanel>
        </Splitter>
      </template>
    </Card>
  </section>
</template>

<style scoped>
.skill-markdown :deep(h1),
.skill-markdown :deep(h2),
.skill-markdown :deep(h3),
.skill-markdown :deep(h4) {
  margin: 0.8rem 0 0.5rem;
  font-weight: 700;
}

.skill-markdown :deep(h1) {
  font-size: 1.2rem;
}

.skill-markdown :deep(h2) {
  font-size: 1.05rem;
}

.skill-markdown :deep(p),
.skill-markdown :deep(li) {
  line-height: 1.6;
}

.skill-markdown :deep(a) {
  color: #0284c7;
}

.skill-markdown :deep(ul),
.skill-markdown :deep(ol) {
  padding-left: 1.25rem;
}

.skill-markdown :deep(code) {
  border: 1px solid rgba(148, 163, 184, 0.35);
  border-radius: 0.35rem;
  background: rgba(15, 23, 42, 0.08);
  padding: 0.1rem 0.3rem;
  font-size: 0.85em;
}

.skill-markdown :deep(pre) {
  overflow: auto;
  border-radius: 0.6rem;
  background: #020617;
  color: #cbd5e1;
  padding: 0.75rem;
}

.skill-markdown :deep(pre code) {
  border: 0;
  background: transparent;
  padding: 0;
}

@media (prefers-color-scheme: dark) {
  .skill-preview-container {
    background: rgba(2, 6, 23, 0.78);
  }

  .skill-markdown {
    color: #e2e8f0;
  }

  .skill-markdown :deep(a) {
    color: #38bdf8;
  }

  .skill-markdown :deep(code) {
    border-color: rgba(148, 163, 184, 0.25);
    background: rgba(15, 23, 42, 0.35);
    color: #e2e8f0;
  }
}

@media (prefers-color-scheme: light) {
  .skill-preview-container {
    background: rgba(255, 255, 255, 0.72);
  }
}
</style>
