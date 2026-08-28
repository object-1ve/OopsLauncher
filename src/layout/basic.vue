<script setup>
import { ref } from "vue";
import Sidebar from "@/components/Sidebar.vue";
import { useFiles } from "@/composables/useFiles";
import { useSettingsStore } from "@/stores/settingsStore";
import TitleBar from "@/components/TitleBar.vue";
const { currentCategory, switchCategory } = useFiles();
const settingsStore = useSettingsStore();

// 侧边栏宽度可调范围（px）
const MIN_SIDEBAR_WIDTH = 100;
const MAX_SIDEBAR_WIDTH = 500;

const isResizing = ref(false);
let dragStartX = 0;
let dragStartWidth = 0;

const startResize = (event) => {
  dragStartX = event.clientX;
  dragStartWidth = settingsStore.settings.appearance.sidebarWidth;
  isResizing.value = true;
  document.body.style.userSelect = "none";
  document.body.style.cursor = "col-resize";
  window.addEventListener("mousemove", handleResize);
  window.addEventListener("mouseup", stopResize);
  event.preventDefault();
};

const handleResize = (event) => {
  const delta = event.clientX - dragStartX;
  const width = Math.min(
    MAX_SIDEBAR_WIDTH,
    Math.max(MIN_SIDEBAR_WIDTH, dragStartWidth + delta)
  );
  settingsStore.settings.appearance.sidebarWidth = Math.round(width);
};

const stopResize = () => {
  isResizing.value = false;
  document.body.style.userSelect = "";
  document.body.style.cursor = "";
  window.removeEventListener("mousemove", handleResize);
  window.removeEventListener("mouseup", stopResize);
};
</script>

<template>
  <div class="layout-wrapper">
    <TitleBar />
    <div
      class="basic-layout"
      :style="{
        '--app-sidebar-width': settingsStore.settings.appearance.sidebarWidth + 'px',
      }"
    >
      <Sidebar
        class="page-side"
        :currentCategory="currentCategory"
        @categoryChange="switchCategory"
      />
      <div
        class="sidebar-resizer"
        :class="{ 'is-resizing': isResizing }"
        title="拖动调整侧边栏宽度"
        @mousedown="startResize"
      ></div>
      <div class="page-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.layout-wrapper {
  height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  background-color: var(--app-bg, #f5f7fa);
}

.basic-layout {
  display: grid;
  grid-template-columns: var(--app-sidebar-width, 150px) 6px 1fr;
  flex: 1;
  min-height: 0;
  width: 100%;
  overflow: hidden;
}

.sidebar-resizer {
  cursor: col-resize;
  position: relative;
  background: transparent;
  transition: background-color 0.2s ease;
  &::after {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 2px;
    background: var(--app-active-border, #409eff);
    opacity: 0;
    transition: opacity 0.2s ease;
  }
  &:hover::after,
  &.is-resizing::after {
    opacity: 1;
  }
  &.is-resizing {
    background: var(--app-border, #e8eaed);
  }
}

.page-content {
  height: 100%;
  min-height: 0;
  overflow: hidden;
  position: relative;
}
</style>
