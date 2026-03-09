<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

import Button from "primevue/button";
import ConfirmDialog from "primevue/confirmdialog";
import Dropdown from "primevue/dropdown";
import Toast from "primevue/toast";

import { useI18n } from "./i18n";

interface NavItem {
  label: string;
  icon: string;
  path: string;
}

const router = useRouter();
const route = useRoute();
const { t, locale, setLocale, localeOptions } = useI18n();

const navItems = computed<NavItem[]>(() => [
  { label: t("app.nav.dashboard"), icon: "pi pi-chart-bar", path: "/dashboard" },
  { label: t("app.nav.config"), icon: "pi pi-cog", path: "/config" },
  { label: t("app.nav.sandbox"), icon: "pi pi-shield", path: "/sandbox" },
  { label: t("app.nav.rules"), icon: "pi pi-sliders-h", path: "/rules" },
  { label: t("app.nav.skills"), icon: "pi pi-bolt", path: "/skills" },
  { label: t("app.nav.snapshots"), icon: "pi pi-history", path: "/snapshots" },
]);

const currentPath = computed(() => route.path);

function go(path: string): void {
  if (path !== currentPath.value) {
    void router.push(path);
  }
}

function onLocaleChange(next: "zh-CN" | "en-US"): void {
  setLocale(next);
}
</script>

<template>
  <Toast position="top-right" />
  <ConfirmDialog />

  <div class="min-h-screen bg-[var(--app-bg)] text-slate-900">
    <div class="mx-auto flex max-w-[1480px] flex-col gap-5 p-4 md:flex-row md:p-6">
      <aside
        class="w-full rounded-2xl border border-slate-200 bg-white/80 p-3 shadow-sm backdrop-blur md:sticky md:top-6 md:h-[calc(100vh-3rem)] md:w-[240px] md:p-4"
      >
        <div class="mb-5 rounded-xl bg-slate-900 px-4 py-3 text-white">
          <h1 class="text-lg font-semibold">{{ t("app.title") }}</h1>
          <p class="mt-1 text-xs text-slate-300">{{ t("app.slogan") }}</p>
        </div>

        <div class="mb-4">
          <label class="mb-1 block text-xs text-slate-500">{{ t("app.language") }}</label>
          <Dropdown
            :model-value="locale"
            :options="localeOptions"
            option-label="label"
            option-value="value"
            class="w-full"
            @update:model-value="onLocaleChange"
          />
        </div>

        <div class="space-y-2">
          <Button
            v-for="item in navItems"
            :key="item.path"
            :icon="item.icon"
            :label="item.label"
            class="w-full justify-start"
            :outlined="currentPath !== item.path"
            :severity="currentPath === item.path ? 'contrast' : 'secondary'"
            @click="go(item.path)"
          />
        </div>
      </aside>

      <main class="min-h-[60vh] flex-1 overflow-hidden rounded-2xl border border-slate-200 bg-white p-4 shadow-sm md:p-6">
        <RouterView />
      </main>
    </div>
  </div>
</template>
