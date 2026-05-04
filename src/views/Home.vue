<template>
  <div class="home-view">
    <!-- 使用DropZone组件 -->
    <DropZone 
      :pinnedCurrentFiles="pinnedCurrentFiles"
      :currentFiles="currentFiles"
      :groupedCurrentFiles="groupedCurrentFiles"
      :classifyMethod="classifyMethod"
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
      :sortMethod="sortMethod"
      :sortOrder="sortOrder"
      :classifyMethod="classifyMethod"
      :showFileName="settings.appearance.showFileName"
      :categories="allCategories"
      :showLocateToCategory="currentCategory === SPECIAL_CATEGORIES.ALL_FILES && !!contextMenu.selectedFile?.category"
      :showPinToggle="!!contextMenu.selectedFile?.category"
      :selectedFileIsPinned="!!contextMenu.selectedFile?.isPinned"
      @delete="handleContextMenuDelete"
      @hide="hideContextMenu"
      @openLocation="handleOpenLocation"
      @openTerminal="handleOpenTerminal"
      @copyPath="handleCopyPath"
      @openWith="handleOpenWith"
      @editInfo="handleEditInfo"
      @sort="handleSort"
      @classify="handleClassify"
      @toggleDisplay="handleToggleDisplay"
      @copyToCategory="handleCopyToCategory"
      @locateToCategory="handleLocateToCategory"
      @togglePin="handleTogglePin"
    />
    
    <!-- 文件信息编辑弹窗 -->
    <FileInfoDialog
      v-model:visible="fileInfoDialog.visible"
      :current-file="fileInfoDialog.currentFile"
      @save="handleSaveFileInfo"
      @cancel="handleCancelFileInfo"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import DropZone from '@/components/DropZone.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import FileInfoDialog from '@/components/FileInfoDialog.vue'
import { useFiles, SPECIAL_CATEGORIES } from '@/composables/useFiles'
import { useSettings } from '@/composables/useSettings'
import { useContextMenu } from '@/composables/useContextMenu'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { ElMessage } from 'element-plus'

const { 
  currentCategory,
  pinnedCurrentFiles,
  currentFiles, 
  groupedCurrentFiles,
  filesByCategory,
  processFiles, 
  deleteFile, 
  openFile,
  loadFiles,
  setupTauriListeners,
  saveFiles,
  switchCategory,
  sortMethod,
  sortOrder,
  classifyMethod,
  allCategories,
  copyFileToCategory,
  togglePinFile
} = useFiles()

const { settings } = useSettings()
const { registerMenu, unregisterMenu } = useContextMenu()

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  selectedFileId: null,
  selectedFile: null
})

// 文件信息编辑弹窗状态
const fileInfoDialog = ref({
  visible: false,
  currentFile: null
})

