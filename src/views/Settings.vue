<template>
  <div class="settings-view">
    <!-- 添加拖拽区域和关闭按钮 -->
    <div class="settings-header" data-tauri-drag-region>
      <div class="window-title">设置</div>
      <div class="window-controls">
        <div class="control-btn close" @click="closeWindow">
          <el-icon><Close /></el-icon>
        </div>
      </div>
    </div>

    <el-container class="settings-container">
      <el-aside width="160px" class="settings-aside">
        <div class="aside-header">
          <h3>设置</h3>
        </div>
        <el-menu :default-active="activeTab" class="settings-menu" @select="handleSelect">
          <el-menu-item index="general">
            <el-icon><Setting /></el-icon>
            <span>常规</span>
          </el-menu-item>
          <el-menu-item index="appearance">
            <el-icon><Brush /></el-icon>
            <span>外观</span>
          </el-menu-item>
          <el-menu-item index="shortcuts">
            <el-icon><Operation /></el-icon>
            <span>快捷键</span>
          </el-menu-item>
          <el-menu-item index="about">
            <el-icon><InfoFilled /></el-icon>
            <span>关于</span>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <el-main class="settings-main">
        <div class="content-wrapper">
          <div v-if="activeTab === 'general'" class="settings-section">
            <h2 class="section-title">常规设置</h2>
            <el-form label-position="top">
              <el-form-item label="系统">
                <el-checkbox v-model="settings.general.autoStart">开机自启动</el-checkbox>
                <el-checkbox v-model="settings.general.minimizeToTray"
                  >最小化到系统托盘</el-checkbox
                >
              </el-form-item>
              <el-form-item label="语言">
                <el-select v-model="settings.general.language" style="width: 200px">
                  <el-option label="简体中文" value="zh-CN" />
                  <el-option label="English" value="en-US" />
                </el-select>
              </el-form-item>
            </el-form>
          </div>

          <div v-if="activeTab === 'appearance'" class="settings-section">
            <h2 class="section-title">外观设置</h2>
            <el-form label-position="top">
              <el-form-item label="主题预设">
                <div class="theme-presets">
                  <div
                    class="theme-preset light"
                    :class="{ active: settings.appearance.theme === 'light' }"
                    @click="settings.appearance.theme = 'light'"
                  >
                    <div class="preset-preview"></div>
                    <span>浅色</span>
                  </div>
                  <div
                    class="theme-preset dark"
                    :class="{ active: settings.appearance.theme === 'dark' }"
                    @click="settings.appearance.theme = 'dark'"
                  >
                    <div class="preset-preview"></div>
                    <span>深色</span>
                  </div>
                </div>
              </el-form-item>
              <el-form-item label="图标大小">
                <el-slider
                  v-model="settings.appearance.iconSize"
                  :min="32"
                  :max="128"
                  :step="8"
                  show-input
                />
              </el-form-item>
              <el-form-item label="布局模式">
                <el-radio-group v-model="settings.appearance.itemLayout">
                  <el-radio-button value="tile">网格</el-radio-button>
                  <el-radio-button value="list">列表</el-radio-button>
                </el-radio-group>
              </el-form-item>
              <el-form-item label="窗口透明度">
                <el-slider
                  v-model="settings.appearance.transparency"
                  :min="0.1"
                  :max="1.0"
                  :step="0.05"
                />
              </el-form-item>
              <el-form-item label="CSS 样式配置">
                <el-collapse v-model="activeCollapseNames">
                  <el-collapse-item title="颜色配置" name="colors">
                    <el-form label-position="top" size="small">
                      <el-form-item label="主色调">
                        <el-color-picker
                          v-model="settings.appearance.css.primaryColor"
                          show-alpha
                        />
                      </el-form-item>
                      <el-form-item label="次要色调">
                        <el-color-picker
                          v-model="settings.appearance.css.secondaryColor"
                          show-alpha
                        />
                      </el-form-item>
                      <el-form-item label="背景颜色">
                        <el-color-picker
                          v-model="settings.appearance.css.backgroundColor"
                          show-alpha
                        />
                      </el-form-item>
                      <el-form-item label="文本颜色">
                        <el-color-picker
                          v-model="settings.appearance.css.textColor"
                          show-alpha
                        />
                      </el-form-item>
                      <el-form-item label="边框颜色">
                        <el-color-picker
                          v-model="settings.appearance.css.borderColor"
                          show-alpha
                        />
                      </el-form-item>
                      <el-form-item label="悬停颜色">
                        <el-color-picker
                          v-model="settings.appearance.css.hoverColor"
                          show-alpha
                        />
                      </el-form-item>
                    </el-form>
                  </el-collapse-item>
                  <el-collapse-item title="尺寸配置" name="sizes">
                    <el-form label-position="top" size="small">
                      <el-form-item label="边框圆角">
                        <el-slider
                          :model-value="parseInt(settings.appearance.css.borderRadius)"
                          @update:model-value="
                            (val) => (settings.appearance.css.borderRadius = val + 'px')
                          "
                          :min="0"
                          :max="12"
                          :step="1"
                          show-input
                        />
                      </el-form-item>
                      <el-form-item label="项目间距">
                        <el-slider
                          :model-value="parseInt(settings.appearance.css.itemMargin)"
                          @update:model-value="
                            (val) => (settings.appearance.css.itemMargin = val + 'px')
                          "
                          :min="0"
                          :max="16"
                          :step="2"
                          show-input
                        />
                      </el-form-item>
                      <el-form-item label="项目内边距">
                        <el-slider
                          :model-value="parseInt(settings.appearance.css.itemPadding)"
                          @update:model-value="
                            (val) => (settings.appearance.css.itemPadding = val + 'px')
                          "
                          :min="0"
                          :max="16"
                          :step="2"
                          show-input
                        />
                      </el-form-item>
                      <el-form-item label="字体大小">
                        <el-slider
                          :model-value="parseInt(settings.appearance.css.fontSize)"
                          @update:model-value="
                            (val) => (settings.appearance.css.fontSize = val + 'px')
                          "
                          :min="10"
                          :max="18"
                          :step="1"
                          show-input
                        />
                      </el-form-item>
                      <el-form-item label="行高">
                        <el-slider
                          :model-value="parseFloat(settings.appearance.css.lineHeight)"
                          @update:model-value="
                            (val) => (settings.appearance.css.lineHeight = val.toString())
                          "
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

          <div v-if="activeTab === 'shortcuts'" class="settings-section">
            <h2 class="section-title">快捷键设置</h2>
            <el-form label-position="top">
              <el-form-item label="显示/隐藏主窗口">
                <el-input
                  v-model="settings.shortcuts.showHide"
                  placeholder="点击设置快捷键"
                  readonly
                  @keydown.prevent="handleShortcutKeyDown($event, 'showHide')"
                />
                <div class="shortcut-tip">按下想要设置的组合键</div>
              </el-form-item>
              <el-form-item label="复制当前时间到剪切板">
                <el-input
                  v-model="settings.shortcuts.copyTime"
                  placeholder="点击设置快捷键"
                  readonly
                  @keydown.prevent="handleShortcutKeyDown($event, 'copyTime')"
                />
                <div class="shortcut-tip">按下想要设置的组合键</div>
              </el-form-item>
            </el-form>
          </div>

          <div v-if="activeTab === 'about'" class="settings-section about-section">
            <div class="app-info">
              <img src="@/assets/icon.png" alt="logo" class="about-logo" />
              <h2 class="app-name">OopsLauncher</h2>
              <p class="app-version">Version {{ appVersion }}</p>
              <p class="app-desc">一个极简、高效的 Tauri 启动器</p>
            </div>
            <div class="about-links">
              <el-link
                type="primary"
                href="https://github.com/yourname/OopsLauncher"
                target="_blank"
                >GitHub 仓库</el-link
              >
              <el-divider direction="vertical" />
              <el-link type="primary">检查更新</el-link>
            </div>
          </div>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup>
