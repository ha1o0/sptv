import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "Config",
    component: () => import("./pages/Config.vue"),
  },
  {
    path: "/player",
    name: "Player",
    component: () => import("./pages/Player.vue"),
  },
  {
    path: "/favorite",
    name: "Favorite",
    component: () => import("./pages/Favorite.vue"),
  },
  {
    path: "/setting/general",
    name: "Setting",
    component: () => import("./pages/Setting.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export { routes, router };