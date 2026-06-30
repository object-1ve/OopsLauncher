<template>
  <transition name="fade">
    <div v-if="showSearchOverlay" class="search-overlay" @mousedown.self="closeSearch">
      <div class="search-container">
        <div class="search-box">
          <el-icon class="search-icon">
            <Search />
          </el-icon>
          <input ref="searchInput" v-model="searchQuery" class="search-input" placeholder="搜索所有分类下的文件..."
            @keyup.esc="closeSearch" @keydown.down.prevent="moveDown" @keydown.up.prevent="moveUp"
            @keyup.enter="handleEnter" />
        </div>

        <div v-if="searchQuery && globalSearchResults.length > 0" class="search-results">
          <div v-for="(file, index) in globalSearchResults.slice(0, 15)" :key="file.id" class="result-item"
            :class="{ active: selectedIndex === index, 'file-not-found': file.exists === false }"
            @click="handleSelect(file)" @mouseover="selectedIndex = index"
            @contextmenu.prevent.stop="handleResultContextMenu($event, file)">
            <div class="item-icon">
              <img v-if="file.icon && file.icon.startsWith('data:')" :src="file.icon" alt="" />
              <span v-else>{{ file.icon || '📄' }}</span>
            </div>
            <div class="item-info">
              <div class="item-name">{{ file.displayName || file.name }}</div>
              <div class="item-path">{{ file.path }}</div>
            </div>
            <div class="item-category" :class="{ 'is-start-menu': file.category === SPECIAL_CATEGORIES.START_MENU }"
              v-if="getCategoryName(file.category)">
              {{ getCategoryName(file.category) }}
            </div>
            <div class="match-reason" v-if="file.matchReason">
              {{ file.matchReason }}
            </div>
          </div>
          <div v-if="globalSearchResults.length > 15" class="results-footer">
            更多结果请缩小搜索范围...
          </div>
        </div>

        <div v-else-if="searchQuery" class="no-results">
          未找到相关文件
        </div>

        <div class="search-hints">
          <span class="hint"><b>ESC</b> 退出</span>
          <span class="hint"><b>↑↓</b> 选择</span>
          <span class="hint"><b>ENTER</b> 打开</span>
        </div>
      </div>
    </div>
  </transition>
  <div v-if="resultMenu.visible" class="result-context-menu"
    :style="{ left: `${resultMenu.x}px`, top: `${resultMenu.y}px` }" @click.stop @contextmenu.prevent>
    <div class="result-context-item" @click="handleLocateFile">
      定位到此文件
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { Search } from '@element-plus/icons-vue'
import { useFiles } from '@/composables/useFiles'

const {
  searchQuery,
  globalSearchResults,
  showSearchOverlay,
  openFile,
  allCategories,
  switchCategory,
  SPECIAL_CATEGORIES
} = useFiles()

const searchInput = ref(null)
const selectedIndex = ref(0)
const resultMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  file: null
})

const getCategoryName = (catId) => {
  if (catId === SPECIAL_CATEGORIES.START_MENU) return '开始菜单'
  const cat = allCategories.value.find(c => c.id === catId)
  return cat ? cat.name : ''
}

const closeSearch = () => {
  showSearchOverlay.value = false
  searchQuery.value = ''
  hideResultMenu()
}

const handleSelect = (file) => {
  openFile(file)
  closeSearch()
}

const handleEnter = () => {
  if (globalSearchResults.value.length > 0) {
    handleSelect(globalSearchResults.value[selectedIndex.value])
  }
}

const hideResultMenu = () => {
  resultMenu.value.visible = false
  resultMenu.value.file = null
}

const handleResultContextMenu = (event, file) => {
  resultMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    file
  }
}

const handleLocateFile = () => {
  const file = resultMenu.value.file
  if (!file) return
  if (file.category) {
    switchCategory(file.category)
  }
  hideResultMenu()
  closeSearch()
  window.dispatchEvent(new CustomEvent('locate-file', {
    detail: {
      fileId: file.id,
      categoryId: file.category
    }
  }))
}

const handleDocumentClick = () => {
  if (resultMenu.value.visible) {
    hideResultMenu()
  }
}

const moveDown = () => {
  const max = Math.min(globalSearchResults.value.length, 15) - 1
  if (selectedIndex.value < max) {
    selectedIndex.value++
  }
}

