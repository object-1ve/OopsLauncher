<template>
  <div class="settings-section">
    <h2 class="section-title">外观设置</h2>
    <el-form label-position="top">
      <el-form-item label="主题预设">
        <div class="theme-presets">
          <div class="theme-preset light" :class="{ active: settings.appearance.theme === 'light' }" @click="settings.appearance.theme = 'light'">
            <div class="preset-preview"></div>
            <span>浅色</span>
          </div>
          <div class="theme-preset dark" :class="{ active: settings.appearance.theme === 'dark' }" @click="settings.appearance.theme = 'dark'">
            <div class="preset-preview"></div>
            <span>深色</span>
          </div>
        </div>
      </el-form-item>
      <el-form-item label="图标大小">
        <el-slider v-model="settings.appearance.iconSize" :min="32" :max="128" :step="8" show-input />
      </el-form-item>
      <el-form-item label="布局模式">
        <el-radio-group v-model="settings.appearance.itemLayout">
          <el-radio-button value="tile">网格</el-radio-button>
          <el-radio-button value="list">列表</el-radio-button>
        </el-radio-group>
      </el-form-item>
      <el-form-item label="深浅模式">
        <div class="mode-slider-container">
          <div class="mode-gradient"></div>
          <el-slider v-model="themeModeModel" :min="0" :max="100" :show-tooltip="false" />
        </div>
      </el-form-item>
      <el-form-item label="主题色相">
        <div class="hue-slider-container">
          <div class="hue-gradient"></div>
          <el-slider v-model="themeHueModel" :min="0" :max="360" :show-tooltip="false" />
        </div>
      </el-form-item>
      <el-form-item label="窗口透明度">
        <el-slider v-model="settings.appearance.transparency" :min="0.1" :max="1.0" :step="0.05" />
      </el-form-item>
      <el-form-item label="CSS 样式配置">
        <el-collapse v-model="activeCollapseNamesModel">
          <el-collapse-item title="颜色配置" name="colors">
            <el-form label-position="top" size="small">
              <el-form-item label="主色调">
                <el-color-picker v-model="settings.appearance.css.primaryColor" show-alpha />
              </el-form-item>
              <el-form-item label="次要色调">
                <el-color-picker v-model="settings.appearance.css.secondaryColor" show-alpha />
              </el-form-item>
              <el-form-item label="背景颜色">
                <el-color-picker v-model="settings.appearance.css.backgroundColor" show-alpha />
              </el-form-item>
              <el-form-item label="文本颜色">
                <el-color-picker v-model="settings.appearance.css.textColor" show-alpha />
              </el-form-item>
              <el-form-item label="边框颜色">
                <el-color-picker v-model="settings.appearance.css.borderColor" show-alpha />
              </el-form-item>
              <el-form-item label="悬停颜色">
                <el-color-picker v-model="settings.appearance.css.hoverColor" show-alpha />
              </el-form-item>
            </el-form>
          </el-collapse-item>
          <el-collapse-item title="尺寸配置" name="sizes">
            <el-form label-position="top" size="small">
              <el-form-item label="边框圆角">
                <el-slider
                  :model-value="parseInt(settings.appearance.css.borderRadius)"
                  @update:model-value="(val) => (settings.appearance.css.borderRadius = val + 'px')"
                  :min="0"
                  :max="12"
                  :step="1"
                  show-input
                />
              </el-form-item>
              <el-form-item label="项目间距">
                <el-slider
                  :model-value="parseInt(settings.appearance.css.itemMargin)"
                  @update:model-value="(val) => (settings.appearance.css.itemMargin = val + 'px')"
                  :min="0"
                  :max="16"
                  :step="2"
                  show-input
                />
              </el-form-item>
              <el-form-item label="项目内边距">
                <el-slider
                  :model-value="parseInt(settings.appearance.css.itemPadding)"
                  @update:model-value="(val) => (settings.appearance.css.itemPadding = val + 'px')"
                  :min="0"
                  :max="16"
                  :step="2"
                  show-input
                />
              </el-form-item>
              <el-form-item label="字体大小">
                <el-slider
                  :model-value="parseInt(settings.appearance.css.fontSize)"
                  @update:model-value="(val) => (settings.appearance.css.fontSize = val + 'px')"
                  :min="10"
                  :max="18"
                  :step="1"
                  show-input
                />
              </el-form-item>
              <el-form-item label="行高">
                <el-slider
                  :model-value="parseFloat(settings.appearance.css.lineHeight)"
                  @update:model-value="(val) => (settings.appearance.css.lineHeight = val.toString())"
                  :min="1"
                  :max="2"
                  :step="0.1"
                  show-input
                />
              </el-form-item>
            </el-form>
          </el-collapse-item>
        </el-collapse>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  settings: {
    type: Object,
    required: true
  },
  themeHue: {
    type: Number,
    required: true
  },
  themeMode: {
    type: Number,
    required: true
  },
  activeCollapseNames: {
    type: Array,
    required: true
  }
});

const emit = defineEmits(["update:themeHue", "update:themeMode", "update:activeCollapseNames"]);

const themeHueModel = computed({
  get: () => props.themeHue,
  set: (value) => emit("update:themeHue", value)
});

const themeModeModel = computed({
  get: () => props.themeMode,
  set: (value) => emit("update:themeMode", value)
});

const activeCollapseNamesModel = computed({
  get: () => props.activeCollapseNames,
  set: (value) => emit("update:activeCollapseNames", value)
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

.theme-presets {
  display: flex;
  gap: 20px;
}

.theme-preset {
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.preset-preview {
  width: 80px;
  height: 50px;
  border-radius: 6px;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.theme-preset.light .preset-preview {
  background-color: #f5f7fa;
  border-color: #e6e6e6;
}

.theme-preset.dark .preset-preview {
  background-color: #2c2c2c;
  border-color: #1a1a1a;
}

.theme-preset.active .preset-preview {
  border-color: #409eff;
  box-shadow: 0 0 8px rgba(64, 158, 255, 0.3);
}

.theme-preset span {
  font-size: 13px;
  color: #606266;
}

.theme-preset.active span {
  color: #409eff;
  font-weight: 600;
}

.hue-slider-container {
  width: 100%;
  position: relative;
  padding: 0 10px;
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

.mode-slider-container {
  width: 100%;
  position: relative;
  padding: 0 10px;
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

:deep(.mode-slider-container .el-slider__runway),
:deep(.hue-slider-container .el-slider__runway) {
  background-color: transparent !important;
}

:deep(.mode-slider-container .el-slider__bar),
:deep(.hue-slider-container .el-slider__bar) {
  background-color: transparent !important;
}

:deep(.mode-slider-container .el-slider__button),
:deep(.hue-slider-container .el-slider__button) {
  border-color: #fff;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

:deep(.el-form-item__label) {
  font-weight: 600;
  padding-bottom: 8px;
}
</style>
