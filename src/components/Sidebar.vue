<template>
  <el-aside width="200px" class="sidebar-aside">
    <el-menu 
      :default-active="currentCategory"
      class="sidebar-menu" 
      @select="handleMenuSelect"
      unique-opened
    >
      <el-sub-menu index="common">
        <template #title>
          <span>常用</span>
        </template>
        <el-menu-item 
          v-for="item in commonCategories" 
          :key="item"
          :index="item"
        >
          {{ item }}
        </el-menu-item>
      </el-sub-menu>
      <el-sub-menu index="categories">
        <template #title>
          <span>分类</span>
        </template>
        <el-menu-item 
          v-for="item in categories" 
          :key="item"
          :index="item"
        >
          {{ item }}
        </el-menu-item>
      </el-sub-menu>
    </el-menu>
  </el-aside>
</template>

<script setup>
import { ref } from 'vue'

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

// 方法：处理菜单选择
const handleMenuSelect = (key) => {
  emit('categoryChange', key)
}
</script>

<style scoped>
/* 侧边栏样式 */
.sidebar-aside {
  background-color: #fafafa;
  display: flex;
  flex-direction: column;
  height: 100%; /* 确保侧边栏占满高度 */
}

.sidebar-header {
  padding: 20px 0 20px;
  text-align: center;
}

.sidebar-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
}

.sidebar-menu {
  border-right: none !important;
  background-color: transparent;
  flex: 1;
  overflow-y: auto; /* 允许菜单内容滚动 */
}

/* 覆盖 Element Plus 菜单样式 */
:deep(.el-menu) {
  background-color: transparent;
  border-right: none;
}

.el-sub-menu__title:hover,
.el-menu-item:hover{
  background-color: rgba(0, 0, 0, 0.03);
}

.el-menu-item.is-active{
  background-color: rgba(0, 0, 0, 0.03);
  border-right: 3px solid rgba(0, 0, 0, 0.03);
  color: black;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sidebar-aside {
    width: 80px !important;
  }
  
  .sidebar-title {
    font-size: 14px;
  }
  
  /* 隐藏文字，只留图标（如果有图标的话，目前没有，所以只是变窄） */
}
</style>