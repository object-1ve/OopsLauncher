<template>
  <div class="settings-section">
    <h2 class="section-title">常规设置</h2>
    <el-form label-position="top">
      <el-form-item label="系统">
        <div class="system-options">
          <el-checkbox v-model="settings.general.autoStart">开机自启动</el-checkbox>
          <el-checkbox v-model="settings.general.autoStartMinimized" :disabled="!settings.general.autoStart">
            开机静默启动 (仅托盘)
          </el-checkbox>
          <el-checkbox v-model="settings.general.minimizeToTray">最小化到系统托盘</el-checkbox>
          <el-checkbox v-model="settings.general.hideTaskbar">隐藏任务栏图标</el-checkbox>
        </div>
      </el-form-item>
      <el-form-item label="语言">
        <el-select v-model="settings.general.language" style="width: 200px">
          <el-option label="简体中文" value="zh-CN" />
          <el-option label="English" value="en-US" />
        </el-select>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup>
defineProps({
  settings: {
    type: Object,
    required: true
  }
});
</script>

<style scoped>
.settings-section {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.section-title {
  margin-bottom: 30px;
  font-size: 22px;
  font-weight: 600;
  color: #303133;
}

.system-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.el-form-item__label) {
  font-weight: 600;
  padding-bottom: 8px;
}
</style>
