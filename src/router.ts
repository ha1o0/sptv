import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "Home",
    component: () => import("./pages/Home.vue"),
  },
  {
    path: "/config",
    name: "Config",
    component: () => import("./pages/Config.vue"),
  },
  {
    path: "/favorite",
    name: "Favorite",
    component: () => import("./pages/Favorite.vue"),
  },
  {
    path: "/setting",
    name: "Setting",
    component: () => import("./pages/Setting.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export { routes, router };