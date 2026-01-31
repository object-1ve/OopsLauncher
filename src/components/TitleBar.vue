<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-content">
      <div class="app-icon">
        <!-- 这里可以放置应用图标 -->
        <span class="icon-emoji"
          ><img src="@/assets/icon.png" alt="图标" style="width: 24px; height: 24px"
        /></span>
      </div>
      <div class="app-title">OopsLauncher</div>
    </div>

    <div class="titlebar-controls">
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
import { ref, onMounted } from "vue";
import { Window } from "@tauri-apps/api/window";
import { Minus, CopyDocument, FullScreen, Close, Setting } from "@element-plus/icons-vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const appWindow = new Window("main");
const isMaximized = ref(false);

const openSettings = async () => {
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
  await appWindow.minimize();
};

const toggleMaximize = async () => {
  await appWindow.toggleMaximize();
};



onMounted(async () => {
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
  height: 32px;
  user-select: none;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: #fff;

}

.titlebar-content {
  display: flex;
  align-items: center;
  padding-left: 10px;
  pointer-events: none; /* 让点击穿透到 drag-region */
}

.app-icon {
  margin-right: 8px;
  display: flex;
  align-items: center;
}

.icon-emoji {
  margin-top: 3px;
  align-items: center;
  justify-content: center;
}

.app-title {
  font-size: 12px;
  font-weight: 500;
  color: #333;
}

.titlebar-controls {
  display: flex;
  height: 100%;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 46px;
  height: 100%;
  cursor: pointer;
  transition: background-color 0.2s;
  color: #333;
}

.titlebar-button:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.titlebar-button.close:hover {
  background-color: #e81123;
  color: white;
}

.titlebar-button .el-icon {
  font-size: 14px;
}
</style>
