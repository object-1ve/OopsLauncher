<template>
  <div class="file-explorer">
    <div class="explorer-header">
      <div class="path-nav">
        <el-button-group size="small">
          <el-button :disabled="pathHistory.length <= 1" @click="goBack">
            <el-icon>
              <ArrowLeft />
            </el-icon>
          </el-button>
          <el-button @click="goHome">
            <el-icon>
              <HomeFilled />
            </el-icon>
          </el-button>
        </el-button-group>

        <el-button-group size="small">
          <el-button @click="handleOpenInTerminal" title="在终端中打开">
            <svg viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="3" width="20" height="18" rx="2" />
              <polyline points="7 8 11 12 7 16" />
              <line x1="13" y1="16" x2="17" y2="16" />
            </svg>
          </el-button>
          <el-button @click="handleOpenInExplorer" title="在资源管理器中打开">
            <el-icon>
              <FolderOpened />
            </el-icon>
          </el-button>
        </el-button-group>

        <el-button
          size="small"
          :type="isCurrentPathFavorite ? 'warning' : 'default'"
          @click="toggleFavoriteCurrentPath"
          :title="isCurrentPathFavorite ? '取消收藏' : '收藏当前目录'"
          class="favorite-toggle-btn"
        >
          <el-icon>
            <StarFilled v-if="isCurrentPathFavorite" />
            <Star v-else />
          </el-icon>
        </el-button>

        <div class="path-display" @dblclick="startEditingPath">
          <button class="path-expand-btn" @click.stop="toggleFavoritesRow" :title="favoritesExpanded ? '收起收藏' : '展开收藏'">
            <el-icon>
              <CaretBottom v-if="!favoritesExpanded" />
              <CaretTop v-else />
            </el-icon>
          </button>
          <template v-if="!isEditingPath">
            <el-breadcrumb separator="/">
              <el-breadcrumb-item @click.stop="goHome">此电脑</el-breadcrumb-item>
              <el-breadcrumb-item v-for="(part, index) in pathParts" :key="index" @click.stop="navigateToPart(index)">
                {{ part }}
              </el-breadcrumb-item>
            </el-breadcrumb>
          </template>
          <template v-else>
            <el-input v-model="editingPath" size="small" ref="pathInputRef" @keyup.enter="handlePathJump"
              @blur="cancelEditingPath" @keyup.esc="cancelEditingPath" />
          </template>
        </div>
      </div>

      <!-- 收藏展开行 -->
      <div v-if="favoritesExpanded" ref="favoritesBarRef" class="favorites-bar">
        <div v-if="favorites.length === 0" class="favorites-bar-empty">暂无收藏的路径</div>
        <div
          v-for="fav in favorites"
          :key="fav.id"
          class="favorites-bar-item"
          @click="jumpToFavorite(fav)"
          :title="fav.path"
        >
          <el-icon class="fav-star-icon"><StarFilled /></el-icon>
          <span class="fav-bar-name">{{ fav.name }}</span>
          <span class="fav-bar-path">{{ fav.path }}</span>
          <span class="fav-bar-count" v-if="fav.openCount">×{{ fav.openCount }}</span>
          <el-button
            text
            size="small"
            class="fav-remove-btn"
            @click.stop="handleRemoveFavorite(fav)"
            title="移除收藏"
          >
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
      </div>
    </div>

    <div class="explorer-content" ref="explorerContentRef" v-loading="loading" @scroll="handleScroll"
      @contextmenu.prevent="handleEmptyAreaContextMenu">
      <div v-if="files.length === 0 && !loading && !creatingItem.active" class="empty-state">
        <el-empty description="此文件夹为空" />
      </div>

      <!-- 网格视图 -->
      <div v-if="viewMode === 'grid' && files.length > 0" class="file-grid-viewport" :style="{ height: virtualGrid.totalHeight + 'px' }">
        <div class="file-grid" :style="{ transform: `translateY(${virtualGrid.startRow * GRID_ITEM_HEIGHT}px)` }">
          <el-tooltip v-for="file in virtualGrid.items" :key="file.path" effect="light" placement="right"
            :show-after="500">
            <template #content>
              <div class="file-tooltip">
                <p><strong>名称:</strong> {{ file.name }}</p>
                <p><strong>类型:</strong> {{ file.type === 'directory' ? (file.isReparsePoint ? '系统联接点/符号链接' : '文件夹') :
                  file.type }}</p>
                <p v-if="file.isReparsePoint" class="special-folder-tip">这是一个 Windows 系统联接点，通常用于兼容性跳转，点击可能打开其他系统路径。</p>
                <p v-if="file.type !== 'directory' || (file.size && file.size > 0) || file.calculating">
                  <strong>大小:</strong>
                  <template v-if="file.calculating">
                    <el-icon class="is-loading" style="vertical-align: middle;">
                      <Loading />
                    </el-icon>
                  </template>
                  <template v-else>
                    {{ formatSize(file.size) }}
                  </template>
                </p>
                <p><strong>路径:</strong> {{ file.path }}</p>
              </div>
            </template>
            <div class="file-item" :class="{
              'is-selected': selectedFile?.path === file.path,
              'is-reparse-point': file.isReparsePoint,
              'is-highlighted': explorerHighlightPath === file.path
            }" draggable="true" @dragstart="handleFileDragStart($event, file)" @click.stop="selectedFile = file; hideContextMenu()" @dblclick.stop="handleItemClick(file)"
              @contextmenu.stop.prevent="handleContextMenu($event, file)">
              <div class="file-icon-wrapper">
                <template v-if="file.icon">
                  <img :src="file.icon" class="file-icon" />
                </template>
                <template v-else>
                  <span class="file-icon-placeholder">{{ file.type === 'directory' ? '📁' : '📄' }}</span>
                </template>
                <div v-if="file.isReparsePoint" class="reparse-point-badge" title="系统联接点">🔗</div>
              </div>
              <div class="file-info">
                <span class="file-name" :title="file.name">{{ file.name }}</span>
              </div>
            </div>
          </el-tooltip>
        </div>
      </div>

      <!-- 列表视图 -->
      <div v-else-if="viewMode === 'list' && files.length > 0" class="file-list-viewport"
        :style="{ height: virtualList.totalHeight + LIST_ITEM_HEIGHT + 'px' }">
        <div class="file-list-header">
          <div class="col-name sortable" @click="handleSort('name')">
            名称
            <el-icon v-if="sortKey === 'name'" class="sort-icon">
              <CaretTop v-if="sortOrder === 'asc'" />
              <CaretBottom v-else />
            </el-icon>
          </div>
          <div class="col-size sortable" @click="handleSort('size')">
            大小
            <el-icon v-if="sortKey === 'size'" class="sort-icon">
              <CaretTop v-if="sortOrder === 'asc'" />
              <CaretBottom v-else />
            </el-icon>
          </div>
          <div class="col-date sortable" @click="handleSort('modified_at')">
            修改时间
            <el-icon v-if="sortKey === 'modified_at'" class="sort-icon">
              <CaretTop v-if="sortOrder === 'asc'" />
              <CaretBottom v-else />
            </el-icon>
          </div>
        </div>
        <div class="file-list-content" :style="{ transform: `translateY(${virtualList.offsetY}px)` }">
          <div v-for="file in virtualList.items" :key="file.path" class="file-list-item" :class="{
            'is-selected': selectedFile?.path === file.path,
            'is-reparse-point': file.isReparsePoint,
            'is-highlighted': explorerHighlightPath === file.path
          }" draggable="true" @dragstart="handleFileDragStart($event, file)" @click.stop="selectedFile = file; hideContextMenu()" @dblclick.stop="handleItemClick(file)"
            @contextmenu.stop.prevent="handleContextMenu($event, file)">
            <div class="col-name">
              <template v-if="file.icon">
                <img :src="file.icon" class="file-list-icon" />
              </template>
              <template v-else>
                <span class="file-list-icon-placeholder">{{ file.type === 'directory' ? '📁' : '📄' }}</span>
              </template>
              <span class="file-name" :title="file.name">
                {{ file.name }}
                <el-tag v-if="file.isReparsePoint" size="small" type="info" class="reparse-tag">联接点</el-tag>
              </span>
            </div>
            <div class="col-size">
              <template v-if="file.calculating">
                <el-icon class="is-loading">
                  <Loading />
                </el-icon>
              </template>
              <template v-else-if="file.type === 'directory' && !file.dirSizeCalculated">
                -
              </template>
              <template v-else>
                {{ formatSize(file.size) }}
              </template>
            </div>
            <div class="col-date">{{ formatDate(file.modifiedAt || file.createdAt) }}</div>
          </div>
        </div>
      </div>

      <!-- 新建文件/文件夹内联输入 -->
      <div v-if="creatingItem.active" class="creating-item-wrapper">
        <div class="creating-item-icon">{{ creatingItem.type === 'folder' ? '📁' : '📄' }}</div>
        <input
          ref="creatingInputRef"
          v-model="creatingItem.name"
          class="creating-item-text-input"
          @keyup.enter="confirmCreate"
          @keyup.escape="cancelCreate"
          @blur="confirmCreate"
        />
      </div>
    </div>

    <!-- 资源管理器右键菜单 -->
    <div v-if="explorerMenu.visible" ref="contextMenuRef" class="explorer-context-menu" :style="{
      left: explorerMenu.x + 'px',
      top: explorerMenu.y + 'px',
    }" @click.stop>
      <ul class="context-menu-list">
        <template v-if="explorerMenu.file">
          <li class="context-menu-item" @click="handleOpen(explorerMenu.file)">
            打开
          </li>
          <li class="context-menu-item" @click="handleOpenFileLocation(explorerMenu.file)">
            窗口打开
          </li>
          <li class="context-menu-item" @click="handleOpenWith(explorerMenu.file)">
            打开方式
          </li>
          <template v-if="explorerMenu.file.type === 'directory'">
            <li class="context-menu-divider"></li>
            <li class="context-menu-item has-submenu">
              <div class="menu-item-content">
                新建
                <el-icon class="arrow-icon">
                  <ArrowRight />
                </el-icon>
              </div>
              <ul class="submenu" :class="{ 'submenu-left': isMenuOnRight }">
                <li class="context-menu-item" @click="handleStartCreate('file', explorerMenu.file.path)">
                  新建文件
                </li>
                <li class="context-menu-item" @click="handleStartCreate('folder', explorerMenu.file.path)">
                  新建文件夹
                </li>
              </ul>
            </li>
          </template>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item has-submenu">
            <div class="menu-item-content">
              添加到分类
              <el-icon class="arrow-icon">
                <ArrowRight />
              </el-icon>
            </div>
            <ul class="submenu" :class="{ 'submenu-left': isMenuOnRight }">
              <li v-for="cat in allCategories" :key="cat.id" class="context-menu-item"
                @click="handleAddToCategory(explorerMenu.file, cat.id)">
                {{ cat.name }}
              </li>
              <li v-if="allCategories.length === 0" class="context-menu-item disabled">
                暂无分类
              </li>
            </ul>
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item" @click="handleCopy(explorerMenu.file)">
            复制
          </li>
          <li class="context-menu-item" @click="handleCut(explorerMenu.file)">
            剪切
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item delete" @click="handleDelete(explorerMenu.file)">
            删除
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item" @click="handleCalculateFolderSizes()">
            计算全部文件大小
          </li>
        </template>
        <template v-else>
          <li class="context-menu-item has-submenu">
            <div class="menu-item-content">
              新建
              <el-icon class="arrow-icon">
                <ArrowRight />
              </el-icon>
            </div>
            <ul class="submenu" :class="{ 'submenu-left': isMenuOnRight }">
              <li class="context-menu-item" @click="handleStartCreate('file')">
                新建文件
              </li>
              <li class="context-menu-item" @click="handleStartCreate('folder')">
                新建文件夹
              </li>
            </ul>
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item has-submenu">
            <div class="menu-item-content">
              展示方式
              <el-icon class="arrow-icon">
                <ArrowRight />
              </el-icon>
            </div>
            <ul class="submenu" :class="{ 'submenu-left': isMenuOnRight }">
              <li class="context-menu-item" :class="{ active: viewMode === 'grid' }" @click="viewMode = 'grid'">
                大图标
              </li>
              <li class="context-menu-item" :class="{ active: viewMode === 'list' }" @click="viewMode = 'list'">
                列表
              </li>
            </ul>
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item" :class="{ disabled: !clipboard.path }" @click="handlePaste">
            粘贴
          </li>
          <li class="context-menu-item" @click="loadDirectory(currentPath)">
            刷新
          </li>
          <li class="context-menu-divider"></li>
          <li class="context-menu-item" @click="handleCalculateFolderSizes()">
            计算全部文件大小
          </li>
        </template>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { ArrowLeft, ArrowRight, HomeFilled, Loading, CaretTop, CaretBottom, FolderOpened, Star, StarFilled, Delete } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useFiles } from '@/composables/useFiles';
