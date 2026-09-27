import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/HomeView.vue'),
      meta: { titleKey: 'nav.home', subtitleKey: 'app.subtitle' },
    },
    {
      path: '/toolbox',
      name: 'toolbox',
      component: () => import('@/views/ToolboxView.vue'),
      meta: { titleKey: 'nav.toolbox', subtitleKey: 'toolbox.subtitle' },
    },
    // 工作台已并入工具箱：旧地址重定向，避免书签/历史链接白屏
    { path: '/workbench', redirect: '/toolbox' },
    {
      path: '/image-story',
      name: 'image-story',
      component: () => import('@/views/ImageStoryView.vue'),
      meta: { titleKey: 'nav.imageStory', subtitleKey: 'imageStory.subtitle' },
    },
    {
      path: '/tasks',
      name: 'tasks',
      component: () => import('@/views/TasksView.vue'),
      meta: { titleKey: 'nav.tasks', subtitleKey: 'task.subtitle' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { titleKey: 'nav.settings', subtitleKey: 'settings.subtitle' },
    },
  ],
})

export default router
