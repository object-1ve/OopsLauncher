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
      @delete="handleContextMenuDelete"
      @hide="hideContextMenu"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import DropZone from '@/components/DropZone.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import { useFiles } from '@/composables/useFiles'

const { 
  currentFiles, 
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
  selectedFileId: null
})

// 方法：处理文件添加
const handleFileAdd = async (fileList) => {
  await processFiles(fileList)
}

// 方法：处理右键菜单显示
const handleContextMenu = (data) => {
  contextMenu.value = {
    visible: true,
    x: data.event.clientX,
    y: data.event.clientY,
    selectedFileId: data.fileId
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