const moveUp = () => {
  if (selectedIndex.value > 0) {
    selectedIndex.value--
  }
}

watch(showSearchOverlay, (val) => {
  if (val) {
    selectedIndex.value = 0
    nextTick(() => {
      searchInput.value?.focus()
    })
  }
})

watch(searchQuery, () => {
  selectedIndex.value = 0
})

onMounted(() => {
  document.addEventListener('click', handleDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
})
</script>

<style scoped>
.search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.search-container {
  width: 560px;
  background-color: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 14px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2), 0 2px 8px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: 1px solid rgba(255, 255, 255, 0.5);
}

.search-box {
  display: flex;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid var(--app-border-light, #f0f0f0);
}

.search-icon {
  font-size: 22px;
  color: var(--app-text-muted, #909399);
  margin-right: 14px;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 18px;
  font-weight: 500;
  color: var(--app-text-primary, #1a1a2e);
  background: transparent;
}

.search-input::placeholder {
  color: var(--app-text-muted, #909399);
  font-weight: 400;
}

.search-results {
  max-height: 420px;
  overflow-y: auto;
  padding: 6px;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  cursor: pointer;
  transition: all var(--app-transition, 0.2s);
  border-radius: var(--app-radius-sm, 6px);
}

.result-item:hover {
  background-color: var(--app-hover, #f5f7fa);
}

.result-item.active {
  background-color: var(--app-active-bg, #ecf5ff);
}

.result-item.file-not-found {
  opacity: 0.5;
}

.result-item.file-not-found .item-name {
  color: #f56c6c;
  text-decoration: line-through;
}

.item-icon {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 14px;
  flex-shrink: 0;
}

.item-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.item-icon span {
  font-size: 20px;
}

.item-info {
  flex: 1;
  overflow: hidden;
}

.item-name {
  font-size: 14px;
  color: var(--app-text-primary, #1a1a2e);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-path {
  font-size: 12px;
  color: var(--app-text-muted, #909399);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.item-category {
  font-size: 11px;
  padding: 2px 8px;
  background-color: var(--app-hover, #f5f7fa);
  color: var(--app-text-muted, #909399);
  border-radius: 4px;
  margin-left: 12px;
  flex-shrink: 0;
  font-weight: 500;
}

.item-category.is-start-menu {
  background-color: var(--app-active-bg, #ecf5ff);
  color: var(--app-active-border, #409eff);
}

.match-reason {
  font-size: 10px;
  padding: 2px 6px;
  background-color: #fef0f0;
  color: #f56c6c;
  border-radius: 4px;
  margin-left: 8px;
  white-space: nowrap;
  flex-shrink: 0;
}

.no-results {
  padding: 40px;
  text-align: center;
  color: var(--app-text-muted, #909399);
  font-size: 15px;
}

.results-footer {
  padding: 10px 20px;
  text-align: center;
  font-size: 12px;
  color: var(--app-text-muted, #909399);
  border-top: 1px solid var(--app-border-light, #f0f0f0);
}

.search-hints {
  padding: 10px 18px;
  background-color: rgba(249, 250, 251, 0.8);
  display: flex;
  gap: 16px;
  border-top: 1px solid var(--app-border-light, #f0f0f0);
}

.hint {
  font-size: 11px;
  color: var(--app-text-muted, #909399);
}

.hint b {
  color: var(--app-text-secondary, #606266);
  background-color: rgba(0, 0, 0, 0.05);
  padding: 1px 5px;
  border-radius: 3px;
  margin-right: 2px;
  font-weight: 600;
}

/* 动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active .search-container {
  animation: slideUp 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.result-context-menu {
  position: fixed;
  z-index: 10001;
  background-color: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border, #e8eaed);
  border-radius: var(--app-radius-md, 8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 4px rgba(0, 0, 0, 0.04);
  min-width: 140px;
  padding: 4px;
}

.result-context-item {
  padding: 7px 12px;
  font-size: 13px;
  color: var(--app-text-primary, #1a1a2e);
  cursor: pointer;
  border-radius: var(--app-radius-sm, 6px);
  transition: all var(--app-transition, 0.2s);
}

.result-context-item:hover {
  background-color: var(--app-hover, #f5f7fa);
  color: var(--app-active-border, #409eff);
}
</style>
