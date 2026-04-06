<template>
  <div class="settings-view">
    <SettingsHeader @close="closeWindow" />
    <el-container class="settings-container">
      <SettingsSidebar :active-tab="activeTab" @select="handleSelect" />
      <el-main class="settings-main">
        <div class="content-wrapper">
          <GeneralSettingsSection v-if="activeTab === 'general'" :settings="settings" />
          <AppearanceSettingsSection
            v-if="activeTab === 'appearance'"
            :settings="settings"
            :theme-hue="themeHue"
            :theme-mode="themeMode"
            :active-collapse-names="activeCollapseNames"
            @update:theme-hue="themeHue = $event"
            @update:theme-mode="themeMode = $event"
            @update:active-collapse-names="activeCollapseNames = $event"
          />
          <ShortcutsSettingsSection
            v-if="activeTab === 'shortcuts'"
            :settings="settings"
            @shortcut-key-down="handleShortcutKeyDown"
          />
          <AboutSettingsSection v-if="activeTab === 'about'" :app-version="appVersion" />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup>
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useSettings } from "@/composables/useSettings";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { hexToHsl, hslToHex, mixHex } from "@/utils/color";
import SettingsHeader from "@/components/settings/SettingsHeader.vue";
import SettingsSidebar from "@/components/settings/SettingsSidebar.vue";
import GeneralSettingsSection from "@/components/settings/GeneralSettingsSection.vue";
import AppearanceSettingsSection from "@/components/settings/AppearanceSettingsSection.vue";
import ShortcutsSettingsSection from "@/components/settings/ShortcutsSettingsSection.vue";
import AboutSettingsSection from "@/components/settings/AboutSettingsSection.vue";

const { settings } = useSettings();
const activeTab = ref("general");
const activeCollapseNames = ref([]);
const appWindow = getCurrentWebviewWindow();
const appVersion = ref("");
const themeHue = ref(0);
const themeMode = ref(0);
const isUpdatingHue = ref(false);
const isUpdatingMode = ref(false);

const closeWindow = async () => {
  await appWindow.close();
};

const fetchAppVersion = async () => {
  try {
    appVersion.value = await invoke("get_app_version");
  } catch (err) {
    console.error("Failed to fetch app version:", err);
    appVersion.value = "Unknown";
  }
};

const handleGlobalKeyDown = (e) => {
  if (e.key === "Escape") {
    closeWindow();
  }
};

const lightTheme = {
  bg: '#F5F7FA',
  text: '#303133',
  border: '#E4E7ED',
  hover: '#ECF5FF'
};

const darkTheme = {
  bg: '#2c2c2c',
  text: '#ffffff',
  border: '#4c4c4c',
  hover: '#3c3c3c'
};

const initTheme = () => {
  const css = settings.value.appearance.css;

  if (css.primaryColor) {
    const hsl = hexToHsl(css.primaryColor);
    themeHue.value = hsl.h;
  }

  if (css.backgroundColor) {
    const hsl = hexToHsl(css.backgroundColor);
    let mode = (96 - hsl.l) / (96 - 17) * 100;
    mode = Math.max(0, Math.min(100, mode));
    themeMode.value = Math.round(mode);
  }
};

watch(themeMode, (newMode) => {
  if (isUpdatingMode.value) return;

  const weight = newMode / 100;

  isUpdatingMode.value = true;

  settings.value.appearance.css.backgroundColor = mixHex(lightTheme.bg, darkTheme.bg, weight);
  settings.value.appearance.css.textColor = mixHex(lightTheme.text, darkTheme.text, weight);
  settings.value.appearance.css.borderColor = mixHex(lightTheme.border, darkTheme.border, weight);
  settings.value.appearance.css.hoverColor = mixHex(lightTheme.hover, darkTheme.hover, weight);

  if (newMode < 50) {
    if (settings.value.appearance.theme !== 'light') settings.value.appearance.theme = 'light';
  } else {
    if (settings.value.appearance.theme !== 'dark') settings.value.appearance.theme = 'dark';
  }
  
  nextTick(() => {
    isUpdatingMode.value = false;
  });
});

watch(
  () => settings.value.appearance.theme,
  (newTheme) => {
    if (isUpdatingMode.value) return;

    if (newTheme === 'light' && themeMode.value > 10) {
      themeMode.value = 0;
    } else if (newTheme === 'dark' && themeMode.value < 90) {
      themeMode.value = 100;
    }
  }
);

watch(themeHue, (newHue) => {
  if (isUpdatingHue.value) return;

  const currentPrimary = settings.value.appearance.css.primaryColor;
  const currentHsl = hexToHsl(currentPrimary);

  if (Math.abs(currentHsl.h - newHue) > 1) {
    let s = currentHsl.s;
    let l = currentHsl.l;

    if (s < 10) s = 80;
    if (l < 10 || l > 90) l = 50;

    const newHex = hslToHex(newHue, s, l);

    isUpdatingHue.value = true;
    settings.value.appearance.css.primaryColor = newHex;

    nextTick(() => {
      isUpdatingHue.value = false;
    });
  }
});

watch(
  () => settings.value.appearance.css.primaryColor,
  (newVal) => {
    if (isUpdatingHue.value) return;

    const hsl = hexToHsl(newVal);
    if (Math.abs(themeHue.value - hsl.h) > 2) {
      isUpdatingHue.value = true;
      themeHue.value = hsl.h;
      nextTick(() => {
        isUpdatingHue.value = false;
      });
    }
  }
);

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKeyDown);
  fetchAppVersion();
  initTheme();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

const updateAutoStart = async () => {
  const isEnabled = settings.value.general.autoStart;
  const isMinimized = settings.value.general.autoStartMinimized;

  try {
    if (isEnabled) {
      const args = isMinimized ? ["--minimized"] : [];
      await enable(args);
    } else {
      await disable();
    }
  } catch (err) {
    console.error("Failed to update autostart:", err);
  }
};

watch(
  [() => settings.value.general.autoStart, () => settings.value.general.autoStartMinimized],
  async () => {
    await updateAutoStart();
  }
);

watch(
  () => settings.value.general.hideTaskbar,
  async (val) => {
    if (!window.__TAURI_INTERNALS__) return;
    try {
      await invoke("set_skip_taskbar", { skip: val });
    } catch (err) {
      console.error("Failed to set skip taskbar from settings window:", err);
    }
  }
);

const handleSelect = (index) => {
  activeTab.value = index;
};

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

.settings-container {
  flex: 1;
  min-height: 0;
}

.settings-main {
  padding: 0;
  background-color: #fff;
  overflow: auto;
}

.content-wrapper {
  padding: 40px;
  max-width: 600px;
  margin: 0 auto;
}
</style>
