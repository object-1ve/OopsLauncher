<template>
  <div class="file-icon-wrapper">
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
  </div>
</template>

<script setup>

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

// 方法：处理右键菜单
const handleContextMenu = (e) => {
  e.preventDefault()
  emit('contextmenu', { event: e, fileId: props.file.id })
}
</script>

<style scoped>
.file-icon-wrapper {
  /* 移除 margin-bottom，布局由父级控制 */
}

.icon-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  width: 80px;  /* 缩小宽度 */
  height: 100px; /* 缩小高度 */
  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-item:hover {
  background-color: rgba(0, 0, 0, 0.05);
  /* 移除 width/height 的重设，保持一致 */
}

.icon-wrapper {
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  height: 50px;
}

.file-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background-color: transparent; /* 确保背景透明 */
  mix-blend-mode: multiply; /* 可选：如果是白色背景的jpg，可以尝试混合 */
}

.file-icon-emoji {
  font-size: 48px;
  line-height: 1;
}

.file-name {
  font-size: 13px;
  color: #555;
  text-align: center;
  width: 100%;  padding: 0 4px;
  
  /* 多行显示逻辑 */
  display: -webkit-box;
  -webkit-line-clamp: 2; /* 限制为2行 */
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: normal; /* 允许换行 */
  line-height: 1.4; /* 优化行高 */
  height: 36px; /* 固定高度，避免跳动 (13px * 1.4 * 2 ≈ 36.4px) */
}

.icon-item:hover {
  opacity: 1;
}
</style>