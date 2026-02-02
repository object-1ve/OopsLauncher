<template>
  <!-- 右键菜单 -->
  <div v-if="visible" class="context-menu" :style="menuStyle" ref="menuRef">
    <ul class="context-menu-list">
      <!-- 文件操作项 -->
      <template v-if="selectedFileId">
        <li @click="handleOpenLocation" class="context-menu-item">
          打开文件所在位置
        </li>
        <li @click="handleEditInfo" class="context-menu-item">
          编辑信息
        </li>
        <li @click="handleDelete" class="context-menu-item">
          删除
        </li>
        <li class="context-menu-divider"></li>
      </template>

      <!-- 排序方式 (始终显示) -->
      <li class="context-menu-item submenu-parent">
        <div class="menu-item-content">
          排序方式
          <el-icon class="arrow-icon"><ArrowRight /></el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleSort('name')" class="context-menu-item" :class="{ active: sortMethod === 'name' }">
            <span>按名称排序</span>
            <el-icon v-if="sortMethod === 'name'" class="order-icon">
              <Top v-if="sortOrder === 'asc'" />
              <Bottom v-else />
            </el-icon>
          </li>
          <li @click="handleSort('openCount')" class="context-menu-item" :class="{ active: sortMethod === 'openCount' }">
            <span>按打开次数排序</span>
            <el-icon v-if="sortMethod === 'openCount'" class="order-icon">
              <Top v-if="sortOrder === 'asc'" />
              <Bottom v-else />
            </el-icon>
          </li>
          <li @click="handleSort('created_at')" class="context-menu-item" :class="{ active: sortMethod === 'created_at' }">
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
          <el-icon class="arrow-icon"><ArrowRight /></el-icon>
        </div>
        <ul class="context-submenu">
          <li @click="handleToggleDisplay('showFileName')" class="context-menu-item">
            <el-icon class="check-icon"><Check v-if="showFileName" /></el-icon>
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
  showFileName: {
    type: Boolean,
    default: true
  }
});

// Emits
const emit = defineEmits(["delete", "hide", "openLocation", "editInfo", "sort", "toggleDisplay"]);

// 菜单元素引用
const menuRef = ref(null);

// 菜单尺寸
const menuSize = ref({ width: 200, height: 150 });

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

// 方法：处理打开文件所在位置
const handleOpenLocation = () => {
  if (props.selectedFile) {
    emit("openLocation", props.selectedFile);
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

// 方法：处理排序
const handleSort = (method) => {
  emit("sort", method);
  emit("hide");
};

// 方法：处理显示切换
const handleToggleDisplay = (key) => {
  emit("toggleDisplay", key);
  emit("hide");
};
</script>

<style scoped>
/* 右键菜单样式 */
.context-menu {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 1000;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  min-width: 120px;
  padding: 5px 0;
  transition: all 0.2s ease;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  padding: 4px 12px; /* 减小内边距 */
  cursor: pointer;
  transition: background-color 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px; /* 稍微减小字号 */
}

.context-menu-item:hover {
  background-color: #f5f5f5;
}

.context-menu-item.active {
  color: #409eff;
  background-color: #ecf5ff;
}

.context-menu-label {
  padding: 4px 12px;
  font-size: 11px;
  color: #999;
  font-weight: bold;
  border-bottom: 1px solid #f0f0f0;
  margin-bottom: 4px;
}

.context-menu-divider {
  height: 1px;
  background-color: #f0f0f0;
  margin: 4px 0;
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
  color: #999;
}

.order-icon {
  margin-left: auto;
  font-size: 14px;
}

.check-icon {
  width: 14px;
  margin-right: 8px;
}

.context-submenu {
  position: absolute;
  left: 100%;
  top: -5px;
  display: none;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  min-width: 140px;
  padding: 5px 0;
  list-style: none;
}

.submenu-parent:hover > .context-submenu {
  display: block;
}
</style>