// 方法：处理文件添加
const handleFileAdd = async (fileList) => {
  const result = await processFiles(fileList)
  
  if (result.error === 'cannot_add_to_special_category') {
    ElMessage.warning('不能直接向“全部文件”分类中添加文件')
    return
  }

  // 显示添加结果的消息
  if (result.addedCount > 0) {
    if (result.failedCount === 0 && result.existingCount === 0) {
      ElMessage.success(`成功添加 ${result.addedCount} 个文件`)
    } else {
      let msg = `成功添加 ${result.addedCount} 个文件`
      if (result.existingCount > 0) msg += `，${result.existingCount} 个文件已存在`
      if (result.failedCount > 0) msg += `，${result.failedCount} 个文件添加失败`
      ElMessage.success(msg)
    }
  } else if (result.failedCount > 0) {
    ElMessage.error(`${result.failedCount} 个文件添加失败，请检查文件是否存在或权限是否足够`)
  } else if (result.existingCount > 0) {
    ElMessage.warning(`所有 ${result.existingCount} 个文件都已存在，添加失败`)
  }
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
  
  registerMenu(hideContextMenu)
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
      // 检测是否在 Tauri 环境
      const isTauri = !!window.__TAURI_INTERNALS__;
      
      // 打开文件所在位置并选择文件
      if (isTauri) {
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

const handleOpenTerminal = async (file) => {
  try {
    if (file && file.path) {
      const isTauri = !!window.__TAURI_INTERNALS__;

      if (isTauri) {
        await invoke('open_terminal', { path: file.path })
      } else {
        alert(`在浏览器环境中无法打开终端: ${file.path}`)
      }
    }
  } catch (error) {
    console.error('Failed to open terminal:', error)
    alert(`在终端打开失败: ${error.message}`)
  }
}

// 方法：处理复制路径
const handleCopyPath = async (file) => {
  try {
    if (file && file.path) {
      await writeText(file.path)
      ElMessage.success('已复制到剪贴板')
    }
  } catch (error) {
    console.error('Failed to copy path:', error)
    ElMessage.error('复制路径失败')
  }
}

const handleOpenWith = async (file) => {
  try {
    if (file && file.path) {
      const isTauri = !!window.__TAURI_INTERNALS__

      if (isTauri) {
        await invoke('open_with_dialog', { path: file.path })
      } else {
        alert(`当前环境不支持打开方式: ${file.path}`)
      }
    }
  } catch (error) {
    console.error('Failed to open with dialog:', error)
    alert(`打开方式失败: ${error.message}`)
  }
}

// 方法：处理文件信息编辑
const handleEditInfo = (file) => {
  fileInfoDialog.value = {
    visible: true,
    currentFile: file
  }
}

// 方法：处理文件信息保存
const handleSaveFileInfo = async (updatedFile) => {
  try {
    // 在 filesByCategory 中找到并更新对应的文件
    for (const category in filesByCategory.value) {
      const files = filesByCategory.value[category]
      if (files) {
        const fileIndex = files.findIndex(f => f.id === updatedFile.id)
        if (fileIndex !== -1) {
          files[fileIndex] = updatedFile
          break
        }
      }
    }
    
    // 保存更改到数据库
    await saveFiles()
    
    // 显示保存成功的消息
    ElMessage.success('文件信息保存成功')
  } catch (error) {
    console.error('保存文件信息失败:', error)
    ElMessage.error('保存文件信息失败')
  }
}

// 方法：处理文件信息取消编辑
const handleCancelFileInfo = () => {
  fileInfoDialog.value.visible = false
  fileInfoDialog.value.currentFile = null
}

// 方法：处理排序
const handleSort = (method) => {
  if (sortMethod.value === method) {
    // 如果点击的是当前已选中的排序方法，则切换升降序
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    // 如果点击的是新的排序方法，默认使用升序
    sortMethod.value = method
    sortOrder.value = 'asc'
  }
}

const handleClassify = (method) => {
  classifyMethod.value = method
}

// 方法：处理显示配置切换
const handleToggleDisplay = (key) => {
  if (key === 'showFileName') {
    settings.value.appearance.showFileName = !settings.value.appearance.showFileName
  }
}

// 方法：处理复制文件到其他分类
const handleCopyToCategory = async ({ file, targetCategoryId }) => {
  const result = await copyFileToCategory(file, targetCategoryId)
  const targetCat = allCategories.value.find(c => c.id === targetCategoryId)
  if (result.success) {
    ElMessage.success(`已复制到「${targetCat?.name || targetCategoryId}」`)
  } else if (result.reason === 'duplicate') {
    ElMessage.warning(`文件已存在于「${targetCat?.name || targetCategoryId}」中`)
  } else if (result.reason === 'cannot_copy_to_special_category') {
    ElMessage.warning('不能直接向“全部文件”分类中复制文件')
  }
  hideContextMenu()
}

const handleLocateToCategory = (file) => {
  if (!file?.id || !file?.category) return
  window.dispatchEvent(new CustomEvent('locate-file', {
    detail: {
      fileId: file.id,
      categoryId: file.category
    }
  }))
  hideContextMenu()
}

const handleTogglePin = async (file) => {
  const result = await togglePinFile(file)
  if (!result.success) {
    ElMessage.error('置顶操作失败')
    return
  }
  ElMessage.success(result.isPinned ? '已置顶' : '已取消置顶')
  hideContextMenu()
}

// 方法：点击空白处隐藏右键菜单
const handleDocumentClick = () => {
  hideContextMenu()
}

let locateHighlightTimer = null

const clearLocateHighlight = () => {
  if (locateHighlightTimer) {
    clearTimeout(locateHighlightTimer)
    locateHighlightTimer = null
  }
  const highlighted = document.querySelector('.icon-item.locating-highlight')
  if (highlighted) {
    highlighted.classList.remove('locating-highlight')
  }
}

const handleLocateFileEvent = async (event) => {
  const { fileId, categoryId } = event.detail || {}
  if (!fileId) return

  if (categoryId) {
    switchCategory(categoryId)
  }

  await nextTick()

  const target = document.querySelector(`.icon-item[data-file-id="${fileId}"]`)
  if (!target) return

  clearLocateHighlight()
  target.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'nearest' })
  target.classList.add('locating-highlight')
  locateHighlightTimer = setTimeout(() => {
    target.classList.remove('locating-highlight')
    locateHighlightTimer = null
  }, 1600)
}

onMounted(async () => {
  console.log('Home View initialized')
  await loadFiles()
  await setupTauriListeners()
  document.addEventListener('click', handleDocumentClick)
  window.addEventListener('locate-file', handleLocateFileEvent)
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
  window.removeEventListener('locate-file', handleLocateFileEvent)
  clearLocateHighlight()
})
</script>

<style scoped>
.home-view {
  height: 100vh;
  width: 100%;
}
</style>
