<template>
  <div class="sidebar-aside" @contextmenu.prevent="handleSidebarContextMenu">
    <div class="sidebar-menu">
      <!-- 特殊分类：全部文件 -->
      <div class="special-categories">
        <div class="menu-item special-item" :data-drop-category="SPECIAL_CATEGORIES.ALL_FILES" :class="{
          active: currentCategory === SPECIAL_CATEGORIES.ALL_FILES,
          'is-dragover': specialDragOver === SPECIAL_CATEGORIES.ALL_FILES || dragTargetCategory === SPECIAL_CATEGORIES.ALL_FILES
        }" @click="handleMenuSelect(SPECIAL_CATEGORIES.ALL_FILES)"
          @dragover.prevent="handleSpecialDragOver(SPECIAL_CATEGORIES.ALL_FILES)" @dragleave="handleSpecialDragLeave"
          @drop="handleSpecialDrop($event, SPECIAL_CATEGORIES.ALL_FILES)">
          <div class="item-content">
            <span class="item-name">全部文件</span>
          </div>
        </div>
        <div class="menu-item special-item" :class="{ active: currentCategory === SPECIAL_CATEGORIES.FILE_EXPLORER }"
          @click="handleMenuSelect(SPECIAL_CATEGORIES.FILE_EXPLORER)">
          <div class="item-content">
            <span class="item-name">资源管理器</span>
          </div>
        </div>
        <div class="menu-item special-item" :class="{
          active: currentCategory === SPECIAL_CATEGORIES.START_MENU,
          'is-refreshing': refreshingStartMenu
        }" @click="handleMenuSelect(SPECIAL_CATEGORIES.START_MENU)">
          <div class="item-content">
            <span class="item-name">开始菜单</span>
          </div>
        </div>
      </div>

      <draggable v-model="allCategories" item-key="id" class="menu-items" animation="150" ghost-class="sortable-ghost"
        drag-class="sortable-drag" :disabled="false" :force-fallback="true" :fallback-tolerance="3"
        @start="handleDragStart" @move="handleDragMove" @end="handleDragEnd">
        <template #item="{ element: item }">
          <SidebarItem :item="item" :currentCategory="currentCategory" @categoryChange="handleMenuSelect"
            @itemContextMenu="handleItemContextMenu" />
        </template>
        <template #footer>
          <!-- 添加分类输入框 -->
          <div v-if="showAddCategoryInput" class="add-category-input">
            <el-input v-model="newCategoryName" placeholder="分类名称" size="small" @keyup.enter="handleAddCategory"
              @blur="handleInputBlur" ref="categoryInput" />
          </div>
        </template>
      </draggable>
    </div>

    <!-- 侧边栏右键菜单 -->
    <div v-if="sidebarContextMenu && sidebarContextMenu.visible" class="sidebar-context-menu" :style="{
      left: sidebarContextMenu.x + 'px',
      top: sidebarContextMenu.y + 'px',
    }">
      <ul class="context-menu-list">
        <template v-if="sidebarContextMenu.item">
          <li class="context-menu-item" @click="handleRenameFromMenu">
            重命名
          </li>
          <li class="context-menu-item delete" @click="handleDeleteFromMenu">
            删除
          </li>
        </template>
        <template v-else>
          <li class="context-menu-item" @click="handleAddCategoryFromMenu">
            添加分类
          </li>
        </template>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, nextTick, watch, computed, onMounted, onUnmounted } from "vue";
import { Plus, Edit, Setting, Delete } from "@element-plus/icons-vue";
import { useFiles } from "@/composables/useFiles";
import { useContextMenu } from "@/composables/useContextMenu";
import SidebarItem from "./SidebarItem.vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ElMessageBox, ElMessage } from "element-plus";
import draggable from "vuedraggable";

// Props
const props = defineProps({
  currentCategory: {
    type: String,
    default: "main",
  },
});

// Emits
const emit = defineEmits(["categoryChange"]);

const {
  allCategories, addCategory, deleteCategory, updateCategoryOrder,
  processFiles, SPECIAL_CATEGORIES, dragTargetCategory, refreshingStartMenu
} = useFiles();
const { registerMenu, unregisterMenu } = useContextMenu();

const specialDragOver = ref(null);

