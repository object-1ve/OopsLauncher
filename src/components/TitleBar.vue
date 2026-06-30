<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-content">
      <div class="app-icon">
        <img src="@/assets/icon.png" alt="OopsLauncher" />
      </div>
      <div class="app-title">OopsLauncher</div>
    </div>

    <div class="titlebar-controls">
      <div class="titlebar-button search" @click="toggleSearch">
        <el-icon><Search /></el-icon>
      </div>
      <div class="titlebar-button settings" @click="openSettings">
        <el-icon><Setting /></el-icon>
      </div>

      <div class="titlebar-button maximize" @click="toggleMaximize">
        <el-icon v-if="isMaximized"><CopyDocument /></el-icon>
        <el-icon v-else><FullScreen /></el-icon>
      </div>
      <div class="titlebar-button minimize" @click="minimize">
        <el-icon><Minus /></el-icon>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from "vue";
import { Window } from "@tauri-apps/api/window";
import { Minus, CopyDocument, FullScreen, Setting, Search } from "@element-plus/icons-vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useFiles } from "@/composables/useFiles";
import { isTauri } from "@/utils/env";

const isTauriApp = isTauri();
const appWindow = isTauriApp ? new Window("main") : null;
const isMaximized = ref(false);
const { showSearchOverlay } = useFiles();

const toggleSearch = () => {
  showSearchOverlay.value = !showSearchOverlay.value;
};

const openSettings = async () => {
  if (!isTauriApp) return;
  // 先尝试获取已存在的设置窗口
  const existingWindow = await WebviewWindow.getByLabel("settings");
  if (existingWindow) {
    await existingWindow.setFocus();
    return;
  }

  const settingsWindow = new WebviewWindow("settings", {
    url: "/settings",
    title: "设置",
    width: 650,
    height: 550,
    resizable: false,
    decorations: false, // 改为无边框，我们在页面内部实现自定义关闭逻辑
    center: true,
  });

  settingsWindow.once("tauri://created", function () {
    console.log("Settings window created");
  });

  settingsWindow.once("tauri://error", function (e) {
    console.error("Failed to create settings window:", e);
  });
};

const minimize = async () => {
  if (!appWindow) return;
  await appWindow.minimize();
};

const toggleMaximize = async () => {
  if (!appWindow) return;
  await appWindow.toggleMaximize();
};



onMounted(async () => {
  if (!appWindow) return;
  // 监听窗口大小变化以更新最大化状态图标
  // 注意：Tauri v2 的事件监听可能需要调整，这里先简单实现
  // 实际开发中可以通过监听 tauri://resize 事件或定期检查

  // 简单的状态检查
  isMaximized.value = await appWindow.isMaximized();

  // 监听最大化事件
  // appWindow.listen('tauri://resize', async () => {
  //   isMaximized.value = await appWindow.isMaximized()
  // })
});
</script>

<style scoped>
.titlebar {
  height: var(--app-titlebar-height, 32px);
  user-select: none;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: #fff;
  border-bottom: 1px solid var(--app-border, #e8eaed);
  flex-shrink: 0;
}

.titlebar-content {
  display: flex;
  align-items: center;
  padding-left: 12px;
  pointer-events: none;
  gap: 8px;
}

.app-icon {
  display: flex;
  align-items: center;
}

.app-icon img {
  width: 20px;
  height: 20px;
}

.app-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--app-text-primary, #1a1a2e);
  letter-spacing: 0.02em;
}

.titlebar-controls {
  display: flex;
  height: 100%;
  align-items: center;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 40px;
  height: 100%;
  cursor: pointer;
  transition: background-color var(--app-transition, 0.2s);
  color: var(--app-text-secondary, #606266);
  border: none;
  background: none;
}

.titlebar-button:hover {
  background-color: var(--app-hover, #f5f7fa);
}

.titlebar-button:active {
  background-color: var(--app-border, #e8eaed);
}

.titlebar-button.close:hover {
  background-color: #e81123;
  color: white;
}

.titlebar-button .el-icon {
  font-size: 14px;
}
</style>