import { startFolderSizeTask, getFileIcon, openFileLocation, addFavorite, removeFavorite, getFavorites, isFavorite, incrementOpenCount } from '@/api/file';
import { ElMessage } from 'element-plus';

const { explorerPath, explorerHighlightPath, allCategories, processFiles } = useFiles();

const files = ref([]);
const loading = ref(false);
const pathHistory = ref(['']); // '' represents 'This PC'

// --- 排序相关状态 ---
const sortKey = ref('name'); // 'name', 'size', 'modified_at'
const sortOrder = ref('asc'); // 'asc' or 'desc'

const sortedFiles = computed(() => {
  const result = [...files.value];
  result.sort((a, b) => {
    // 目录始终排在最前面
    const aIsDir = a.type === 'directory';
    const bIsDir = b.type === 'directory';
    if (aIsDir !== bIsDir) {
      return bIsDir ? 1 : -1;
    }

    let comparison = 0;
    if (sortKey.value === 'name') {
      comparison = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
    } else if (sortKey.value === 'size') {
      comparison = (a.size || 0) - (b.size || 0);
    } else if (sortKey.value === 'modified_at') {
      comparison = (a.modifiedAt || 0) - (b.modifiedAt || 0);
    }

    return sortOrder.value === 'asc' ? comparison : -comparison;
  });
  return result;
});

