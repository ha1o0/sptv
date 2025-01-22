import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: '/',
    component: () => import("./layouts/Main.vue"),
    children: [
      {
        path: "",
        name: "Config",
        component: () => import("./pages/Config.vue"),
      },
      {
        path: "player",
        name: "Player",
        component: () => import("./pages/Player.vue"),
      },
      {
        path: "favorite",
        name: "Favorite",
        component: () => import("./pages/Favorite.vue"),
      },
      {
        path: "setting",
        name: "Setting",
        component: () => import("./pages/Setting.vue"),
      }
    ],
  },
  {
    path: "/window",
    component: () => import("./layouts/Blank.vue"),
    children: [
      { path: "player", name: "player-window", component: () => import("./pages/Player.vue") },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export { routes, router };
