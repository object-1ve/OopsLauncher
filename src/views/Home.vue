<template>
  <div class="home-view">
    <!-- 使用DropZone组件 -->
    <DropZone 
      :currentFiles="currentFiles"
      @fileAdd="handleFileAdd"
      @fileOpen="openFile"
      @fileDelete="deleteFile"
      @contextmenu="handleContextMenu"
    />

    <!-- 使用ContextMenu组件 -->
    <ContextMenu 
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :selectedFileId="contextMenu.selectedFileId"
      :selectedFile="contextMenu.selectedFile"
      @delete="handleContextMenuDelete"
      @hide="hideContextMenu"
      @openLocation="handleOpenLocation"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import DropZone from '@/components/DropZone.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import { useFiles } from '@/composables/useFiles'
import { invoke } from '@tauri-apps/api/core'

const { 
  currentFiles, 
  filesByCategory,
  processFiles, 
  deleteFile, 
  openFile,
  loadFiles,
  setupTauriListeners 
} = useFiles()

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  selectedFileId: null,
  selectedFile: null
})

// 方法：处理文件添加
const handleFileAdd = async (fileList) => {
  await processFiles(fileList)
}

// 方法：处理右键菜单显示
const handleContextMenu = (data) => {
  // 查找对应的文件对象
  let selectedFile = null
  if (data.fileId) {
    // 遍历所有分类查找文件
    for (const category in filesByCategory.value) {
      const files = filesByCategory.value[category]
      if (files) {
        const file = files.find(f => f.id === data.fileId)
        if (file) {
          selectedFile = file
          break
        }
      }
    }
  }
  
  contextMenu.value = {
    visible: true,
    x: data.event.clientX,
    y: data.event.clientY,
    selectedFileId: data.fileId,
    selectedFile: selectedFile
  }
}

// 方法：隐藏右键菜单
const hideContextMenu = () => {
  contextMenu.value.visible = false
}

// 方法：处理右键菜单删除
const handleContextMenuDelete = (fileId) => {
  deleteFile(fileId)
  hideContextMenu()
}

// 方法：处理打开文件所在位置
const handleOpenLocation = async (file) => {
  try {
    if (file && file.path) {
      // 打开文件所在位置并选择文件
      if (window.__TAURI_INTERNALS__?.invoke) {
        await invoke('open_file_location', { path: file.path })
      } else {
        // 浏览器环境下的降级处理
        console.log('Opening file location:', file.path)
        alert(`在浏览器环境中无法打开文件位置: ${file.path}`)
      }
    }
  } catch (error) {
    console.error('Failed to open file location:', error)
    alert(`打开文件所在位置失败: ${error.message}`)
  }
}

// 方法：点击空白处隐藏右键菜单
const handleDocumentClick = () => {
  hideContextMenu()
}

onMounted(async () => {
  console.log('Home View initialized')
  await loadFiles()
  await setupTauriListeners()
  document.addEventListener('click', handleDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
})
</script>

<style scoped>
.home-view {
  height: 100vh;
  width: 100%;
}
</style>
