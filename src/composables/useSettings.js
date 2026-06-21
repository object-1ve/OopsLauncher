import { storeToRefs } from 'pinia'
import { useSettingsStore } from '@/stores/settingsStore'

export function useSettings() {
  const settingsStore = useSettingsStore()
  const { settings, isInitializing } = storeToRefs(settingsStore)
  return {
    settings,
    loadSettings: settingsStore.loadSettings,
    isInitializing
  }
}