const handleSort = (key) => {
  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
  } else {
    sortKey.value = key;
    sortOrder.value = 'desc';
  }
};

// --- 虚拟滚动相关状态 ---
const explorerContentRef = ref(null);
const scrollTop = ref(0);
const containerHeight = ref(0);
const iconCache = ref(new Map()); // 图标缓存

// 常量配置
const LIST_ITEM_HEIGHT = 40;
const GRID_ITEM_HEIGHT = 110;
const GRID_ITEM_WIDTH = 115; // 包括 gap
const BUFFER_COUNT = 5; // 上下多渲染的数量

const isEditingPath = ref(false);
const editingPath = ref('');
const pathInputRef = ref(null);
const contextMenuRef = ref(null);

const explorerMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  file: null,
});

const viewMode = ref('list'); // 'grid' or 'list'

const handleScroll = (e) => {
  scrollTop.value = e.target.scrollTop;
};

const updateContainerHeight = () => {
  if (explorerContentRef.value) {
    containerHeight.value = explorerContentRef.value.clientHeight;
  }
};

// 计算列表模式下的可见文件
const virtualList = computed(() => {
  if (viewMode.value !== 'list') return { items: [], totalHeight: 0, offsetY: 0 };

  const totalHeight = sortedFiles.value.length * LIST_ITEM_HEIGHT;
  const startIndex = Math.max(0, Math.floor(scrollTop.value / LIST_ITEM_HEIGHT) - BUFFER_COUNT);
  const endIndex = Math.min(sortedFiles.value.length, Math.ceil((scrollTop.value + containerHeight.value) / LIST_ITEM_HEIGHT) + BUFFER_COUNT);

  const visibleItems = sortedFiles.value.slice(startIndex, endIndex).map((file, idx) => ({
    ...file,
    _index: startIndex + idx,
    _top: (startIndex + idx) * LIST_ITEM_HEIGHT
  }));

  return { items: visibleItems, totalHeight, offsetY: startIndex * LIST_ITEM_HEIGHT };
});

