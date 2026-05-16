<template>
  <div class="settings-header" data-tauri-drag-region>
    <div class="header-left">
      <div class="header-icon">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
        </svg>
      </div>
      <span class="window-title">设置</span>
    </div>
    <div class="window-controls">
      <div class="control-btn close" @click="$emit('close')" title="关闭">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup>
defineEmits(["close"]);
</script>

<style scoped>
.settings-header {
  height: 40px;
  min-height: 40px;
  flex: 0 0 40px;
  background: var(--settings-sidebar-bg);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0 0 16px;
  border-bottom: 1px solid var(--settings-border);
  user-select: none;
  -webkit-app-region: drag;
  position: relative;
  z-index: 10;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  -webkit-app-region: drag;
}

.header-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--el-color-primary);
  opacity: 0.8;
}

.window-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--settings-text-primary);
  letter-spacing: 0.03em;
}

.window-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.control-btn {
  width: 46px;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--settings-text-muted);
}

.control-btn:hover {
  background-color: rgba(0, 0, 0, 0.04);
  color: var(--settings-text-primary);
}

.control-btn.close:hover {
  background-color: #e81123;
  color: #fff;
}
</style>
