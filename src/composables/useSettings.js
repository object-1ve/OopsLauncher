import { ref, watch } from 'vue'

const defaultSettings = {
  general: {
    autoStart: false,
    minimizeToTray: true,
    language: 'zh-CN',
  },
  appearance: {
    theme: 'light',
    transparency: 1.0,
    itemLayout: 'tile', // 'tile' or 'list'
    iconSize: 48,
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

const storedSettingsRaw = localStorage.getItem('oopslauncher_settings')
const storedSettings = storedSettingsRaw ? JSON.parse(storedSettingsRaw) : null

const settings = ref({
  ...defaultSettings,
  ...(storedSettings || {}),
  general: {
    ...defaultSettings.general,
    ...(storedSettings?.general || {}),
  },
  appearance: {
    ...defaultSettings.appearance,
    ...(storedSettings?.appearance || {}),
  },
  shortcuts: {
    ...defaultSettings.shortcuts,
    ...(storedSettings?.shortcuts || {}),
  },
})

watch(settings, (newSettings) => {
  localStorage.setItem('oopslauncher_settings', JSON.stringify(newSettings))
}, { deep: true })

// 监听其他窗口修改存储
window.addEventListener('storage', (e) => {
  if (e.key === 'oopslauncher_settings' && e.newValue) {
    const next = JSON.parse(e.newValue)
    settings.value = {
      ...defaultSettings,
      ...(next || {}),
      general: {
        ...defaultSettings.general,
        ...(next?.general || {}),
      },
      appearance: {
        ...defaultSettings.appearance,
        ...(next?.appearance || {}),
      },
      shortcuts: {
        ...defaultSettings.shortcuts,
        ...(next?.shortcuts || {}),
      },
    }
  }
})

export function useSettings() {
  return {
    settings
  }
}