// 计算网格模式下的可见文件
const virtualGrid = computed(() => {
  if (viewMode.value !== 'grid' || !explorerContentRef.value) return { items: [], totalHeight: 0, offsetY: 0 };

  const contentWidth = explorerContentRef.value.clientWidth - 40; // 减去 padding
  const itemsPerRow = Math.max(1, Math.floor(contentWidth / GRID_ITEM_WIDTH));
  const rowCount = Math.ceil(sortedFiles.value.length / itemsPerRow);
  const totalHeight = rowCount * GRID_ITEM_HEIGHT;

  const startRow = Math.max(0, Math.floor(scrollTop.value / GRID_ITEM_HEIGHT) - 1);
  const endRow = Math.min(rowCount, Math.ceil((scrollTop.value + containerHeight.value) / GRID_ITEM_HEIGHT) + 1);

  const startIndex = startRow * itemsPerRow;
  const endIndex = Math.min(sortedFiles.value.length, endRow * itemsPerRow);

  const visibleItems = sortedFiles.value.slice(startIndex, endIndex).map((file, idx) => ({
    ...file,
    _index: startIndex + idx,
    _row: Math.floor((startIndex + idx) / itemsPerRow),
    _col: (startIndex + idx) % itemsPerRow
  }));

  return { items: visibleItems, totalHeight, startRow };
});

// 图标延迟加载逻辑
const loadVisibleIcons = async (visibleItems) => {
  for (const file of visibleItems) {
    if (!file.icon && !iconCache.value.has(file.path)) {
      // 标记为正在加载中，防止重复请求
      iconCache.value.set(file.path, 'loading');
      try {
        const icon = await getFileIcon(file.path);
        iconCache.value.set(file.path, icon);
        // 更新文件对象的图标
        const targetFile = files.value.find(f => f.path === file.path);
        if (targetFile) {
          targetFile.icon = icon;
        }
      } catch (e) {
        console.error('Failed to load icon:', file.path, e);
        iconCache.value.set(file.path, ''); // 失败则空
      }
    } else if (iconCache.value.has(file.path)) {
      const cachedIcon = iconCache.value.get(file.path);
      if (cachedIcon !== 'loading' && !file.icon) {
        file.icon = cachedIcon;
      }
    }
  }
};

// 监听可见项变化，加载图标
watch([virtualList, virtualGrid], () => {
  const visibleItems = viewMode.value === 'list' ? virtualList.value.items : virtualGrid.value.items;
  if (visibleItems.length > 0) {
    loadVisibleIcons(visibleItems);
  }
}, { immediate: true });

// 监听高亮路径，执行滚动
watch([sortedFiles, explorerHighlightPath], () => {
  if (explorerHighlightPath.value && sortedFiles.value.length > 0) {
    const index = sortedFiles.value.findIndex(f => f.path === explorerHighlightPath.value);
    if (index !== -1) {
      nextTick(() => {
        if (!explorerContentRef.value) return;

        let targetScrollTop = 0;
        if (viewMode.value === 'list') {
          targetScrollTop = Math.max(0, index * LIST_ITEM_HEIGHT - containerHeight.value / 2 + LIST_ITEM_HEIGHT / 2);
        } else {
          const contentWidth = explorerContentRef.value.clientWidth - 40;
          const itemsPerRow = Math.max(1, Math.floor(contentWidth / GRID_ITEM_WIDTH));
          const row = Math.floor(index / itemsPerRow);
          targetScrollTop = Math.max(0, row * GRID_ITEM_HEIGHT - containerHeight.value / 2 + GRID_ITEM_HEIGHT / 2);
        }

        explorerContentRef.value.scrollTop = targetScrollTop;
        scrollTop.value = targetScrollTop;

        // 3秒后清除高亮
        if (highlightTimeout) clearTimeout(highlightTimeout);
        highlightTimeout = setTimeout(() => {
          if (explorerHighlightPath.value) {
            explorerHighlightPath.value = '';
          }
          highlightTimeout = null;
        }, 3000);
      });
    }
  }
}, { immediate: true });

