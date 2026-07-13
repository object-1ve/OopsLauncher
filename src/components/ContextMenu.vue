<template>
  <!-- 右键菜单 -->
  <div v-if="visible" class="context-menu" :style="menuStyle" ref="menuRef">
    <ul class="context-menu-list">
      <!-- 开始菜单特殊操作 -->
      <template v-if="showRefreshStartMenu">
        <li @click="handleRefreshStartMenu" class="context-menu-item">
          刷新程序列表
        </li>
        <li class="context-menu-divider"></li>
      </template>

      <!-- 文件操作项 -->
      <template v-if="selectedFileId">
        <li @click="handleOpenLocation" class="context-menu-item">
          窗口打开
        </li>
        <li @click="handleOpenInExplorer" class="context-menu-item">
          跳转目录
        </li>
        <li @click="handleOpenTerminal" class="context-menu-item">
          在终端打开
        </li>
        <li @click="handleCopyPath" class="context-menu-item">
          复制路径
        </li>
        <li @click="handleOpenWith" class="context-menu-item">
          打开方式
        </li>
        <li @click="handleEditInfo" class="context-menu-item">
          编辑信息
        </li>
        <li v-if="showPinToggle" @click="handleTogglePin" class="context-menu-item">
          {{ selectedFileIsPinned ? '取消置顶' : '置顶' }}
        </li>
        <li @click="handleDelete" class="context-menu-item">
          删除
        </li>
        <li v-if="showLocateToCategory || targetCategories.length > 0" class="context-menu-divider"></li>
        <li v-if="showLocateToCategory" @click="handleLocateToCategory" class="context-menu-item">
          定位到对应分类
        </li>
        <li v-if="showLocateToCategory && targetCategories.length > 0" class="context-menu-divider"></li>
        <!-- 复制到其他分类 -->
        <li v-if="targetCategories.length > 0" class="context-menu-item submenu-parent">
          <div class="menu-item-content">
            复制到
            <el-icon class="arrow-icon">
              <ArrowRight />
            </el-icon>
          </div>
          <ul class="context-submenu" style="max-height: 200px; overflow-y: auto;">
            <li v-for="cat in targetCategories" :key="cat.id" @click="handleCopyToCategory(cat.id)"
              class="context-menu-item">
              {{ cat.name }}
            </li>
          </ul>
        </li>

        <li v-if="showLocateToCategory || targetCategories.length > 0" class="context-menu-divider"></li>

      </template>
      <!-- 新建 (二级菜单) -->
      <li class="context-menu-item submenu-parent">
        <div class="menu-item-content">
          新建
          <el-icon class="arrow-icon">
            <ArrowRight />
          </el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleNewFolder" class="context-menu-item">
            <el-icon class="menu-icon"><FolderAdd /></el-icon>
            <span>新建文件夹</span>
          </li>
          <li @click="handleNewFile" class="context-menu-item">
            <el-icon class="menu-icon"><DocumentAdd /></el-icon>
            <span>新建文件</span>
          </li>
        </ul>
      </li>

      <!-- 分类方式 -->
      <li class="context-menu-item submenu-parent">
        <div class="menu-item-content">
          分类方式
          <el-icon class="arrow-icon">
            <ArrowRight />
          </el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleClassify('none')" class="context-menu-item" :class="{ active: classifyMethod === 'none' }">
            <span>不分类</span>
          </li>
          <li @click="handleClassify('type')" class="context-menu-item" :class="{ active: classifyMethod === 'type' }">
            <span>按文件类型分类</span>
          </li>
          <li @click="handleClassify('letter')" class="context-menu-item"
            :class="{ active: classifyMethod === 'letter' }">
            <span>按首字母分类</span>
          </li>
        </ul>
      </li>
      <!-- 排序方式 (始终显示) -->
      <li class="context-menu-item submenu-parent">
        <div class="menu-item-content">
          排序方式
          <el-icon class="arrow-icon">
            <ArrowRight />
          </el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleSort('name')" class="context-menu-item" :class="{ active: sortMethod === 'name' }">
            <span>按名称排序</span>
            <el-icon v-if="sortMethod === 'name'" class="order-icon">
              <Top v-if="sortOrder === 'asc'" />
              <Bottom v-else />
            </el-icon>
          </li>
          <li @click="handleSort('openCount')" class="context-menu-item"
            :class="{ active: sortMethod === 'openCount' }">
            <span>按打开次数排序</span>
            <el-icon v-if="sortMethod === 'openCount'" class="order-icon">
              <Top v-if="sortOrder === 'asc'" />
              <Bottom v-else />
            </el-icon>
          </li>
          <li @click="handleSort('created_at')" class="context-menu-item"
            :class="{ active: sortMethod === 'created_at' }">
            <span>按创建时间排序</span>
            <el-icon v-if="sortMethod === 'created_at'" class="order-icon">
              <Top v-if="sortOrder === 'asc'" />
              <Bottom v-else />
            </el-icon>
          </li>
        </ul>
      </li>


      <!-- 显示设置 (始终显示) -->
      <li class="context-menu-item submenu-parent">
        <div class="menu-item-content">
          显示
          <el-icon class="arrow-icon">
            <ArrowRight />
          </el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleToggleDisplay('showFileName')" class="context-menu-item">
            <el-icon class="check-icon">
              <Check v-if="showFileName" />
            </el-icon>
            <span>文件名称</span>
          </li>
        </ul>
      </li>

    </ul>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, watch } from "vue";
