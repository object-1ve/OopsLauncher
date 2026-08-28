<template>
  <div class="settings-section">
    <div class="section-header">
      <h2 class="section-title">常规设置</h2>
      <p class="section-subtitle">配置应用的基础行为与系统集成</p>
    </div>

    <div class="settings-card">
      <div class="card-label">系统</div>
      <div class="setting-items">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">开机自启动</span>
            <span class="setting-hint">登录 Windows 时自动启动 OopsLauncher</span>
          </div>
          <el-switch v-model="settings.general.autoStart" />
        </div>
        <div class="setting-row" :class="{ disabled: !settings.general.autoStart }">
          <div class="setting-info">
            <span class="setting-name">开机静默启动</span>
            <span class="setting-hint">启动后最小化到系统托盘，不显示主窗口</span>
          </div>
          <el-switch v-model="settings.general.autoStartMinimized" :disabled="!settings.general.autoStart" />
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">最小化到系统托盘</span>
            <span class="setting-hint">关闭窗口时最小化到托盘区域而非退出</span>
          </div>
          <el-switch v-model="settings.general.minimizeToTray" />
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">隐藏任务栏图标</span>
            <span class="setting-hint">在任务栏中隐藏应用图标，仅保留托盘图标</span>
          </div>
          <el-switch v-model="settings.general.hideTaskbar" />
        </div>
      </div>
    </div>

    <div class="settings-card">
      <div class="card-label">语言</div>
      <div class="setting-items">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">界面语言</span>
            <span class="setting-hint">更改应用的显示语言</span>
          </div>
          <el-select v-model="settings.general.language" style="width: 160px" size="default">
            <el-option label="简体中文" value="zh-CN" />
            <el-option label="English" value="en-US" />
          </el-select>
        </div>
      </div>
    </div>

    <div class="settings-card">
      <div class="card-label">最近打开</div>
      <div class="setting-items">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">最近打开记录数量</span>
            <span class="setting-hint">搜索面板空查询时显示的最多历史记录条数，设为 0 可关闭</span>
          </div>
          <el-input-number v-model="settings.general.recentCount" :min="0" :max="50" :step="1" size="default" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  settings: { type: Object, required: true }
});
</script>

<style scoped>
.settings-section {
  animation: sectionEnter 0.4s ease both;
}

.section-header {
  margin-bottom: 28px;
}

.section-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--settings-text-primary);
  letter-spacing: -0.01em;
  margin-bottom: 6px;
}

.section-subtitle {
  font-size: 14px;
  color: var(--settings-text-muted);
  font-weight: 400;
}

.settings-card {
  background: var(--settings-card-bg);
  border: 1px solid var(--settings-card-border);
  border-radius: var(--settings-radius-md);
  padding: 20px 24px;
  margin-bottom: 20px;
  box-shadow: var(--settings-shadow-sm);
  transition: box-shadow var(--settings-transition);
}

.settings-card:hover {
  box-shadow: var(--settings-shadow-md);
}

.card-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--settings-text-muted);
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--settings-border);
}

.setting-items {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  transition: opacity var(--settings-transition);
}

.setting-row + .setting-row {
  border-top: 1px solid var(--settings-border);
}

.setting-row.disabled {
  opacity: 0.45;
  pointer-events: none;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--settings-text-primary);
}

.setting-hint {
  font-size: 12px;
  color: var(--settings-text-muted);
  line-height: 1.4;
}

@keyframes sectionEnter {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
