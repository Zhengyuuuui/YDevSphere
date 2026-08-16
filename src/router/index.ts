import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "welcome",
      component: () => import("@/pages/Welcome.vue"),
    },
    // Sidebar 布局下的页面（Overview / Projects / Recent / Settings）
    {
      path: "/",
      component: () => import("@/layouts/AppLayout.vue"),
      children: [
        {
          path: "overview",
          name: "overview",
          component: () => import("@/pages/Overview.vue"),
        },
        {
          path: "projects",
          name: "projects",
          component: () => import("@/pages/Projects.vue"),
        },
        {
          path: "recent",
          name: "recent",
          component: () => import("@/pages/Recent.vue"),
        },
        {
          path: "settings",
          name: "settings",
          component: () => import("@/pages/Settings.vue"),
        },
        {
          path: "project/:id",
          name: "project-detail",
          component: () => import("@/pages/ProjectDetail.vue"),
        },
      ],
    },
    // 旧路由兼容：/dashboard → 重定向到 /projects（表格页）
    {
      path: "/dashboard",
      redirect: { name: "projects" },
    },
  ],
});

export { router };