import {
  ArrowRight,
  Bottom,
  Top,
  Check,
  FolderAdd,
  DocumentAdd,
} from "@element-plus/icons-vue";

// Props
const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  x: {
    type: Number,
    default: 0,
  },
  y: {
    type: Number,
    default: 0,
  },
  selectedFileId: {
    type: String,
    default: null,
  },
  selectedFile: {
    type: Object,
    default: null,
  },
  sortMethod: {
    type: String,
    default: "name",
  },
  sortOrder: {
    type: String,
    default: 'asc'
  },
  classifyMethod: {
    type: String,
    default: 'none'
  },
  showFileName: {
    type: Boolean,
    default: true
  },
  categories: {
    type: Array,
    default: () => []
  },
  showLocateToCategory: {
    type: Boolean,
    default: false
  },
  showPinToggle: {
    type: Boolean,
    default: false
  },
  selectedFileIsPinned: {
    type: Boolean,
    default: false
  },
  showRefreshStartMenu: {
    type: Boolean,
    default: false
  }
});

// Emits
const emit = defineEmits(["delete", "hide", "openLocation", "openInExplorer", "openTerminal", "copyPath", "openWith", "editInfo", "sort", "classify", "toggleDisplay", "copyToCategory", "locateToCategory", "togglePin", "refreshStartMenu", "newFolder", "newFile"]);

// 菜单元素引用
const menuRef = ref(null);

// 菜单尺寸
const menuSize = ref({ width: 160, height: 150 });

// 计算菜单位置
const menuStyle = computed(() => {
  if (!props.visible) return {};

  let { x, y } = props;
  const { width, height } = menuSize.value;

  // 获取视窗尺寸
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  // 调整水平位置，确保菜单不会超出视窗
  if (x + width > viewportWidth) {
    x = Math.max(0, x - width);
  }

  // 调整垂直位置，确保菜单不会超出视窗
  if (y + height > viewportHeight) {
    y = Math.max(0, y - height);
  }

  return {
    left: x + "px",
    top: y + "px",
  };
});

// 监听可见性变化，更新菜单尺寸
watch(
  () => props.visible,
  async (newVisible) => {
    if (newVisible) {
      await nextTick();
      updateMenuSize();
    }
  }
);

// 更新菜单尺寸
const updateMenuSize = () => {
  if (menuRef.value) {
    const rect = menuRef.value.getBoundingClientRect();
    menuSize.value = {
      width: rect.width,
      height: rect.height,
    };
  }
};

