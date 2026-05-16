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
  },
  appearance: {
    theme: 'light',
    transparency: 1.0,
    itemLayout: 'tile',
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

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref({ ...defaultSettings })
  const isLoading = ref(false)
  const isExternallyUpdating = ref(false)

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
    loadSettings
  }
})
