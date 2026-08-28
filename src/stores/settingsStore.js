import { defineStore } from 'pinia'
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { isTauri } from '@/utils/env'

const defaultSettings = {
  general: {
    autoStart: false,
    autoStartMinimized: true,
    minimizeToTray: true,
    hideTaskbar: false,
    language: 'zh-CN',
    recentCount: 10,
  },
  appearance: {
    theme: 'light',
    transparency: 1.0,
    itemLayout: 'tile',
    iconSize: 48,
    sidebarWidth: 150,
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

const normalizeNumber = (value, fallback) => {
  const n = Number(value)
  return Number.isFinite(n) && n > 0 ? n : fallback
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

  merged.appearance.sidebarWidth = normalizeNumber(merged.appearance.sidebarWidth, defaultSettings.appearance.sidebarWidth)

  // 最近打开记录数量：0-50，整数
  const rc = Number(merged.general.recentCount)
  merged.general.recentCount = Number.isFinite(rc) ? Math.max(0, Math.min(50, Math.round(rc))) : defaultSettings.general.recentCount

  return merged
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref({ ...defaultSettings })
  const isLoading = ref(false)
  const isExternallyUpdating = ref(false)
  const isInitializing = ref(true) // 标记是否处于初始加载阶段，用于阻止 watch 显示成功提示

  let saveTimeout = null
  const saveSettings = (newSettings) => {
    if (isLoading.value || isExternallyUpdating.value) return
    
    // 防抖处理，避免频繁保存
    if (saveTimeout) clearTimeout(saveTimeout)
    saveTimeout = setTimeout(async () => {
      try {
        if (isTauri()) {
          await invoke('save_settings_to_json', { settings: newSettings })
        } else {
          localStorage.setItem('oopslauncher_settings', JSON.stringify(newSettings))
        }
      } catch (error) {
        console.error('Failed to save settings:', error)
      }
    }, 500) // 500ms 防抖
  }

  const loadSettings = async () => {
    isLoading.value = true
    isInitializing.value = true
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
      // 延迟一点点结束加载状态，确保 watch 不会被立即触发的异步保存动作影响
      setTimeout(() => {
        isLoading.value = false
        isInitializing.value = false
      }, 100)
    }
  }

  watch(settings, (newSettings) => {
    saveSettings(newSettings)
  }, { deep: true })

  // 监听 Tauri 事件（跨窗口同步）
  if (isTauri()) {
    listen('settings-changed', (event) => {
      const newSettings = event.payload
      if (JSON.stringify(newSettings) !== JSON.stringify(settings.value)) {
        isExternallyUpdating.value = true
        settings.value = mergeSettings(newSettings)
        nextTick(() => {
          isExternallyUpdating.value = false
        })
      }
    })
  }

  loadSettings()

  if (!isTauri()) {
    window.addEventListener('storage', (e) => {
      if (e.key === 'oopslauncher_settings' && e.newValue) {
        const next = JSON.parse(e.newValue)
        if (JSON.stringify(next) !== JSON.stringify(settings.value)) {
          isExternallyUpdating.value = true
          settings.value = mergeSettings(next)
          nextTick(() => {
            isExternallyUpdating.value = false
          })
        }
      }
    })
  }

  return {
    settings,
    loadSettings,
    isInitializing
  }
})
