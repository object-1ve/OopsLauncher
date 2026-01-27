<template>
  <el-col 
    :xs="12" :sm="8" :md="6" :lg="4" :xl="3"
    class="file-icon-col"
  >
    <div 
      class="icon-item"
      @click="handleFileClick"
      @contextmenu="handleContextMenu"
    >

      <!-- 显示真实图标或emoji -->
      <div class="icon-wrapper">
        <img 
          v-if="file.icon && file.icon.startsWith('data:image/')"
          :src="file.icon" 
          :alt="file.name"
          class="file-icon-img"
        />
        <div 
          v-else
          class="file-icon-emoji"
        >
          {{ file.icon || '📄' }}
        </div>
      </div>
      
      <div class="file-name" :title="file.name">
        {{ file.name }}
      </div>
    </div>
  </el-col>
</template>

<script setup>
import { Close } from '@element-plus/icons-vue'

// Props
const props = defineProps({
  file: {
    type: Object,
    required: true
  }
})

// Emits
const emit = defineEmits(['open', 'delete', 'contextmenu'])

// 方法：处理文件点击
const handleFileClick = () => {
  emit('open', props.file)
}

// 方法：处理删除点击
const handleDeleteClick = () => {
  emit('delete', props.file.id)
}

// 方法：处理右键菜单
const handleContextMenu = (e) => {
  e.preventDefault()
  emit('contextmenu', { event: e, fileId: props.file.id })
}
</script>

<style scoped>
.file-icon-col {
  margin-bottom: 16px;
}

.icon-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 8px;
  border-radius: 8px;

  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-item:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.icon-wrapper {
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
}

.file-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.file-icon-emoji {
  font-size: 48px;
  line-height: 1;
}

.file-name {
  font-size: 13px;
  color: #555;
  text-align: center;
  width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 4px;
}

.icon-item:hover {
  opacity: 1;
}
</style>