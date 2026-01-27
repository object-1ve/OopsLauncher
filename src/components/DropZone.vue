<template>
  <el-main class="drop-zone-main">
    <!-- 合并的拖拽和展示区域 -->
    <div 
      class="drop-zone"
      :class="{ 'dragover': isDragOver }"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
      @click.self="handleSelectClick"
    >
      <!-- 图标展示区域 -->
      <div class="icons-container" v-if="currentFiles.length > 0">
        <el-row :gutter="10">
          <FileIcon 
            v-for="file in currentFiles" 
            :key="file.id"
            :file="file"
            @open="handleFileOpen"
            @delete="handleFileDelete"
            @contextmenu="handleFileContextMenu"
          />
        </el-row>
      </div>
      <!-- 隐藏的文件输入 -->
      <input 
        ref="fileInput"
        type="file" 
        multiple 
        class="file-input"
        @change="handleFileSelect"
      />
    </div>
  </el-main>
</template>

<script setup>
import { ref } from 'vue'
import FileIcon from './FileIcon.vue'

// Props
const props = defineProps({
  currentFiles: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits(['fileAdd', 'fileOpen', 'fileDelete', 'contextmenu'])

// 响应式数据
const fileInput = ref(null)
const isDragOver = ref(false)

// 方法：处理文件选择点击
const handleSelectClick = () => {
  if (fileInput.value) {
    fileInput.value.click()
  }
}

// 方法：处理文件选择
const handleFileSelect = async (e) => {
  const selectedFiles = Array.from(e.target.files)
  emit('fileAdd', selectedFiles)
  // 清空文件输入
  e.target.value = ''
}

// 方法：处理拖拽经过
const handleDragOver = (e) => {
  e.preventDefault()
  isDragOver.value = true
}

// 方法：处理拖拽离开
const handleDragLeave = (e) => {
  e.preventDefault()
  // 确保不是进入子元素触发的 leave
  if (e.currentTarget === e.target) {
    isDragOver.value = false
  }
}

// 方法：处理拖拽放下
const handleDrop = async (e) => {
  e.preventDefault()
  isDragOver.value = false
  
  const droppedFiles = Array.from(e.dataTransfer.files)
  emit('fileAdd', droppedFiles)
}

// 方法：处理文件打开
const handleFileOpen = (file) => {
  emit('fileOpen', file)
}

// 方法：处理文件删除
const handleFileDelete = (fileId) => {
  emit('fileDelete', fileId)
}

// 方法：处理文件右键菜单
const handleFileContextMenu = (data) => {
  emit('contextmenu', data)
}
</script>

<style scoped>


/* 拖拽区域样式 */
.drop-zone {
  height: 100%;
  transition: all 0.3s ease;
}

.drop-zone.dragover {
  background-color: rgba(64, 158, 255, 0.05);
  border-color: rgba(64, 158, 255, 0.05);
  border-style: dashed;
}

/* 图标容器样式 */
.icons-container {
  height: 100%;
  overflow-y: auto;
  padding: 10px;
}

/* 滚动条美化 */
.icons-container::-webkit-scrollbar {
  width: 6px;
}
.icons-container::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 3px;
}
.icons-container::-webkit-scrollbar-track {
  background-color: transparent;
}
/* 文件输入样式 */
.file-input {
  display: none;
}
</style>