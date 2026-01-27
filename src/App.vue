<template>
  <div id="app" @contextmenu.prevent>
    <!-- 自定义标题栏 -->
    <TitleBar />

    <!-- 路由视图 -->
    <div class="app-container">
      <router-view />
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import TitleBar from './components/TitleBar.vue'
import { register, unregister } from '@tauri-apps/plugin-global-shortcut'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

const SHORTCUT = 'Alt+T'

const registerShortcut = async () => {
  try {
    // 防止重复注册
    await unregister(SHORTCUT).catch(() => {})

    await register(SHORTCUT, async (event) => {
      // 如果有状态区分，只在按下时触发 (event.state === 'Pressed')
      // 如果没有状态字段，说明只在触发时调用
      if (event && event.state === 'Released') return

      // 获取当前时间
      const now = new Date()
      const year = now.getFullYear()
      const month = String(now.getMonth() + 1).padStart(2, '0')
      const day = String(now.getDate()).padStart(2, '0')
      const hours = String(now.getHours()).padStart(2, '0')
      const minutes = String(now.getMinutes()).padStart(2, '0')
      const seconds = String(now.getSeconds()).padStart(2, '0')
      
      const timeStr = `${year}-${month}-${day}_${hours}-${minutes}-${seconds}`
      
      try {
        // 使用 Tauri 剪贴板插件写入
        await writeText(timeStr)
        ElMessage({
          message: `时间已复制: ${timeStr}`,
          type: 'success',
          duration: 2000
        })
      } catch (err) {
        console.error('Failed to copy time:', err)
        ElMessage.error('复制时间失败')
      }
    })
    console.log(`Global shortcut ${SHORTCUT} registered`)
  } catch (err) {
    console.error('Failed to register global shortcut:', err)
  }
}

onMounted(() => {
  registerShortcut()
})

onUnmounted(async () => {
  await unregister(SHORTCUT).catch(() => {})
})
</script>

<style scoped>
/* 主容器 */
#app {
  height: 100vh;
  width: 100%;
  border: 1px solid #dcdfe6; /* 自定义外框 */
  box-sizing: border-box;
  background-color: #fff;
}

/* 应用容器 */
.app-container {
  height: calc(100vh - 32px); /* 减去标题栏高度 */
  width: 100%;
  margin-top: 32px; /* 给标题栏留出空间 */
}
</style>
