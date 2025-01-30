import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: '/',
    component: () => import("./layouts/Main.vue"),
    children: [
      {
        path: "",
        name: "ConfigList",
        component: () => import("./pages/Config/List.vue"),
      },
      {
        path: "config/:id",
        name: "ConfigDetail",
        component: () => import("./pages/Config/Detail.vue"),
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