const formatSize = (bytes) => {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatDate = (timestamp) => {
  if (!timestamp) return '-';
  const date = new Date(timestamp);
  const pad2 = (value) => String(value).padStart(2, '0');
  const year = date.getFullYear();
  const month = pad2(date.getMonth() + 1);
  const day = pad2(date.getDate());
  const hours = pad2(date.getHours());
  const minutes = pad2(date.getMinutes());
  const seconds = pad2(date.getSeconds());
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};

const clipboard = ref({
  path: '',
  isCut: false,
});

const selectedFile = ref(null);
const activeFolderSizeTaskId = ref('');
let highlightTimeout = null;

// --- 新建文件/文件夹状态 ---
const creatingItem = ref({ active: false, type: '', name: '', parentDir: '' });
const creatingInputRef = ref(null);

const isMenuOnRight = computed(() => {
  return explorerMenu.value.x > window.innerWidth / 2;
});

let removeFolderSizeItemListener = null;
let removeFolderSizeCompleteListener = null;

const currentPath = computed({
  get: () => explorerPath.value || '',
  set: (val) => {
    explorerPath.value = val;
  }
});

const pathParts = computed(() => {
  if (!currentPath.value) return [];
  return currentPath.value.split(/[\\/]/).filter(p => p);
});

const loadDirectory = async (path, options = {}) => {
  const { resetScroll = true } = options;
  loading.value = true;
  // 切换路径时清空图标缓存和滚动位置
  iconCache.value.clear();
  if (resetScroll && explorerContentRef.value) {
    explorerContentRef.value.scrollTop = 0;
    scrollTop.value = 0;
  }

  try {
    const result = await invoke('list_directory', { path });
    files.value = result.map((file) => ({
      ...file,
      dirSizeCalculated: file.dirSizeCalculated ?? (file.type !== 'directory'),
    }));
    if (pathHistory.value[pathHistory.value.length - 1] !== path) {
      pathHistory.value.push(path);
    }
  } catch (error) {
    console.error('Failed to list directory:', error);
    ElMessage.error(`无法读取目录: ${error}`);
    // If failed, maybe go back
    if (pathHistory.value.length > 1) {
      pathHistory.value.pop();
    }
  } finally {
    loading.value = false;
  }
};

const handleCalculateFolderSizes = async (targetFile = null) => {
  const targetFiles = targetFile ? [targetFile] : files.value.filter(f => f.type === 'directory');

  if (targetFiles.length === 0) return;

  hideContextMenu();
  targetFiles.forEach(file => {
    file.calculating = true;
  });

  try {
    activeFolderSizeTaskId.value = await startFolderSizeTask(targetFiles.map(file => file.path));
  } catch (error) {
    targetFiles.forEach(file => {
      file.calculating = false;
    });
    ElMessage.error(`启动文件夹大小计算失败: ${error}`);
  }
};

const handleFolderSizeTaskItem = (payload) => {
  if (!payload?.path) return;
  const file = files.value.find(item => item.path === payload.path);
  if (!file) return;

  file.calculating = false;

  if (payload.error) {
    console.error(`计算文件夹大小失败: ${payload.path}`, payload.error);
    return;
  }

  if (payload.size !== undefined && payload.size !== null) {
    file.size = payload.size;
    file.dirSizeCalculated = true;
  }
};

const registerFolderSizeTaskListeners = async () => {
  removeFolderSizeItemListener = await listen('folder-size-task-item', (event) => {
    handleFolderSizeTaskItem(event.payload);
  });

  removeFolderSizeCompleteListener = await listen('folder-size-task-complete', (event) => {
    const taskId = event.payload?.taskId;
    if (taskId && taskId === activeFolderSizeTaskId.value) {
      activeFolderSizeTaskId.value = '';
      ElMessage.success('文件夹大小计算完成');
    }
  });
};

const handleItemClick = (file) => {
  if (file.type === 'directory') {
    currentPath.value = file.path;
    loadDirectory(file.path);
  } else {
    handleOpen(file);
  }
};

const goBack = () => {
  if (pathHistory.value.length > 1) {
    pathHistory.value.pop(); // Remove current
    const prevPath = pathHistory.value.pop(); // Get previous
    currentPath.value = prevPath;
    loadDirectory(prevPath);
  }
};

const goHome = () => {
  currentPath.value = '';
  loadDirectory('');
};

const handleOpenInExplorer = async () => {
  if (!currentPath.value) return;
  try {
    await invoke('open_path', { path: currentPath.value });
  } catch (error) {
    console.error('Failed to open in explorer:', error);
    ElMessage.error(`打开资源管理器失败: ${error}`);
  }
};

const handleOpenInTerminal = async () => {
  if (!currentPath.value) return;
  try {
    await invoke('open_terminal', { path: currentPath.value });
  } catch (error) {
    console.error('Failed to open in terminal:', error);
    ElMessage.error(`打开终端失败: ${error}`);
  }
};

// --- 收藏功能 ---
const favorites = ref([]);
const favoritesExpanded = ref(false);
const favoritesBarRef = ref(null); // 引用收藏夹菜单元素

const isCurrentPathFavorite = computed(() => {
  return favorites.value.some(f => f.path === currentPath.value);
});

const loadFavorites = async () => {
  try {
    favorites.value = await getFavorites();
  } catch (error) {
    console.error('Failed to load favorites:', error);
  }
};

const toggleFavoriteCurrentPath = async () => {
  if (!currentPath.value) return;
  try {
    if (isCurrentPathFavorite.value) {
      await removeFavorite(currentPath.value);
    } else {
      await addFavorite(currentPath.value);
    }
    await loadFavorites();
  } catch (error) {
    console.error('Failed to toggle favorite:', error);
    ElMessage.error(`操作收藏失败: ${error}`);
  }
};

const toggleFavoritesRow = () => {
  favoritesExpanded.value = !favoritesExpanded.value;
};

const jumpToFavorite = (fav) => {
  favoritesExpanded.value = false;
  currentPath.value = fav.path;
  loadDirectory(fav.path);
  incrementOpenCount(fav.path).then(() => loadFavorites()).catch(() => {});
};


const handleRemoveFavorite = async (fav) => {
  try {
    await removeFavorite(fav.path);
    await loadFavorites();
  } catch (error) {
    console.error('Failed to remove favorite:', error);
    ElMessage.error(`移除收藏失败: ${error}`);
  }
};

const navigateToPart = (index) => {
  const parts = pathParts.value.slice(0, index + 1);
  // Reconstruct path. On Windows this might be tricky with drive letters.
  // Our backend handles absolute paths.
  let newPath = '';
  if (currentPath.value.includes(':')) {
    // Windows
    const drive = currentPath.value.split(':')[0];
    newPath = drive + ':\\' + parts.slice(1).join('\\');
    if (index === 0) newPath = drive + ':\\';
  } else {
    // Unix
    newPath = '/' + parts.join('/');
  }
  currentPath.value = newPath;
  loadDirectory(newPath);
};

const startEditingPath = () => {
  editingPath.value = currentPath.value;
  isEditingPath.value = true;
  nextTick(() => {
    pathInputRef.value?.focus();
  });
};

const cancelEditingPath = () => {
  isEditingPath.value = false;
};

const handlePathJump = async () => {
  const targetPath = editingPath.value.trim();
  if (targetPath === currentPath.value) {
    isEditingPath.value = false;
    return;
  }

  if (!targetPath || targetPath === '此电脑' || targetPath === 'This PC') {
    goHome();
    isEditingPath.value = false;
    return;
  }

  loading.value = true;
  // 切换路径时清空图标缓存和滚动位置
  iconCache.value.clear();
  if (explorerContentRef.value) {
    explorerContentRef.value.scrollTop = 0;
    scrollTop.value = 0;
  }

  try {
    // Verify path exists and is a directory
    const result = await invoke('list_directory', { path: targetPath });
    files.value = result.map((file) => ({
      ...file,
      dirSizeCalculated: file.dirSizeCalculated ?? (file.type !== 'directory'),
    }));
    currentPath.value = targetPath;
    if (pathHistory.value[pathHistory.value.length - 1] !== targetPath) {
      pathHistory.value.push(targetPath);
    }
    isEditingPath.value = false;
  } catch (error) {
    console.error('Failed to jump to path:', error);
    ElMessage.error(`路径无效或无法访问: ${error}`);
  } finally {
    loading.value = false;
  }
};

const handleOpen = (file) => {
  if (file.type === 'directory') {
    currentPath.value = file.path;
    loadDirectory(file.path);
  } else {
    invoke('open_path', { path: file.path }).catch(err => {
      ElMessage.error(`无法打开文件: ${err}`);
    });
  }
  hideContextMenu();
};

const handleOpenFileLocation = async (file) => {
  try {
    await openFileLocation(file.path);
  } catch (error) {
    console.error('Failed to open file location:', error);
    ElMessage.error(`窗口打开失败: ${error}`);
  } finally {
    hideContextMenu();
  }
};

const handleOpenWith = async (file) => {
  try {
    await invoke('open_with_dialog', { path: file.path });
  } catch (error) {
    console.error('Failed to open with dialog:', error);
    ElMessage.error(`打开方式失败: ${error}`);
  } finally {
    hideContextMenu();
  }
};

const handleAddToCategory = async (file, categoryId) => {
  try {
    const result = await processFiles([file], categoryId);
    if (result.addedCount > 0) {
      ElMessage.success(`成功添加到分类`);
    } else if (result.existingCount > 0) {
      ElMessage.warning(`该分类已存在此文件`);
    } else if (result.error === 'cannot_add_to_special_category') {
      ElMessage.warning(`不能添加到特殊分类`);
    }
  } catch (error) {
    console.error('Failed to add to category:', error);
    ElMessage.error(`添加失败: ${error}`);
  } finally {
    hideContextMenu();
  }
};

const handleFileDragStart = (event, file) => {
  if (file && file.path) {
    // 设置拖拽数据，包含文件路径
    event.dataTransfer.setData('application/x-oops-file', JSON.stringify(file));
    event.dataTransfer.effectAllowed = 'copy';
  }
};

const handleCopy = (file) => {
  clipboard.value = {
    path: file.path,
    isCut: false,
  };
  hideContextMenu();
  ElMessage.success('已复制');
};

const handleCut = (file) => {
  clipboard.value = {
    path: file.path,
    isCut: true,
  };
  hideContextMenu();
  ElMessage.success('已剪切');
};

const handlePaste = async () => {
  if (!clipboard.value.path) return;
  if (!currentPath.value) {
    ElMessage.warning('不能在“此电脑”下粘贴');
    return;
  }

  const fileName = clipboard.value.path.split(/[\\/]/).pop();
  const destPath = `${currentPath.value}\\${fileName}`;

  if (clipboard.value.path === destPath) {
    ElMessage.warning('源文件和目标文件相同');
    return;
  }

  loading.value = true;
  try {
    if (clipboard.value.isCut) {
      await invoke('move_file_or_dir', { src: clipboard.value.path, dst: destPath });
      clipboard.value = { path: '', isCut: false }; // Clear after cut
      ElMessage.success('已移动');
    } else {
      await invoke('copy_file_or_dir', { src: clipboard.value.path, dst: destPath });
      ElMessage.success('已复制');
    }
    loadDirectory(currentPath.value);
  } catch (error) {
    console.error('Failed to paste:', error);
    ElMessage.error(`粘贴失败: ${error}`);
  } finally {
    loading.value = false;
    hideContextMenu();
  }
};

const handleDelete = async (file) => {
  const preservedScrollTop = explorerContentRef.value?.scrollTop ?? scrollTop.value;
  try {
    await invoke('delete_to_trash', { path: file.path });
    ElMessage.success('已移至回收站');
    await loadDirectory(currentPath.value, { resetScroll: false });
    await nextTick();
    if (explorerContentRef.value) {
      explorerContentRef.value.scrollTop = preservedScrollTop;
      scrollTop.value = explorerContentRef.value.scrollTop;
    }
  } catch (error) {
    console.error('Failed to delete:', error);
    ElMessage.error(`删除失败: ${error}`);
  } finally {
    hideContextMenu();
  }
};

const handleStartCreate = (type, targetDir = null) => {
  const parentDir = targetDir || currentPath.value;
  if (!parentDir) {
    ElMessage.warning('请先打开一个文件夹');
    hideContextMenu();
    return;
  }

  const defaultName = type === 'folder' ? '新建文件夹' : '新建文本文档.txt';
  creatingItem.value = {
    active: true,
    type,
    name: defaultName,
    parentDir,
  };
  hideContextMenu();

  nextTick(() => {
    if (creatingInputRef.value) {
      creatingInputRef.value.focus();
      // 选中文件名部分（不含扩展名）
      const dotIndex = defaultName.lastIndexOf('.');
      if (dotIndex > 0 && type === 'file') {
        creatingInputRef.value.setSelectionRange(0, dotIndex);
      } else {
        creatingInputRef.value.select();
      }
    }
  });
};

const confirmCreate = async () => {
  if (!creatingItem.value.active) return;

  const name = creatingItem.value.name.trim();
  if (!name) {
    cancelCreate();
    return;
  }

  const { type, parentDir } = creatingItem.value;
  creatingItem.value.active = false;

  try {
    if (type === 'folder') {
      await invoke('create_folder', { parentDir, folderName: name });
    } else {
      await invoke('create_file', { parentDir, fileName: name });
    }
    await loadDirectory(currentPath.value);
    ElMessage.success(type === 'folder' ? '文件夹已创建' : '文件已创建');
  } catch (error) {
    console.error('Failed to create:', error);
    ElMessage.error(`创建失败: ${error}`);
  }
};

const cancelCreate = () => {
  creatingItem.value.active = false;
};

const handleContextMenu = async (event, file) => {
  selectedFile.value = file;
  explorerMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    file,
  };

  await nextTick();
  if (contextMenuRef.value) {
    const menuWidth = contextMenuRef.value.offsetWidth;
    const menuHeight = contextMenuRef.value.offsetHeight;
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;

    if (event.clientX + menuWidth > windowWidth) {
      explorerMenu.value.x = event.clientX - menuWidth;
    }
    if (event.clientY + menuHeight > windowHeight) {
      explorerMenu.value.y = event.clientY - menuHeight;
    }
  }
};

