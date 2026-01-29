<template>
  <!-- 右键菜单 -->
  <div
    v-if="visible"
    class="context-menu"
    :style="{
      left: x + 'px',
      top: y + 'px',
    }"
  >
    <ul class="context-menu-list">
      <li @click="handleOpenLocation" class="context-menu-item">
        <el-icon><Folder /></el-icon> 打开文件所在位置
      </li>
      <li @click="handleEditInfo" class="context-menu-item">
        <el-icon><Edit /></el-icon> 编辑信息
      </li>
      <li @click="handleDelete" class="context-menu-item">
        <el-icon><Delete /></el-icon> 删除
      </li>
    </ul>
  </div>
</template>

<script setup>
import { ref } from "vue";

// Props
const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  x: {
    type: Number,
    default: 0,
  },
  y: {
    type: Number,
    default: 0,
  },
  selectedFileId: {
    type: String,
    default: null,
  },
  selectedFile: {
    type: Object,
    default: null,
  },
});

// Emits
const emit = defineEmits(["delete", "hide", "openLocation", "editInfo"]);

// 方法：处理删除
const handleDelete = () => {
  if (props.selectedFileId) {
    emit("delete", props.selectedFileId);
    emit("hide");
  }
};

// 方法：处理打开文件所在位置
const handleOpenLocation = () => {
  if (props.selectedFile) {
    emit("openLocation", props.selectedFile);
    emit("hide");
  }
};

// 方法：处理编辑信息
const handleEditInfo = () => {
  if (props.selectedFile) {
    emit("editInfo", props.selectedFile);
    emit("hide");
  }
};
</script>

<style scoped>
/* 右键菜单样式 */
.context-menu {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 1000;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  min-width: 120px;
  padding: 5px 0;
  transition: all 0.2s ease;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  padding: 4px 12px; /* 减小内边距 */
  cursor: pointer;
  transition: background-color 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px; /* 稍微减小字号 */
}

.context-menu-item:hover {
  background-color: #f5f5f5;
}
</style>
