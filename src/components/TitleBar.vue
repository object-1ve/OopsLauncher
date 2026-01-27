<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-content">
      <div class="app-icon">
        <!-- 这里可以放置应用图标 -->
        <span class="icon-emoji">🚀</span>
      </div>
      <div class="app-title">OopsLauncher</div>
    </div>
    
    <div class="titlebar-controls">
      <div class="titlebar-button minimize" @click="minimize">
        <el-icon><Minus /></el-icon>
      </div>
      <div class="titlebar-button maximize" @click="toggleMaximize">
        <el-icon v-if="isMaximized"><CopyDocument /></el-icon>
        <el-icon v-else><FullScreen /></el-icon>
      </div>
      <div class="titlebar-button close" @click="close">
        <el-icon><Close /></el-icon>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { Minus, CopyDocument, FullScreen, Close } from '@element-plus/icons-vue'

const appWindow = new Window('main')
const isMaximized = ref(false)

const minimize = async () => {
  await appWindow.minimize()
}

const toggleMaximize = async () => {
  await appWindow.toggleMaximize()
}

const close = async () => {
  await appWindow.close()
}

onMounted(async () => {
  // 监听窗口大小变化以更新最大化状态图标
  // 注意：Tauri v2 的事件监听可能需要调整，这里先简单实现
  // 实际开发中可以通过监听 tauri://resize 事件或定期检查
  
  // 简单的状态检查
  isMaximized.value = await appWindow.isMaximized()
  
  // 监听最大化事件
  // appWindow.listen('tauri://resize', async () => {
  //   isMaximized.value = await appWindow.isMaximized()
  // })
})
</script>

<style scoped>
.titlebar {
  height: 32px;
  user-select: none;
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 9999;

}

.titlebar-content {
  display: flex;
  align-items: center;
  padding-left: 10px;
  pointer-events: none; /* 让点击穿透到 drag-region */
}

.app-icon {
  margin-right: 8px;
  display: flex;
  align-items: center;
}

.icon-emoji {
  font-size: 16px;
}

.app-title {
  font-size: 12px;
  font-weight: 500;
  color: #333;
}

.titlebar-controls {
  display: flex;
  height: 100%;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 46px;
  height: 100%;
  cursor: pointer;
  transition: background-color 0.2s;
  color: #333;
}

.titlebar-button:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.titlebar-button.close:hover {
  background-color: #e81123;
  color: white;
}

.titlebar-button .el-icon {
  font-size: 14px;
}
</style>