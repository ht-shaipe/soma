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
      path: '/workbench',
      name: 'workbench',
      component: () => import('@/views/WorkbenchView.vue'),
      meta: { titleKey: 'nav.workbench', subtitleKey: 'workbench.subtitle' },
    },
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