const handleSpecialDragOver = (id) => {
  specialDragOver.value = id;
  dragTargetCategory.value = id;
};

const handleSpecialDragLeave = () => {
  specialDragOver.value = null;
  dragTargetCategory.value = null;
};

const handleSpecialDrop = async (event, categoryId) => {
  event.preventDefault();
  event.stopPropagation();
  specialDragOver.value = null;
  dragTargetCategory.value = null;

  // 1. 处理内部拖拽
  const internalData = event.dataTransfer.getData('application/x-oops-file');
  if (internalData) {
    try {
      const file = JSON.parse(internalData);
      const result = await processFiles([file], categoryId);
      handleProcessResult(result, categoryId);
      return;
    } catch (err) {
      console.error('Failed to process internal drop:', err);
    }
  }

  // 2. 处理外部拖拽
  const files = Array.from(event.dataTransfer.files);
  if (files.length > 0) {
    const result = await processFiles(files, categoryId);
    handleProcessResult(result, categoryId);
  }
};

const handleProcessResult = (result, categoryId) => {
  if (result.addedCount > 0) {
    const targetCat = allCategories.value.find(c => c.id === (result.targetCategory || categoryId));
    const catName = targetCat ? `「${targetCat.name}」` : '默认分类';
    ElMessage.success(`成功添加 ${result.addedCount} 个文件到 ${catName}`);
  } else if (result.existingCount > 0) {
    ElMessage.warning(`文件已存在`);
  } else if (result.error === 'cannot_add_to_special_category') {
    ElMessage.warning('请先创建一个分类再添加文件');
  }
};

// 侧边栏元素引用
const menuItemsRef = ref(null);

const handleDragStart = (evt) => {
  console.log('Drag started:', evt);
  console.log('Dragged element:', evt.item);
  console.log('Start index:', evt.oldIndex);
  console.log('All categories:', allCategories.value);
};

const handleDragMove = (evt, originalEvent) => {
  console.log('Drag moving:', evt);
  console.log('Original event:', originalEvent);
  console.log('Related element:', evt.related);
  console.log('Drag operation:', evt.operation);
  return true; // 允许所有移动操作
};

const handleDragEnd = async (evt) => {
  console.log('Drag ended:', evt);
  console.log('Old index:', evt.oldIndex, 'New index:', evt.newIndex);
  console.log('Updated categories:', allCategories.value);
  // vuedraggable 已经通过 v-model 更新了 allCategories.value
  // 我们只需要调用 updateCategoryOrder 来持久化到数据库
  await updateCategoryOrder(allCategories.value);
};

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
    registerMenu(hideContextMenu);
    sidebarContextMenu.value.visible = true;
    sidebarContextMenu.value.x = e.clientX;
    sidebarContextMenu.value.y = e.clientY;
    sidebarContextMenu.value.item = null;
    sidebarContextMenu.value.startRename = null;
  }
};

// 处理项右键
const handleItemContextMenu = ({ event, item, startRename }) => {
  registerMenu(hideContextMenu);
  sidebarContextMenu.value.visible = true;
  sidebarContextMenu.value.x = event.clientX;
  sidebarContextMenu.value.y = event.clientY;
  sidebarContextMenu.value.item = item;
  sidebarContextMenu.value.startRename = startRename;
};