const handleEmptyAreaContextMenu = async (event) => {
  selectedFile.value = null;
  explorerMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    file: null,
  };

  await nextTick();
  if (contextMenuRef.value) {
    const menuWidth = contextMenuRef.value.offsetWidth;
    const menuHeight = contextMenuRef.value.offsetHeight;
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;

    if (event.clientX + menuWidth > windowWidth) {
      explorerMenu.value.x = event.clientX - menuWidth;
    }
    if (event.clientY + menuHeight > windowHeight) {
      explorerMenu.value.y = event.clientY - menuHeight;
    }
  }
};

const handleOutsideMousedown = (e) => {
  if (!favoritesExpanded.value) return;

  const favoritesBar = favoritesBarRef.value;
  const expandBtn = document.querySelector('.path-expand-btn');

  if (
    (favoritesBar && favoritesBar.contains(e.target)) ||
    (expandBtn && expandBtn.contains(e.target))
  ) {
    return;
  }

  favoritesExpanded.value = false;
};

const hideContextMenu = () => {
  explorerMenu.value.visible = false;
};

onMounted(() => {
  loadDirectory(currentPath.value);
  loadFavorites();
  document.addEventListener('mousedown', handleOutsideMousedown);
  window.addEventListener('click', hideContextMenu);
  window.addEventListener('resize', updateContainerHeight);
  updateContainerHeight();
  registerFolderSizeTaskListeners();
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleOutsideMousedown);
  window.removeEventListener('click', hideContextMenu);
  window.removeEventListener('resize', updateContainerHeight);
  if (removeFolderSizeItemListener) removeFolderSizeItemListener();
  if (removeFolderSizeCompleteListener) removeFolderSizeCompleteListener();
});