import { ref, watch, onMounted, onUnmounted } from "vue";
import { Setting, Brush, Operation, InfoFilled, Close } from "@element-plus/icons-vue";
import { useSettings } from "@/composables/useSettings";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";

const { settings } = useSettings();
const activeTab = ref("general");
const activeCollapseNames = ref([]);
const appWindow = getCurrentWebviewWindow();
const appVersion = ref("");

const closeWindow = async () => {
  await appWindow.close();
};

// 获取应用版本号
const fetchAppVersion = async () => {
  try {
    appVersion.value = await invoke("get_app_version");
  } catch (err) {
    console.error("Failed to fetch app version:", err);
    appVersion.value = "Unknown";
  }
};

// 快捷键关闭 (ESC)
const handleGlobalKeyDown = (e) => {
  if (e.key === "Escape") {
    closeWindow();
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKeyDown);
  fetchAppVersion();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

// 开机启动
const updateAutoStart = async (val) => {
  if (val) {
    await enable();
  } else {
    await disable();
  }
};

// 监听开机启动设置
watch(
  () => settings.value.general.autoStart,
  (val) => {
    updateAutoStart(val);
  }
);

const handleSelect = (index) => {
  activeTab.value = index;
};

// 防抖函数
const debounce = (func, wait) => {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
};

// 防抖处理的快捷键设置
const debouncedHandleShortcutKeyDown = debounce((e, keyType) => {
  const keys = [];
  if (e.ctrlKey) keys.push("Ctrl");
  if (e.altKey) keys.push("Alt");
  if (e.shiftKey) keys.push("Shift");
  if (e.metaKey) keys.push("Meta");

  const key = e.key === " " ? "Space" : e.key;
  if (!["Control", "Alt", "Shift", "Meta"].includes(key)) {
    keys.push(key.toUpperCase());
    settings.value.shortcuts[keyType] = keys.join("+");
  }
}, 1000);

const handleShortcutKeyDown = (e, keyType) => {
  debouncedHandleShortcutKeyDown(e, keyType);
};
</script>

<style scoped>
.settings-view {
  height: 100vh;
  background-color: #fff;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.settings-header {
  height: 36px;
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

.control-btn:hover {
  background-color: #f5f5f5;
}

.control-btn.close:hover {
  background-color: #e81123;
  color: #fff;
}

.settings-container {
  flex: 1;
}

.settings-aside {
  background-color: #fff;
  border-right: 1px solid #e6e6e6;
  display: flex;
  flex-direction: column;
}

.aside-header {
  padding: 20px;
  border-bottom: 1px solid #f0f0f0;
}

.aside-header h3 {
  margin: 0;
  font-size: 18px;
  color: #303133;
}

.settings-menu {
  border-right: none;
  flex: 1;
}

.settings-main {
  padding: 0;
  background-color: #fff;
}

.content-wrapper {
  padding: 40px;
  max-width: 600px;
  margin: 0 auto;
}

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

.shortcut-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
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

:deep(.el-form-item__label) {
  font-weight: 600;
  padding-bottom: 8px;
}
</style>