// 隐藏右键菜单
const hideContextMenu = () => {
  sidebarContextMenu.value.visible = false;
  unregisterMenu();
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

// 从菜单删除
const handleDeleteFromMenu = async () => {
  const item = sidebarContextMenu.value.item;
  if (!item) return;

  // 检查是否是最后一个分类
  if (allCategories.value.length <= 1) {
    ElMessage.warning("必须保留至少一个分类");
    hideContextMenu();
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定要删除分类 "${item.name}" 吗？该分类下的所有文件也将被移除。`,
      "提示",
      {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning",
      }
    );

    await deleteCategory(item.id);
    ElMessage.success("删除成功");
  } catch (error) {
    if (error !== "cancel") {
      console.error("Failed to delete category:", error);
      ElMessage.error("删除失败");
    }
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
  dragTargetCategory.value = null;
});
</script>

<style scoped>
/* 侧边栏样式 */
.sidebar-aside {
  width: var(--app-sidebar-width, 150px);
  background-color: var(--app-sidebar-bg, #ffffff);
  display: flex;
  flex-direction: column;
  height: 100%;
  user-select: none;
  border-right: 1px solid var(--app-border, #e8eaed);
}

.sidebar-menu {
  flex: 1;
  overflow-y: auto;
  position: relative;
  user-select: none;
}

.special-categories {
  padding: 8px 0;
  border-bottom: 1px solid var(--app-border-light, #f0f0f0);
}

.menu-item.special-item {
  height: 36px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  cursor: pointer;
  transition: all var(--app-transition, 0.2s);
  color: var(--app-text-secondary, #606266);
  font-size: 13px;
  font-weight: 500;
  position: relative;
}

.menu-item.special-item:hover {
  background-color: var(--app-hover, #f5f7fa);
  color: var(--app-text-primary, #1a1a2e);
}

.menu-item.special-item.active {
  background-color: var(--app-active-bg, #ecf5ff);
  color: var(--app-active-border, #409eff);
  font-weight: 600;
}

.menu-item.special-item.active::after {
  content: '';
  position: absolute;
  right: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  background-color: var(--app-active-border, #409eff);
  border-radius: 3px 0 0 3px;
}

.menu-item.is-refreshing {
  color: #f56c6c !important;
}

.menu-item.special-item.is-dragover {
  background-color: var(--app-active-bg, #ecf5ff);
  box-shadow: inset 0 0 0 1.5px var(--app-active-border, #409eff);
  border-radius: var(--app-radius-sm, 6px);
}

.menu-item.special-item:active {
  cursor: pointer;
}

.item-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-icon {
  font-size: 16px;
}

.menu-items {
  padding: 8px 0;
  margin: 0;
  list-style: none;
  position: relative;
  z-index: 1;
  user-select: none;
}

.sidebar-item-container {
  position: relative;
  z-index: 1;
  user-select: none;
}

.menu-item {
  position: relative;
  z-index: 1;
  cursor: grab;
  user-select: none;
}

.menu-item:active {
  cursor: grabbing;
}

.item-name {
  user-select: none;
}

.sortable-ghost {
  opacity: 0.3;
  background-color: var(--app-active-bg, #ecf5ff) !important;
  border: 1.5px dashed var(--app-active-border, #409eff);
  border-radius: var(--app-radius-sm, 6px);
  position: relative;
  z-index: 10;
}

.sortable-drag {
  opacity: 0.9;
  background-color: white !important;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  border-radius: var(--app-radius-sm, 6px);
  position: relative;
  z-index: 100;
}

.sidebar-footer {
  padding: 10px 0;
  border-top: 1px solid var(--app-border-light, #f0f0f0);
}

.footer-item {
  height: 36px;
  padding: 0 16px 0 32px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--app-text-secondary, #606266);
  cursor: pointer;
  transition: all var(--app-transition, 0.2s);
}

.footer-item:hover {
  background-color: var(--app-hover, #f5f7fa);
  color: var(--app-active-border, #409eff);
}

.footer-item .el-icon {
  font-size: 16px;
}

/* 分组样式 */
.add-category-input {
  padding: 4px 16px 4px 32px;
}

/* 右键菜单样式 */
.sidebar-context-menu {
  position: fixed;
  z-index: 1000;
  background-color: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border, #e8eaed);
  border-radius: var(--app-radius-md, 8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 4px rgba(0, 0, 0, 0.04);
  min-width: 140px;
  padding: 4px;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  padding: 7px 12px;
  cursor: pointer;
  transition: all var(--app-transition, 0.2s);
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--app-text-primary, #1a1a2e);
  border-radius: var(--app-radius-sm, 6px);
}

.context-menu-item:hover {
  background-color: var(--app-hover, #f5f7fa);
  color: var(--app-active-border, #409eff);
}

.context-menu-item.delete:hover {
  color: #f56c6c;
  background-color: #fef0f0;
}

/* 滚动条样式 */
.sidebar-menu::-webkit-scrollbar {
  width: 4px;
}

.sidebar-menu::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

.sidebar-menu::-webkit-scrollbar-thumb:hover {
  background-color: rgba(0, 0, 0, 0.15);
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
    display: none;
  }
}
</style>
