import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 检测是否在 Tauri 环境中运行
const isTauri = () => !!window.__TAURI_INTERNALS__;

const defaultSettings = {
  general: {
    autoStart: false,
    autoStartMinimized: true,
    minimizeToTray: true,
    hideTaskbar: false,
    language: 'zh-CN',
  },
  appearance: {
    theme: 'light',
    transparency: 1.0,
    itemLayout: 'tile', // 'tile' or 'list'
    iconSize: 48,
    showFileName: true,
    css: {
      primaryColor: '#409EFF',
      secondaryColor: '#67C23A',
      backgroundColor: '#F5F7FA',
      textColor: '#303133',
      borderColor: '#E4E7ED',
      hoverColor: '#ECF5FF',
      borderRadius: '4px',
      itemMargin: '4px',
      itemPadding: '8px',
      fontSize: '13px',
      lineHeight: '1.4'
    }
  },
  shortcuts: {
    showHide: 'Alt+Shift+Space',
    copyTime: 'Alt+T',
    testNotification: 'Ctrl+Alt+N',
    notificationIcon: '',
  }
}

const settings = ref({ ...defaultSettings })

let isLoading = false

const normalizePx = (value, fallback) => {
  const n = parseInt(value, 10)
  return Number.isFinite(n) && n > 0 ? `${n}px` : fallback
}

const mergeSettings = (source = {}) => {
  const merged = {
    ...defaultSettings,
    ...source,
    general: {
      ...defaultSettings.general,
      ...(source.general || {}),
    },
    appearance: {
      ...defaultSettings.appearance,
      ...(source.appearance || {}),
      css: {
        ...defaultSettings.appearance.css,
        ...(source.appearance?.css || {}),
      },
    },
    shortcuts: {
      ...defaultSettings.shortcuts,
      ...(source.shortcuts || {}),
    },
  }

  merged.appearance.css.itemPadding = normalizePx(merged.appearance.css.itemPadding, defaultSettings.appearance.css.itemPadding)
  merged.appearance.css.itemMargin = normalizePx(merged.appearance.css.itemMargin, defaultSettings.appearance.css.itemMargin)

  return merged
}

const loadSettings = async () => {
  isLoading = true
  try {
    let storedSettings = null
    if (isTauri()) {
      storedSettings = await invoke('load_settings_from_json')
    } else {
      const storedSettingsRaw = localStorage.getItem('oopslauncher_settings')
      storedSettings = storedSettingsRaw ? JSON.parse(storedSettingsRaw) : null
    }

    if (storedSettings) {
      settings.value = mergeSettings(storedSettings)
    }
  } catch (error) {
    console.error('Failed to load settings:', error)
  } finally {
    isLoading = false
  }
}

const saveSettings = async (newSettings) => {
  if (isLoading) return
  try {
    if (isTauri()) {
      await invoke('save_settings_to_json', { settings: newSettings })
    } else {
      localStorage.setItem('oopslauncher_settings', JSON.stringify(newSettings))
    }
  } catch (error) {
    console.error('Failed to save settings:', error)
  }
}

watch(settings, (newSettings) => {
  saveSettings(newSettings)
}, { deep: true })

// 立即尝试加载设置
loadSettings()

// 监听其他窗口修改存储 (仅浏览器环境)
if (!isTauri()) {
  window.addEventListener('storage', (e) => {
    if (e.key === 'oopslauncher_settings' && e.newValue) {
      const next = JSON.parse(e.newValue)
      settings.value = mergeSettings(next)
    }
  })
}

export function useSettings() {
  return {
    settings,
    loadSettings
  }
}
