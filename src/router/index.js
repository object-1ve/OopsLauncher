import { createRouter, createWebHistory } from 'vue-router'
import BasicLayout from '@/layout/basic.vue'
import Home from '@/views/Home.vue'
import Settings from '@/views/Settings.vue'

const routes = [
  {
    path: '/',
    component: BasicLayout,
    children: [
      {
        path: '',
        name: 'Home',
        component: Home
      }
    ]
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
