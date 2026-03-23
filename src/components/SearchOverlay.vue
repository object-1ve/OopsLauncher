<template>
  <transition name="fade">
    <div v-if="showSearchOverlay" class="search-overlay" @mousedown.self="closeSearch">
      <div class="search-container">
        <div class="search-box">
          <el-icon class="search-icon"><Search /></el-icon>
          <input
            ref="searchInput"
            v-model="searchQuery"
            class="search-input"
            placeholder="搜索所有分类下的文件..."
            @keyup.esc="closeSearch"
            @keydown.down.prevent="moveDown"
            @keydown.up.prevent="moveUp"
            @keyup.enter="handleEnter"
          />
        </div>
        
        <div v-if="searchQuery && globalSearchResults.length > 0" class="search-results">
          <div 
            v-for="(file, index) in globalSearchResults.slice(0, 15)" 
            :key="file.id" 
            class="result-item"
            :class="{ active: selectedIndex === index }"
            @click="handleSelect(file)"
            @mouseover="selectedIndex = index"
          >
            <div class="item-icon">
              <img v-if="file.icon && file.icon.startsWith('data:')" :src="file.icon" alt="" />
              <span v-else>{{ file.icon || '📄' }}</span>
            </div>
            <div class="item-info">
              <div class="item-name">{{ file.displayName || file.name }}</div>
              <div class="item-path">{{ file.path }}</div>
            </div>
            <div class="item-category" v-if="getCategoryName(file.category)">
              {{ getCategoryName(file.category) }}
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
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { Search } from '@element-plus/icons-vue'
import { useFiles } from '@/composables/useFiles'

const { 
  searchQuery, 
  globalSearchResults, 
  showSearchOverlay, 
  openFile,
  allCategories
} = useFiles()

const searchInput = ref(null)
const selectedIndex = ref(0)

const getCategoryName = (catId) => {
  const cat = allCategories.value.find(c => c.id === catId)
  return cat ? cat.name : ''
}

const closeSearch = () => {
  showSearchOverlay.value = false
  searchQuery.value = ''
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
</script>

<style scoped>
.search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.4);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 15vh;
}

.search-container {
  width: 600px;
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.search-box {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #f0f0f0;
}

.search-icon {
  font-size: 24px;
  color: #909399;
  margin-right: 16px;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 20px;
  color: #303133;
  background: transparent;
}

.search-input::placeholder {
  color: #c0c4cc;
}

.search-results {
  max-height: 450px;
  overflow-y: auto;
  padding: 8px 0;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s;
}

.result-item.active {
  background-color: #f0f7ff;
}

.item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
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
  font-size: 15px;
  color: #303133;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-path {
  font-size: 12px;
  color: #909399;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.item-category {
  font-size: 11px;
  padding: 2px 6px;
  background-color: #f0f2f5;
  color: #909399;
  border-radius: 4px;
  margin-left: 12px;
}

.no-results {
  padding: 40px;
  text-align: center;
  color: #909399;
  font-size: 16px;
}

.results-footer {
  padding: 10px 20px;
  text-align: center;
  font-size: 12px;
  color: #c0c4cc;
  border-top: 1px solid #f9f9f9;
}

.search-hints {
  padding: 12px 20px;
  background-color: #fafafa;
  display: flex;
  gap: 20px;
  border-top: 1px solid #f0f0f0;
}

.hint {
  font-size: 11px;
  color: #909399;
}

.hint b {
  color: #606266;
  background-color: #eee;
  padding: 1px 4px;
  border-radius: 3px;
  margin-right: 2px;
}

/* 动画 */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
