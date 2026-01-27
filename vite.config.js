import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src')
    }
  },
  // vite 配置
  server: {
    port: 5173
  },
  build: {
    // 简化构建配置
    rollupOptions: {
      // 移除外部依赖配置
    }
  }
})