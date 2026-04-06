<template>
  <div class="settings-header" data-tauri-drag-region>
    <div class="window-title">设置</div>
    <div class="window-controls">
      <div class="control-btn close" @click="$emit('close')">
        <el-icon><Close /></el-icon>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Close } from "@element-plus/icons-vue";

defineEmits(["close"]);
</script>

<style scoped>
.settings-header {
  height: 36px;
  min-height: 36px;
  max-height: 36px;
  flex: 0 0 36px;
  background-color: #fff;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0 0 15px;
  border-bottom: 1px solid #f0f0f0;
  user-select: none;
}

.window-title {
  font-size: 12px;
  color: #909399;
  font-weight: 500;
}

.window-controls {
  display: flex;
  height: 100%;
}

.control-btn {
  width: 46px;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: all 0.2s;
  color: #606266;
}

.control-btn.close:hover {
  background-color: #e81123;
  color: #fff;
}
</style>
