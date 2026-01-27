import { createRouter, createWebHistory } from 'vue-router'
import BasicLayout from '@/layout/basic.vue'
import Home from '@/views/Home.vue'

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
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
