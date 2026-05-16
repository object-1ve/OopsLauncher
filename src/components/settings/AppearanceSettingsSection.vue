<template>
  <div class="settings-section">
    <div class="section-header">
      <h2 class="section-title">外观设置</h2>
      <p class="section-subtitle">自定义应用的外观与主题</p>
    </div>

    <!-- Theme Presets -->
    <div class="settings-card">
      <div class="card-label">主题预设</div>
      <div class="theme-presets">
        <div
          class="theme-preset-card"
          :class="{ active: settings.appearance.theme === 'light' }"
          @click="settings.appearance.theme = 'light'"
        >
          <div class="preset-visual light">
            <div class="preset-toolbar"></div>
            <div class="preset-body">
              <div class="preset-sidebar"></div>
              <div class="preset-content">
                <div class="preset-dot"></div>
                <div class="preset-line"></div>
                <div class="preset-line short"></div>
              </div>
            </div>
          </div>
          <div class="preset-label">
            <span class="preset-name">浅色模式</span>
            <svg v-if="settings.appearance.theme === 'light'" class="check-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </div>
        </div>
        <div
          class="theme-preset-card"
          :class="{ active: settings.appearance.theme === 'dark' }"
          @click="settings.appearance.theme = 'dark'"
        >
          <div class="preset-visual dark">
            <div class="preset-toolbar"></div>
            <div class="preset-body">
              <div class="preset-sidebar"></div>
              <div class="preset-content">
                <div class="preset-dot"></div>
                <div class="preset-line"></div>
                <div class="preset-line short"></div>
              </div>
            </div>
          </div>
          <div class="preset-label">
            <span class="preset-name">深色模式</span>
            <svg v-if="settings.appearance.theme === 'dark'" class="check-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Settings -->
    <div class="settings-card">
      <div class="card-label">快速调整</div>
      <div class="setting-items">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">图标大小</span>
            <span class="setting-hint">{{ settings.appearance.iconSize }}px</span>
          </div>
          <div class="slider-wrap" style="width: 200px;">
            <el-slider v-model="settings.appearance.iconSize" :min="32" :max="128" :step="8" />
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">布局模式</span>
            <span class="setting-hint">{{ settings.appearance.itemLayout === 'tile' ? '网格排列' : '列表排列' }}</span>
          </div>
          <el-radio-group v-model="settings.appearance.itemLayout" size="small">
            <el-radio-button value="tile">网格</el-radio-button>
            <el-radio-button value="list">列表</el-radio-button>
          </el-radio-group>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">窗口透明度</span>
            <span class="setting-hint">{{ (settings.appearance.transparency * 100).toFixed(0) }}%</span>
          </div>
          <div class="slider-wrap" style="width: 200px;">
            <el-slider v-model="settings.appearance.transparency" :min="0.1" :max="1.0" :step="0.05" />
          </div>
        </div>
      </div>
    </div>

    <!-- Theme Customization -->
    <div class="settings-card">
      <div class="card-label">主题定制</div>
      <div class="setting-items">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">深浅模式</span>
            <span class="setting-hint">在浅色与深色之间平滑过渡</span>
          </div>
          <div class="custom-slider-container mode-slider-wrap">
            <div class="mode-gradient"></div>
            <el-slider v-model="themeModeModel" :min="0" :max="100" :show-tooltip="false" />
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">主题色相</span>
            <span class="setting-hint">调整主色调的色相值</span>
          </div>
          <div class="custom-slider-container hue-slider-wrap">
            <div class="hue-gradient"></div>
            <el-slider v-model="themeHueModel" :min="0" :max="360" :show-tooltip="false" />
          </div>
        </div>
      </div>
    </div>

    <!-- CSS Config -->
    <div class="settings-card">
      <div class="card-label">CSS 样式配置</div>
      <el-collapse v-model="activeCollapseNamesModel">
        <el-collapse-item title="颜色配置" name="colors">
          <div class="color-grid">
            <div class="color-item" v-for="c in colorFields" :key="c.key">
              <span class="color-label">{{ c.label }}</span>
              <el-color-picker v-model="settings.appearance.css[c.key]" show-alpha size="small" />
            </div>
          </div>
        </el-collapse-item>
        <el-collapse-item title="尺寸配置" name="sizes">
          <div class="size-list">
            <div class="size-row" v-for="s in sizeFields" :key="s.key">
              <span class="size-label">{{ s.label }}</span>
              <div class="size-control">
                <el-slider
                  :model-value="s.parse(settings.appearance.css[s.key])"
                  @update:model-value="(val) => (settings.appearance.css[s.key] = s.format(val))"
                  :min="s.min"
                  :max="s.max"
                  :step="s.step"
                  :show-input="s.showInput"
                />
              </div>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>
  </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  settings: { type: Object, required: true },
  themeHue: { type: Number, required: true },
  themeMode: { type: Number, required: true },
  activeCollapseNames: { type: Array, required: true }
});

const emit = defineEmits(["update:themeHue", "update:themeMode", "update:activeCollapseNames"]);

const themeHueModel = computed({
  get: () => props.themeHue,
  set: (v) => emit("update:themeHue", v)
});

const themeModeModel = computed({
  get: () => props.themeMode,
  set: (v) => emit("update:themeMode", v)
});

