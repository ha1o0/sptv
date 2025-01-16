import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "Home",
    component: () => import("./components/Home.vue"),
  },
  {
    path: "/config",
    name: "Config",
    component: () => import("./components/Config.vue"),
  },
  {
    path: "/favorite",
    name: "Favorite",
    component: () => import("./components/Favorite.vue"),
  },
  {
    path: "/setting",
    name: "Setting",
    component: () => import("./components/Setting.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export { routes, router };