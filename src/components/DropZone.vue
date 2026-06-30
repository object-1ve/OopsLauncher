<template>
  <el-main class="drop-zone-main">
    <!-- 合并的拖拽和展示区域 -->
    <div class="drop-zone" :class="{ dragover: isDragOver }" @dragover="handleDragOver" @dragleave="handleDragLeave"
      @drop="handleDrop" @contextmenu.prevent="handleEmptyContextMenu">
      <!-- 图标展示区域 -->
      <div class="icons-container" v-if="pinnedCurrentFiles.length > 0 || currentFiles.length > 0">
        <div v-if="pinnedCurrentFiles.length > 0" class="group-section pinned-section">
          <div class="group-title">📌 置顶</div>
          <div class="row-container" :class="settings.appearance.itemLayout">
            <FileIcon v-for="file in pinnedCurrentFiles" :key="file.id" :file="file" class="file-icon-item"
              @open="handleFileOpen" @delete="handleFileDelete" @contextmenu="handleFileContextMenu" />
          </div>
        </div>
        <div v-if="classifyMethod !== 'none'">
          <div v-for="group in groupedCurrentFiles" :key="group.type" class="group-section">
            <div class="group-title">{{ group.label }}</div>
            <div class="row-container" :class="settings.appearance.itemLayout">
              <FileIcon v-for="file in group.files" :key="file.id" :file="file" class="file-icon-item"
                @open="handleFileOpen" @delete="handleFileDelete" @contextmenu="handleFileContextMenu" />
            </div>
          </div>
        </div>
        <div v-else class="row-container" :class="settings.appearance.itemLayout">
          <FileIcon v-for="file in currentFiles" :key="file.id" :file="file" class="file-icon-item"
            @open="handleFileOpen" @delete="handleFileDelete" @contextmenu="handleFileContextMenu" />
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-icon">📂</div>
        <div class="empty-title">暂无文件</div>
        <div class="empty-hint">拖拽文件到此处，或右键添加</div>
      </div>

      <!-- 拖拽覆盖层 -->
      <div v-if="isDragOver" class="dragover-overlay">
        <div class="dragover-content">
          <div class="dragover-icon">📥</div>
          <div class="dragover-text">释放以添加文件</div>
        </div>
      </div>

      <!-- 隐藏的文件输入 -->
      <input ref="fileInput" type="file" multiple class="file-input" tabindex="-1" @change="handleFileSelect" />
    </div>
  </el-main>
</template>

<script setup>
import { ref } from "vue";
import FileIcon from "./FileIcon.vue";
import { useSettings } from "@/composables/useSettings";

const { settings } = useSettings();

import { isTauri } from "@/utils/env";

// Props
const props = defineProps({
  pinnedCurrentFiles: {
    type: Array,
    default: () => [],
  },
  currentFiles: {
    type: Array,
    default: () => [],
  },
  groupedCurrentFiles: {
    type: Array,
    default: () => [],
  },
  classifyMethod: {
    type: String,
    default: 'none',
  },
});
// Emits
const emit = defineEmits(["fileAdd", "fileOpen", "fileDelete", "contextmenu"]);

// 响应式数据
const isDragOver = ref(false);

// 方法：处理文件选择
const handleFileSelect = async (e) => {
  const selectedFiles = Array.from(e.target.files);
  emit("fileAdd", selectedFiles);
  // 清空文件输入
  e.target.value = "";
};

// 方法：处理拖拽经过
const handleDragOver = (e) => {
  e.preventDefault();
  isDragOver.value = true;
};

// 方法：处理拖拽离开
const handleDragLeave = (e) => {
  e.preventDefault();
  // 确保不是进入子元素触发的 leave
  if (e.currentTarget === e.target) {
    isDragOver.value = false;
  }
};

// 方法：处理拖拽放下
const handleDrop = async (e) => {
  e.preventDefault();
  isDragOver.value = false;

  // 在 Tauri 环境下，拖拽添加由 useFiles 中的 tauri://drag-drop 监听器统一处理
  // 否则会造成重复添加（浏览器 drop 事件和 Tauri 原生事件同时触发）
  if (isTauri()) return;

  const droppedFiles = Array.from(e.dataTransfer.files);
  emit("fileAdd", droppedFiles);
};

// 方法：处理文件打开
const handleFileOpen = (file) => {
  emit("fileOpen", file);
};

// 方法：处理文件删除
const handleFileDelete = (fileId) => {
  emit("fileDelete", fileId);
};

// 方法：处理文件右键菜单
const handleFileContextMenu = (data) => {
  emit("contextmenu", data);
};

// 方法：处理空白区域右键菜单
const handleEmptyContextMenu = (e) => {
  emit("contextmenu", { event: e, fileId: null });
};
</script>

<style scoped>
.drop-zone-main {
  padding: 0;
  height: 100%;
  overflow: hidden;
}

.file-icon-item {
  margin: 2px;
}

/* 拖拽区域样式 */
.drop-zone {
  height: 100%;
  transition: all var(--app-transition, 0.2s);
  padding: 0;
  position: relative;
}

.drop-zone.dragover {
  background-color: rgba(64, 158, 255, 0.02);
}

/* 图标容器样式 */
.icons-container {
  height: 100%;
  overflow-y: auto;
  padding: 8px 8px 20px 8px;
}

.row-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
}

.row-container.list {
  flex-direction: column;
}

.group-section {
  margin-bottom: 16px;
}

.group-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--app-text-muted, #909399);
  padding: 0 4px 6px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.pinned-section .group-title {
  color: #e6a23c;
}

/* 图标项样式覆盖 */
.file-icon-item {
  margin: 0;
}

.list .file-icon-item {
  margin: 0 0 2px 0;
}

/* 空状态样式 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 4px;
  opacity: 0.6;
}

.empty-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text-secondary, #606266);
}

.empty-hint {
  font-size: 13px;
  color: var(--app-text-muted, #909399);
}

/* 拖拽覆盖层 */
.dragover-overlay {
  position: absolute;
  inset: 0;
  background-color: rgba(64, 158, 255, 0.06);
  border: 2px dashed rgba(64, 158, 255, 0.4);
  border-radius: var(--app-radius-md, 8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  pointer-events: none;
  margin: 4px;
}

.dragover-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.dragover-icon {
  font-size: 36px;
}

.dragover-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--app-active-border, #409eff);
}

/* 滚动条美化 */
.icons-container::-webkit-scrollbar {
  width: 4px;
}

.icons-container::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

.icons-container::-webkit-scrollbar-thumb:hover {
  background-color: rgba(0, 0, 0, 0.15);
}

.icons-container::-webkit-scrollbar-track {
  background-color: transparent;
}

/* 文件输入样式 */
.file-input {
  display: none;
}
</style>