const activeCollapseNamesModel = computed({
  get: () => props.activeCollapseNames,
  set: (v) => emit("update:activeCollapseNames", v)
});

const colorFields = [
  { key: 'primaryColor', label: '主色调' },
  { key: 'secondaryColor', label: '次要色' },
  { key: 'backgroundColor', label: '背景色' },
  { key: 'textColor', label: '文本色' },
  { key: 'borderColor', label: '边框色' },
  { key: 'hoverColor', label: '悬停色' },
];

const sizeFields = [
  { key: 'borderRadius', label: '边框圆角', min: 0, max: 12, step: 1, showInput: true, parse: (v) => parseInt(v), format: (v) => v + 'px' },
  { key: 'itemMargin', label: '项目间距', min: 0, max: 16, step: 2, showInput: true, parse: (v) => parseInt(v), format: (v) => v + 'px' },
  { key: 'itemPadding', label: '项目内边距', min: 0, max: 16, step: 2, showInput: true, parse: (v) => parseInt(v), format: (v) => v + 'px' },
  { key: 'fontSize', label: '字体大小', min: 10, max: 18, step: 1, showInput: true, parse: (v) => parseInt(v), format: (v) => v + 'px' },
  { key: 'lineHeight', label: '行高', min: 1, max: 2, step: 0.1, showInput: true, parse: (v) => parseFloat(v), format: (v) => v.toString() },
];
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

/* ── Theme Presets ── */
.theme-presets {
  display: flex;
  gap: 16px;
}

.theme-preset-card {
  flex: 1;
  cursor: pointer;
  border-radius: var(--settings-radius-sm);
  border: 2px solid var(--settings-border);
  overflow: hidden;
  transition: all var(--settings-transition);
  background: var(--settings-card-bg);
}

.theme-preset-card:hover {
  border-color: #c0c4cc;
  transform: translateY(-2px);
  box-shadow: var(--settings-shadow-md);
}

.theme-preset-card.active {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.15);
}

.preset-visual {
  padding: 16px;
}

.preset-visual.light {
  background: #f8f9fc;
}

.preset-visual.dark {
  background: #1e1e2e;
}

.preset-toolbar {
  height: 8px;
  border-radius: 4px;
  margin-bottom: 8px;
}

.preset-visual.light .preset-toolbar {
  background: #e2e4ea;
}

.preset-visual.dark .preset-toolbar {
  background: #2d2d44;
}

.preset-body {
  display: flex;
  gap: 6px;
  height: 48px;
}

.preset-sidebar {
  width: 24px;
  border-radius: 4px;
}

.preset-visual.light .preset-sidebar {
  background: #e2e4ea;
}

.preset-visual.dark .preset-sidebar {
  background: #2d2d44;
}

.preset-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  justify-content: center;
}

.preset-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--el-color-primary);
  margin-bottom: 2px;
}

.preset-line {
  height: 4px;
  border-radius: 2px;
  background: var(--settings-border);
  width: 100%;
}

.preset-visual.dark .preset-line {
  background: #3d3d56;
}

.preset-line.short {
  width: 60%;
}

.preset-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-top: 1px solid var(--settings-border);
}

.preset-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--settings-text-primary);
}

.check-icon {
  color: var(--el-color-primary);
  flex-shrink: 0;
}

/* ── Setting Rows ── */
.setting-items {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
  gap: 16px;
}

.setting-row + .setting-row {
  border-top: 1px solid var(--settings-border);
}

.setting-info {
  flex-shrink: 0;
}

.setting-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--settings-text-primary);
  display: block;
  margin-bottom: 2px;
}

.setting-hint {
  font-size: 12px;
  color: var(--settings-text-muted);
}

.slider-wrap {
  flex-shrink: 0;
}

/* ── Custom Sliders (Hue & Mode) ── */
.custom-slider-container {
  position: relative;
  flex: 1;
  max-width: 220px;
}

.custom-slider-container :deep(.el-slider__runway) {
  background-color: transparent !important;
}

.custom-slider-container :deep(.el-slider__bar) {
  background-color: transparent !important;
}

.custom-slider-container :deep(.el-slider__button) {
  border-color: #fff;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  width: 18px;
  height: 18px;
}

.hue-gradient {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 6px;
  margin-top: -3px;
  border-radius: 3px;
  background: linear-gradient(to right, hsl(0, 100%, 50%), hsl(60, 100%, 50%), hsl(120, 100%, 50%), hsl(180, 100%, 50%), hsl(240, 100%, 50%), hsl(300, 100%, 50%), hsl(360, 100%, 50%));
  z-index: 0;
  pointer-events: none;
}

.mode-gradient {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 6px;
  margin-top: -3px;
  border-radius: 3px;
  background: linear-gradient(to right, #f5f7fa, #2c2c2c);
  z-index: 0;
  pointer-events: none;
}

/* ── Color Grid ── */
.color-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 12px;
}

.color-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.color-label {
  font-size: 12px;
  color: var(--settings-text-secondary);
  font-weight: 500;
}

/* ── Size Config ── */
.size-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.size-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.size-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--settings-text-secondary);
  min-width: 80px;
  flex-shrink: 0;
}

.size-control {
  flex: 1;
}

@keyframes sectionEnter {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
