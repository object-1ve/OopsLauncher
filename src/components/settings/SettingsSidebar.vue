<template>
  <aside class="settings-sidebar">
    <nav class="sidebar-nav">
      <button
        v-for="item in navItems"
        :key="item.key"
        :class="['nav-item', { active: activeTab === item.key }]"
        @click="$emit('select', item.key)"
      >
        <span class="nav-icon" v-html="item.icon"></span>
        <span class="nav-label">{{ item.label }}</span>
      </button>
    </nav>
    <div class="sidebar-footer">
      <div class="version-badge">v{{ version }}</div>
    </div>
  </aside>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { ref, onMounted } from 'vue'

defineProps({
  activeTab: { type: String, required: true }
})

defineEmits(["select"])

const version = ref('')

onMounted(async () => {
  try {
    version.value = await invoke('get_app_version')
  } catch { version.value = '0.0.0' }
})

const navItems = [
  {
    key: 'general',
    label: '常规',
    icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>'
  },
  {
    key: 'appearance',
    label: '外观',
    icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>'
  },
  {
    key: 'shortcuts',
    label: '快捷键',
    icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3"/></svg>'
  },
  {
    key: 'about',
    label: '关于',
    icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>'
  }
]
</script>

<style scoped>
.settings-sidebar {
  width: 180px;
  min-width: 180px;
  background: var(--settings-sidebar-bg);
  border-right: 1px solid var(--settings-border);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 12px 0;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  border-radius: var(--settings-radius-sm);
  cursor: pointer;
  transition: all var(--settings-transition);
  color: var(--settings-text-secondary);
  font-family: inherit;
  font-size: 14px;
  font-weight: 500;
  text-align: left;
  position: relative;
}

.nav-item:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--settings-text-primary);
}

.nav-item.active {
  background: var(--el-color-primary-light-9, #ecf5ff);
  color: var(--el-color-primary, #409eff);
  font-weight: 600;
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-icon :deep(svg) {
  transition: transform 0.2s ease;
}

.nav-item:hover .nav-icon :deep(svg) {
  transform: scale(1.1);
}

.nav-label {
  line-height: 1;
}

.sidebar-footer {
  padding: 0 22px;
}

.version-badge {
  font-size: 11px;
  color: var(--settings-text-muted);
  font-weight: 500;
  letter-spacing: 0.02em;
  padding: 6px 0;
  border-top: 1px solid var(--settings-border);
}
</style>