// Watch for external changes to explorerPath (e.g. from persistence loading)
watch(() => explorerPath.value, (newPath, oldPath) => {
  if (newPath !== oldPath) {
    loadDirectory(newPath);
  }
});

</script>

<style scoped>
.file-explorer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #fff;
}

.explorer-header {
  position: relative;
  padding: 10px 20px;
  border-bottom: 1px solid #f0f0f0;
  background-color: #fafafa;
}

.path-nav {
  display: flex;
  align-items: center;
  gap: 15px;
}

.path-display {
  flex: 1;
  overflow: hidden;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: text;
  min-height: 24px;
  display: flex;
  align-items: center;
  transition: background-color 0.2s;
}

.path-display:hover {
  background-color: #f0f0f0;
}

.explorer-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 20px;
  position: relative;
}

.file-grid-viewport,
.file-list-viewport {
  position: relative;
  width: 100%;
}

.file-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 15px;
  padding-top: 20px;
}

.file-list-content {
  position: relative;
}

.file-list-header {
  display: flex;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-weight: bold;
  font-size: 13px;
  color: #909399;
  background-color: #fff;
  position: sticky;
  top: 0;
  z-index: 10;
  margin: 0 -20px;
  padding-left: 32px;
  padding-right: 32px;
}

.col-name.sortable,
.col-size.sortable,
.col-date.sortable {
  cursor: pointer;
  display: flex;
  align-items: center;
  user-select: none;
}

.col-name.sortable:hover,
.col-size.sortable:hover,
.col-date.sortable:hover {
  color: #409eff;
}

.sort-icon {
  margin-left: 4px;
  font-size: 12px;
}

