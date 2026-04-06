<template>
  <div class="settings-section about-section">
    <div class="app-info">
      <img src="@/assets/icon.png" alt="logo" class="about-logo" />
      <h2 class="app-name">OopsLauncher</h2>
      <p class="app-version">Version {{ appVersion }}</p>
      <p class="app-desc">一个极简、高效的 Tauri 启动器</p>
    </div>
    <div class="about-links">
      <el-link type="primary" href="https://github.com/yourname/OopsLauncher" target="_blank">GitHub 仓库</el-link>
      <el-divider direction="vertical" />
      <el-link type="primary">检查更新</el-link>
    </div>
  </div>
</template>

<script setup>
defineProps({
  appVersion: {
    type: String,
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

.about-section {
  text-align: center;
  padding-top: 20px;
}

.about-logo {
  width: 80px;
  height: 80px;
  margin-bottom: 20px;
}

.app-name {
  margin-bottom: 10px;
  font-size: 24px;
}

.app-version {
  color: #909399;
  margin-bottom: 15px;
}

.app-desc {
  color: #606266;
  margin-bottom: 30px;
}

.about-links {
  margin-top: 40px;
}
</style>
