<template>
  <div class="sidebar-aside">
    <div class="sidebar-menu">
      <!-- 常用分组 -->
      <div class="menu-group">
        <div class="menu-title" @click="toggleGroup('common')">
          <span>常用</span>
          <el-icon class="arrow-icon" :class="{ 'is-active': !collapsedGroups.common }">
            <ArrowRight />
          </el-icon>
        </div>
        <div class="menu-items" v-show="!collapsedGroups.common">
          <div 
            v-for="item in commonCategories" 
            :key="item"
            class="menu-item"
            :class="{ 'active': currentCategory === item }"
            @click="handleMenuSelect(item)"
          >
            {{ item }}
          </div>
        </div>
      </div>

      <!-- 分类分组 -->
      <div class="menu-group">
        <div class="menu-title" @click="toggleGroup('categories')">
          <span>分类</span>
          <el-icon class="arrow-icon" :class="{ 'is-active': !collapsedGroups.categories }">
            <ArrowRight />
          </el-icon>
        </div>
        <div class="menu-items" v-show="!collapsedGroups.categories">
          <div 
            v-for="item in categories" 
            :key="item"
            class="menu-item"
            :class="{ 'active': currentCategory === item }"
            @click="handleMenuSelect(item)"
          >
            {{ item }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { ArrowRight } from '@element-plus/icons-vue'

// Props
const props = defineProps({
  currentCategory: {
    type: String,
    default: 'main'
  }
})

// Emits
const emit = defineEmits(['categoryChange'])

// 分类数据
const commonCategories = ['main']
const categories = ['编程', '娱乐', '音乐', 'web3', '工具', '电影', '数据库', '项目']

// 折叠状态
const collapsedGroups = reactive({
  common: false,
  categories: false
})

// 方法：切换分组折叠状态
const toggleGroup = (group) => {
  collapsedGroups[group] = !collapsedGroups[group]
}

// 方法：处理菜单选择
const handleMenuSelect = (key) => {
  emit('categoryChange', key)
}
</script>

<style scoped>
/* 侧边栏样式 */
.sidebar-aside {
  width: 200px; /* 固定宽度 */
  background-color: #fafafa;
  display: flex;
  flex-direction: column;
  height: 100%; /* 确保侧边栏占满高度 */
  border-right: 1px solid rgba(0, 0, 0, 0.05);
}

.sidebar-menu {
  flex: 1;
  overflow-y: auto; /* 允许菜单内容滚动 */
  padding: 10px 0;
}

/* 分组样式 */
.menu-group {
  margin-bottom: 10px;
}

.menu-title {
  padding: 10px 20px;
  font-size: 14px;
  color: #909399;
  cursor: pointer; /* 标题现在可点击 */
  font-weight: 500;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: color 0.2s;
}

.menu-title:hover {
  color: #606266;
}

.arrow-icon {
  font-size: 12px;
  transition: transform 0.3s;
  color: #C0C4CC;
}

.arrow-icon.is-active {
  transform: rotate(90deg);
}

/* 菜单项样式 */
.menu-item {
  height: 40px;
  line-height: 40px;
  padding: 0 20px 0 40px; /* 增加左侧内边距以体现层级 */
  font-size: 14px;
  color: #303133;
  cursor: pointer;
  transition: background-color 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.menu-item:hover {
  background-color: rgba(0, 0, 0, 0.03);
}

.menu-item.active {
  background-color: rgba(0, 0, 0, 0.03);
  color: #303133; /* 保持文字颜色一致，或者可以使用主题色 */
  font-weight: 500;
  border-right: 3px solid #DCDEDF; /* 右侧高亮条 */
}

/* 滚动条样式 */
.sidebar-menu::-webkit-scrollbar {
  width: 6px;
}
.sidebar-menu::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 3px;
}
.sidebar-menu::-webkit-scrollbar-track {
  background-color: transparent;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sidebar-aside {
    width: 80px;
  }
  
  .menu-title span,
  .menu-item {
    display: none; /* 暂时隐藏文字，实际可能需要显示图标 */
  }
}
</style>