// 方法：处理删除
const handleDelete = () => {
  if (props.selectedFileId) {
    emit("delete", props.selectedFileId);
    emit("hide");
  }
};

// 方法：处理窗口打开
const handleOpenLocation = () => {
  if (props.selectedFile) {
    emit("openLocation", props.selectedFile);
    emit("hide");
  }
};

const handleOpenInExplorer = () => {
  if (props.selectedFile) {
    emit("openInExplorer", props.selectedFile);
    emit("hide");
  }
};

const handleLocateToCategory = () => {
  if (props.selectedFile) {
    emit("locateToCategory", props.selectedFile);
    emit("hide");
  }
};

const handleOpenTerminal = () => {
  if (props.selectedFile) {
    emit("openTerminal", props.selectedFile);
    emit("hide");
  }
};

// 方法：处理复制路径
const handleCopyPath = () => {
  if (props.selectedFile) {
    emit("copyPath", props.selectedFile);
    emit("hide");
  }
};

const handleOpenWith = () => {
  if (props.selectedFile) {
    emit("openWith", props.selectedFile);
    emit("hide");
  }
};

// 方法：处理编辑信息
const handleEditInfo = () => {
  if (props.selectedFile) {
    emit("editInfo", props.selectedFile);
    emit("hide");
  }
};

const handleTogglePin = () => {
  if (props.selectedFile) {
    emit("togglePin", props.selectedFile);
    emit("hide");
  }
};

const handleRefreshStartMenu = () => {
  emit("refreshStartMenu");
  emit("hide");
};

const handleNewFolder = () => {
  emit("newFolder");
  emit("hide");
};

const handleNewFile = () => {
  emit("newFile");
  emit("hide");
};

// 方法：处理排序
const handleSort = (method) => {
  emit("sort", method);
  emit("hide");
};

const handleClassify = (method) => {
  emit("classify", method);
  emit("hide");
};

// 方法：处理显示切换
const handleToggleDisplay = (key) => {
  emit("toggleDisplay", key);
  emit("hide");
};

// 过滤掉文件当前所在分类，只显示其他可复制目标
const targetCategories = computed(() => {
  return props.categories.filter(c => c.id !== props.selectedFile?.category)
})

// 方法：处理复制到分类
const handleCopyToCategory = (targetCategoryId) => {
  if (props.selectedFile) {
    emit('copyToCategory', { file: props.selectedFile, targetCategoryId })
    emit('hide')
  }
}
</script>

<style scoped>
/* 右键菜单样式 */
.context-menu {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 1000;
  background-color: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border, #e8eaed);
  border-radius: var(--app-radius-md, 8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 4px rgba(0, 0, 0, 0.04);
  min-width: 160px;
  padding: 4px;
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  padding: 6px 12px;
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

.context-menu-item.active {
  color: var(--app-active-border, #409eff);
  background-color: var(--app-active-bg, #ecf5ff);
}

.context-menu-label {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--app-text-muted, #909399);
  font-weight: 600;
  border-bottom: 1px solid var(--app-border-light, #f0f0f0);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.context-menu-divider {
  height: 1px;
  background-color: var(--app-border-light, #f0f0f0);
  margin: 4px 8px;
}

.submenu-parent {
  position: relative;
}

.menu-item-content {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 8px;
}

.arrow-icon {
  margin-left: auto;
  font-size: 12px;
  color: var(--app-text-muted, #909399);
}

.order-icon {
  margin-left: auto;
  font-size: 14px;
}

.check-icon {
  width: 14px;
  margin-right: 8px;
}

.menu-icon {
  font-size: 14px;
  color: var(--app-text-muted, #909399);
}

.context-submenu {
  position: absolute;
  left: 100%;
  top: -4px;
  display: none;
  background-color: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border, #e8eaed);
  border-radius: var(--app-radius-md, 8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 4px rgba(0, 0, 0, 0.04);
  min-width: 140px;
  padding: 4px;
  list-style: none;
}

.submenu-parent:hover>.context-submenu {
  display: block;
}
</style>
