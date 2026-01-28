<template>
  <div class="sidebar-aside" @contextmenu.prevent="handleSidebarContextMenu">
    <div class="sidebar-menu">
      <div class="menu-items">
        <SidebarItem
          v-for="item in allCategories"
          :key="item.id"
          :item="item"
          :currentCategory="currentCategory"
          @categoryChange="handleMenuSelect"
          @itemContextMenu="handleItemContextMenu"
        />

        <!-- 添加分类输入框 -->
        <div v-if="showAddCategoryInput" class="add-category-input">
          <el-input
            v-model="newCategoryName"
            placeholder="分类名称"
            size="small"
            @keyup.enter="handleAddCategory"
            @blur="handleInputBlur"
            ref="categoryInput"
          />
        </div>
      </div>
    </div>

    <!-- 侧边栏右键菜单 -->
    <div
      v-if="sidebarContextMenu && sidebarContextMenu.visible"
      class="sidebar-context-menu"
      :style="{
        left: sidebarContextMenu.x + 'px',
        top: sidebarContextMenu.y + 'px',
      }"
    >
      <ul class="context-menu-list">
        <template v-if="sidebarContextMenu.item">
          <li class="context-menu-item" @click="handleRenameFromMenu">
            <el-icon><Edit /></el-icon> 重命名
          </li>
        </template>
        <template v-else>
          <li class="context-menu-item" @click="handleAddCategoryFromMenu">
            <el-icon><Plus /></el-icon> 添加分类
          </li>
        </template>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, nextTick, watch, computed, onMounted, onUnmounted } from "vue";
import { Plus, Edit, Setting } from "@element-plus/icons-vue";
import { useFiles } from "@/composables/useFiles";
import SidebarItem from "./SidebarItem.vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// Props
const props = defineProps({
  currentCategory: {
    type: String,
    default: "main",
  },
});

// Emits
const emit = defineEmits(["categoryChange"]);

const { allCategories, addCategory } = useFiles();

// 右键菜单状态
const sidebarContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  item: null,
  startRename: null,
});

// 状态
const showAddCategoryInput = ref(false);
const newCategoryName = ref("");
const categoryInput = ref(null);

// 处理侧边栏右键
const handleSidebarContextMenu = (e) => {
  // 检查是否点击的是空白区域
  if (
    e.target.classList.contains("sidebar-aside") ||
    e.target.classList.contains("sidebar-menu") ||
    e.target.classList.contains("menu-items")
  ) {
    sidebarContextMenu.value.visible = true;
    sidebarContextMenu.value.x = e.clientX;
    sidebarContextMenu.value.y = e.clientY;
    sidebarContextMenu.value.item = null;
    sidebarContextMenu.value.startRename = null;
  }
};

// 处理项右键
const handleItemContextMenu = ({ event, item, startRename }) => {
  sidebarContextMenu.value.visible = true;
  sidebarContextMenu.value.x = event.clientX;
  sidebarContextMenu.value.y = event.clientY;
  sidebarContextMenu.value.item = item;
  sidebarContextMenu.value.startRename = startRename;
};

// 隐藏右键菜单
const hideContextMenu = () => {
  sidebarContextMenu.value.visible = false;
};

// 从菜单添加分类
const handleAddCategoryFromMenu = () => {
  openAddInput();
  hideContextMenu();
};

// 从菜单重命名
const handleRenameFromMenu = () => {
  if (sidebarContextMenu.value.startRename) {
    sidebarContextMenu.value.startRename();
  }
  hideContextMenu();
};

// 方法：处理菜单选择
const handleMenuSelect = (key) => {
  emit("categoryChange", key);
};

// 方法：打开添加输入框
const openAddInput = () => {
  showAddCategoryInput.value = true;
};

// 方法：处理添加分类
const handleAddCategory = async () => {
  if (newCategoryName.value.trim()) {
    await addCategory(newCategoryName.value.trim());
    newCategoryName.value = "";
    showAddCategoryInput.value = false;
  }
};

// 方法：处理输入框失去焦点
const handleInputBlur = () => {
  if (!newCategoryName.value.trim()) {
    showAddCategoryInput.value = false;
  }
};

// 监听输入框显示，自动聚焦
watch(showAddCategoryInput, (val) => {
  if (val) {
    nextTick(() => {
      categoryInput.value?.focus();
    });
  }
});

onMounted(() => {
  document.addEventListener("click", hideContextMenu);
});

onUnmounted(() => {
  document.removeEventListener("click", hideContextMenu);
});
</script>

<style scoped>
/* 侧边栏样式 */
.sidebar-aside {
  width: 150px; /* 固定宽度 */
  background-color: #fafafa;
  display: flex;
  flex-direction: column;
  height: 100%; /* 确保侧边栏占满高度 */
  border-right: 1px solid #f5f5f5;
}
.sidebar-menu {
  flex: 1;
  overflow-y: auto; /* 允许菜单内容滚动 */
}

.sidebar-footer {
  padding: 10px 0;
  border-top: 1px solid #f0f0f0;
}

.footer-item {
  height: 40px;
  padding: 0 20px 0 40px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: #606266;
  cursor: pointer;
  transition: all 0.2s;
}

.footer-item:hover {
  background-color: #f0f0f0;
  color: #409eff;
}

.footer-item .el-icon {
  font-size: 16px;
}

/* 分组样式 */
.add-category-input {
  padding: 5px 20px 5px 40px;
}

/* 右键菜单样式 */
.sidebar-context-menu {
  position: fixed;
  z-index: 1000;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  min-width: 120px;
  padding: 5px 0;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #303133;
}

.context-menu-item:hover {
  background-color: #f5f5f5;
  color: #409eff;
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
