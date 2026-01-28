<template>
  <div id="app" @contextmenu.prevent>
    <!-- 路由视图 -->
    <router-view />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, watch } from "vue";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useSettings } from "@/composables/useSettings";
import { ElMessage } from "element-plus";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { invoke } from "@tauri-apps/api/core";

const { settings } = useSettings();
const appWindow = getCurrentWebviewWindow();

// 发送原生系统通知
const sendNativeNotification = async (title, body) => {
  console.log(`[Notification] Attempting to send: ${title} - ${body}`);
  try {
    let permissionGranted = await isPermissionGranted();
    console.log(`[Notification] Permission status: ${permissionGranted}`);
    
    if (!permissionGranted) {
      console.log(`[Notification] Requesting permission...`);
      const permission = await requestPermission();
      permissionGranted = permission === "granted";
      console.log(`[Notification] Permission request result: ${permission}`);
    }

    if (permissionGranted) {
      console.log(`[Notification] Sending via official JS API...`);
      // 使用官方推荐的 JS API
      sendNotification({ title, body });
      
    } else {
      console.warn(`[Notification] Permission denied by user.`);
      ElMessage.warning(`通知权限被拒绝: ${title}`);
    }
  } catch (err) {
    console.error("[Notification] Critical error:", err);
    ElMessage.error(`发送通知失败: ${err.message || err}`);
  }
};

const registerShowHideShortcut = async () => {
  if (appWindow.label !== "main") return;
  const shortcut = settings.value.shortcuts.showHide;
  if (!shortcut) return;

  try {
    try {
      await unregister(shortcut);
    } catch (e) {}
    await register(shortcut, async (event) => {
      if (event && event.state === "Released") return;
      const isVisible = await appWindow.isVisible();
      if (isVisible) {
        await appWindow.hide();
      } else {
        await appWindow.show();
        await appWindow.setFocus();
      }
    });
  } catch (err) {
    console.error("Failed to register show/hide shortcut:", err);
    if (err.includes("already registered")) {
      sendNativeNotification(
        "快捷键冲突",
        `显示/隐藏快捷键 ${shortcut} 已被其他程序占用。`
      );
    }
  }
};

const registerCopyTimeShortcut = async () => {
  if (appWindow.label !== "main") return;
  const shortcut = settings.value.shortcuts.copyTime;
  if (!shortcut) return;

  try {
    try {
      await unregister(shortcut);
    } catch (e) {}
    await register(shortcut, async (event) => {
      console.log("Copy time shortcut triggered:", event);
      if (event && event.state === "Released") return;

      try {
        const now = new Date();
        const timeStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(
          2,
          "0"
        )}-${String(now.getDate()).padStart(2, "0")}_${String(now.getHours()).padStart(
          2,
          "0"
        )}-${String(now.getMinutes()).padStart(2, "0")}-${String(
          now.getSeconds()
        ).padStart(2, "0")}`;

        await writeText(timeStr);
        await sendNativeNotification("时间已复制", timeStr);
        console.log("Time copied to clipboard:", timeStr);
      } catch (err) {
        console.error("Failed to copy time inside shortcut:", err);
      }
    });
  } catch (err) {
    console.error("Failed to register copy time shortcut:", err);
    if (err.includes("already registered")) {
      sendNativeNotification(
        "快捷键冲突",
        `复制时间快捷键 ${shortcut} 已被其他程序占用。`
      );
    }
  }
};

const registerTestNotificationShortcut = async () => {
  if (appWindow.label !== "main") return;
  const shortcut = settings.value.shortcuts.testNotification;
  if (!shortcut) return;

  try {
    try {
      await unregister(shortcut);
    } catch (e) {}
    await register(shortcut, async (event) => {
      if (event && event.state === "Released") return;
      const now = new Date();
      await sendNativeNotification(
        "OopsLauncher 通知测试",
        `触发时间：${now.toLocaleString()}`
      );
    });
  } catch (err) {
    console.error("Failed to register test notification shortcut:", err);
    if (err.includes("already registered")) {
      sendNativeNotification(
        "快捷键冲突",
        `测试通知快捷键 ${shortcut} 已被其他程序占用。`
      );
    }
  }
};

const registerAllShortcuts = async () => {
  await registerShowHideShortcut();
  await registerCopyTimeShortcut();
  await registerTestNotificationShortcut();
};

// 监听快捷键变化
watch(
  () => settings.value.shortcuts.showHide,
  async (newVal, oldVal) => {
    if (oldVal) await unregister(oldVal).catch(() => {});
    if (newVal) registerShowHideShortcut();
  }
);

watch(
  () => settings.value.shortcuts.copyTime,
  async (newVal, oldVal) => {
    if (oldVal) await unregister(oldVal).catch(() => {});
    if (newVal) registerCopyTimeShortcut();
  }
);

watch(
  () => settings.value.shortcuts.testNotification,
  async (newVal, oldVal) => {
    if (oldVal) await unregister(oldVal).catch(() => {});
    if (newVal) registerTestNotificationShortcut();
  }
);

// 监听透明度变化
watch(
  () => settings.value.appearance.transparency,
  async (val) => {
    if (appWindow && typeof appWindow.setOpacity === "function") {
      await appWindow.setOpacity(val);
    }
  },
  { immediate: true }
);

onMounted(async () => {
  try {
    // 只在主窗口注册快捷键，避免多窗口重复注册冲突
    if (appWindow.label === "main") {
      await registerAllShortcuts();
    }

    // 应用透明度
    if (appWindow && typeof appWindow.setOpacity === "function") {
      await appWindow.setOpacity(settings.value.appearance.transparency);
    }

    // 3. 强制显示窗口
    await appWindow.show();
    await appWindow.setFocus();
  } catch (error) {
    console.error("Failed to initialize app:", error);
    await appWindow.show().catch(() => {});
  }
});

onUnmounted(async () => {
  const { showHide, copyTime, testNotification } = settings.value.shortcuts;
  if (showHide) await unregister(showHide).catch(() => {});
  if (copyTime) await unregister(copyTime).catch(() => {});
  if (testNotification) await unregister(testNotification).catch(() => {});
});
</script>

<style scoped>
/* 主容器 */
#app {
  height: 100vh;
  width: 100%;
  box-sizing: border-box;
  background-color: #fff;
  overflow: hidden;
}
</style>
