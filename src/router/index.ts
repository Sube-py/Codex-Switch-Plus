import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/dashboard" },
  {
    path: "/dashboard",
    name: "dashboard",
    component: () => import("../pages/DashboardPage.vue"),
  },
  {
    path: "/config",
    name: "config",
    component: () => import("../pages/ConfigPage.vue"),
  },
  {
    path: "/sandbox",
    name: "sandbox",
    component: () => import("../pages/SandboxPage.vue"),
  },
  {
    path: "/rules",
    name: "rules",
    component: () => import("../pages/RulesPage.vue"),
  },
  {
    path: "/skills",
    name: "skills",
    component: () => import("../pages/SkillsPage.vue"),
  },
  {
    path: "/snapshots",
    name: "snapshots",
    component: () => import("../pages/SnapshotsPage.vue"),
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
