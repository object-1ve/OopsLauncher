<template>
  <div class="settings-section">
    <div class="section-header">
      <h2 class="section-title">快捷键设置</h2>
      <p class="section-subtitle">点击快捷键框后按下键盘组合键进行设置</p>
    </div>

    <div class="settings-card">
      <div class="card-label">全局快捷键</div>
      <div class="shortcut-list">
        <div
          v-for="def in shortcutDefs"
          :key="def.key"
          class="shortcut-row"
          :class="{ 'is-recording': recordingKey === def.key }"
        >
          <div class="shortcut-info">
            <span class="shortcut-name">{{ def.label }}</span>
            <span class="shortcut-desc" v-if="def.description && recordingKey !== def.key">{{ def.description }}</span>
            <span class="shortcut-desc recording-hint" v-if="recordingKey === def.key">
              <span class="rec-dot"></span>
              录制中，按 <kbd>Esc</kbd> 取消
            </span>
          </div>
          <div class="shortcut-action">
            <div
              class="shortcut-viewer"
              :class="{ 'recording-active': recordingKey === def.key }"
              @click="startRecording(def.key)"
            >
              <div class="keys-container" v-if="getDisplayValue(def.key)">
                <span v-for="(key, index) in getDisplayValue(def.key).split('+')" :key="index" class="key-cap">{{ key }}</span>
              </div>
              <span class="placeholder" v-else>{{ recordingKey === def.key ? '输入组合键...' : '点击录制' }}</span>
            </div>
            <el-button
              v-if="settings.shortcuts[def.key] && recordingKey !== def.key"
              class="clear-btn"
              size="small"
              circle
              @click.stop="clearShortcut(def.key)"
            >
              <el-icon><Close /></el-icon>
            </el-button>
          </div>
          <div class="shortcut-conflict" v-if="recordingKey !== def.key && conflictFor(def.key)">
            <el-icon><WarningFilled /></el-icon>
            与「{{ conflictFor(def.key) }}」冲突
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { Close, WarningFilled } from '@element-plus/icons-vue'

const props = defineProps({
  settings: { type: Object, required: true }
})

const shortcutDefs = [
  { key: 'showHide', label: '显示/隐藏主窗口', description: '全局快捷键，随时唤醒应用' },
  { key: 'copyTime', label: '复制当前时间', description: '复制当前时间戳到剪贴板' },
  { key: 'notificationIcon', label: '取色器（预留）', description: '预留功能，暂未实现' },
]

const recordingKey = ref(null)
const pendingKeys = ref('')

const getDisplayValue = (key) => {
  if (recordingKey.value === key && pendingKeys.value) return pendingKeys.value
  return props.settings.shortcuts[key] || ''
}

const formatKeys = (event) => {
  const parts = []
  if (event.ctrlKey) parts.push('Ctrl')
  if (event.altKey) parts.push('Alt')
  if (event.shiftKey) parts.push('Shift')
  if (event.metaKey) parts.push('Meta')
  const key = event.key === ' ' ? 'Space' : event.key
  if (!['Control', 'Alt', 'Shift', 'Meta'].includes(key)) parts.push(key.toUpperCase())
  return parts
}

const startRecording = (key) => { recordingKey.value = key; pendingKeys.value = '' }
const stopRecording = () => { recordingKey.value = null; pendingKeys.value = '' }
const clearShortcut = (key) => { props.settings.shortcuts[key] = '' }

const conflictFor = (key) => {
  const value = props.settings.shortcuts[key]
  if (!value) return null
  for (const def of shortcutDefs) {
    if (def.key !== key && props.settings.shortcuts[def.key] === value) return def.label
  }
  return null
}

const onKeyDown = (event) => {
  if (!recordingKey.value) return

  // 录制期间阻止所有系统默认行为（如 Alt+Space 系统菜单）
  event.preventDefault()
  event.stopPropagation()

  if (event.key === 'Escape') { stopRecording(); return }
  if (event.key === 'Backspace') { props.settings.shortcuts[recordingKey.value] = ''; stopRecording(); return }
  const combo = formatKeys(event)
  pendingKeys.value = combo.join('+')
  const key = event.key === ' ' ? 'Space' : event.key
  if (!['Control', 'Alt', 'Shift', 'Meta'].includes(key) && combo.length > 0) {
    props.settings.shortcuts[recordingKey.value] = combo.join('+')
    stopRecording()
  }
}

onMounted(() => window.addEventListener('keydown', onKeyDown, true))
onUnmounted(() => window.removeEventListener('keydown', onKeyDown, true))
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
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--settings-border);
}

.shortcut-list {
  display: flex;
  flex-direction: column;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  padding: 14px 0;
  gap: 12px;
}

.shortcut-row + .shortcut-row {
  border-top: 1px solid var(--settings-border);
}

.shortcut-info {
  flex: 1;
  min-width: 180px;
}

.shortcut-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--settings-text-primary);
  display: block;
  margin-bottom: 2px;
}

.shortcut-desc {
  font-size: 12px;
  color: var(--settings-text-muted);
}

.shortcut-desc kbd {
  display: inline-block;
  padding: 1px 5px;
  font-size: 10px;
  font-family: inherit;
  background: #f4f4f5;
  border: 1px solid #d9d9d9;
  border-radius: 3px;
  color: #606266;
}

.recording-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--el-color-danger);
}

.rec-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--el-color-danger);
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.3; transform: scale(0.8); }
}

.shortcut-action {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.shortcut-viewer {
  min-width: 160px;
  min-height: 36px;
  padding: 4px 12px;
  background: #f8f9fc;
  border: 1px solid var(--settings-border);
  border-radius: var(--settings-radius-sm);
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  gap: 8px;
}

.shortcut-viewer:hover {
  border-color: #c0c4cc;
  background: #f0f1f5;
}

.shortcut-viewer.recording-active {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9, #ecf5ff);
  box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.15);
}

.keys-container {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.key-cap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 26px;
  height: 26px;
  padding: 0 7px;
  background: #fff;
  border: 1px solid #dcdfe6;
  border-bottom-width: 3px;
  border-radius: 5px;
  font-size: 12px;
  font-weight: 600;
  color: var(--settings-text-primary);
  text-transform: uppercase;
  user-select: none;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.recording-active .key-cap {
  background: var(--el-color-primary);
  border-color: var(--el-color-primary);
  color: #fff;
  border-bottom-color: var(--el-color-primary-dark-2, #337ecc);
}

.placeholder {
  font-size: 13px;
  color: var(--settings-text-muted);
  user-select: none;
}

.recording-active .placeholder {
  color: var(--el-color-primary);
  font-weight: 500;
}

.clear-btn {
  transition: all 0.2s;
  flex-shrink: 0;
}

.clear-btn:hover {
  transform: rotate(90deg);
  color: var(--el-color-danger);
}

.shortcut-conflict {
  width: 100%;
  font-size: 12px;
  color: #e6a23c;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: rgba(230, 162, 60, 0.08);
  border-radius: 6px;
  margin-top: 4px;
}

.is-recording .shortcut-info {
  color: var(--el-color-primary);
}

@keyframes sectionEnter {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