.file-list-item {
  display: flex;
  padding: 0 12px;
  height: 40px;
  border-radius: 4px;
  cursor: pointer;
  align-items: center;
  transition: background-color 0.2s;
}

.file-list-item:hover {
  background-color: #f5f7fa;
}

.file-list-item.is-selected {
  background-color: #ecf5ff;
}

.file-list-item.is-highlighted {
  background-color: #fdf6ec;
  outline: 2px solid #e6a23c;
  outline-offset: -2px;
  z-index: 1;
}

.col-name {
  flex: 2;
  display: flex;
  align-items: center;
  gap: 10px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.col-size {
  flex: 1;
  color: #909399;
  font-size: 12px;
}

.col-date {
  flex: 1.5;
  color: #909399;
  font-size: 12px;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  font-feature-settings: "tnum";
}

.file-list-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.file-list-icon-placeholder {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.file-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 10px;
  width: 100px;
  height: 110px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
  text-align: center;
}

.file-item:hover {
  background-color: #f5f7fa;
}

.file-item.is-reparse-point {
  opacity: 0.8;
}

.file-item.is-highlighted {
  background-color: #fdf6ec;
  outline: 2px solid #e6a23c;
  outline-offset: -2px;
  z-index: 1;
}

.file-icon-wrapper {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  position: relative;
}

.reparse-point-badge {
  position: absolute;
  bottom: -2px;
  right: -2px;
  font-size: 14px;
  background: #fff;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
}

.file-list-item.is-reparse-point {
  opacity: 0.8;
  font-style: italic;
}

.reparse-tag {
  margin-left: 8px;
  font-style: normal;
  font-size: 10px;
  height: 18px;
  padding: 0 4px;
}

.special-folder-tip {
  font-size: 12px;
  color: #e6a23c;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed #eee;
  max-width: 200px;
}

.file-icon {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.file-icon-placeholder {
  font-size: 32px;
}

.file-info {
  width: 100%;
}

.file-name {
  font-size: 12px;
  color: #606266;
  word-break: break-all;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.4;
}

/* 右键菜单基础样式 */
.explorer-context-menu {
  position: fixed;
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

.context-menu-item.disabled {
  color: #c0c4cc;
  cursor: not-allowed;
}

.context-menu-item.delete {
  color: #f56c6c;
}

.context-menu-item.delete:hover {
  background-color: #fef0f0;
}

.context-menu-divider {
  height: 1px;
  background-color: var(--app-border-light, #f0f0f0);
  margin: 4px 8px;
}

/* 工具提示样式 */
.file-tooltip p {
  margin: 4px 0;
  font-size: 12px;
}

/* 右键菜单二级菜单样式 */
.context-menu-item.has-submenu {
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

.submenu {
  display: none;
  position: absolute;
  left: 100%;
  top: -4px;
  background-color: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--app-border, #e8eaed);
  border-radius: var(--app-radius-md, 8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 4px rgba(0, 0, 0, 0.04);
  min-width: 140px;
  padding: 4px;
  list-style: none;
  z-index: 1001;
  max-height: 300px;
  overflow-y: auto;
}

.context-menu-item.has-submenu:hover>.submenu {
  display: block;
}

.submenu.submenu-left {
  left: auto;
  right: 100%;
}

.context-menu-item.active {
  color: var(--app-active-border, #409eff);
  background-color: var(--app-active-bg, #ecf5ff);
}

.context-menu-item.active::before {
  content: '✓';
  position: absolute;
  left: 5px;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

:deep(.el-breadcrumb__item) {
  cursor: pointer;
}

:deep(.el-breadcrumb__item:hover .el-breadcrumb__inner) {
  color: #409eff;
}

/* 新建文件/文件夹内联输入 */
.creating-item-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin: 4px 0;
  background-color: #ecf5ff;
  border-radius: 4px;
}

.creating-item-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.creating-item-text-input {
  flex: 1;
  border: 1px solid #409eff;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 13px;
  outline: none;
  background: white;
  color: #606266;
}

.creating-item-text-input:focus {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
}

.favorites-section {
  display: flex;
  align-items: center;
}

.favorite-toggle-btn {
  margin: 0 2px;
}

.path-expand-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  cursor: pointer;
  color: #909399;
  padding: 2px 4px;
  border-radius: 4px;
  font-size: 13px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.path-expand-btn:hover {
  color: #409eff;
  background: #ecf5ff;
}

/* 收藏展开行 */
.favorites-bar {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 10;
  background: #fafafa;
  border-bottom: 1px solid #ebeef5;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  padding: 4px 12px;
  max-height: 240px;
  overflow-y: auto;
}

.favorites-bar-empty {
  color: #909399;
  font-size: 12px;
  padding: 4px 8px;
}

.favorites-bar-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.15s;
}

.favorites-bar-item:hover {
  background: #ecf5ff;
}

.favorites-bar-item .fav-star-icon {
  color: #e6a23c;
  font-size: 13px;
  flex-shrink: 0;
}

.fav-bar-name {
  font-weight: 500;
  flex-shrink: 0;
}

.fav-bar-path {
  color: #909399;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.fav-bar-count {
  color: #e6a23c;
  font-size: 11px;
  flex-shrink: 0;
}

.favorites-bar-item .fav-remove-btn {
  opacity: 0;
  transition: opacity 0.15s;
  margin-left: auto;
  flex-shrink: 0;
}

.favorites-bar-item:hover .fav-remove-btn {
  opacity: 1;
}
</style>
