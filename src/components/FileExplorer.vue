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

        <div class="path-display" @dblclick="startEditingPath">
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
    </div>

    <div class="explorer-content" ref="explorerContentRef" v-loading="loading" @scroll="handleScroll"
      @contextmenu.prevent="handleEmptyAreaContextMenu">
      <div v-if="files.length === 0 && !loading" class="empty-state">
        <el-empty description="此文件夹为空" />
      </div>

      <!-- 网格视图 -->
      <div v-if="viewMode === 'grid'" class="file-grid-viewport" :style="{ height: virtualGrid.totalHeight + 'px' }">
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
            <div class="file-item"
              :class="{ 'is-selected': selectedFile?.path === file.path, 'is-reparse-point': file.isReparsePoint }"
              @click.stop="handleItemClick(file)" @contextmenu.stop.prevent="handleContextMenu($event, file)">
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
      <div v-else-if="viewMode === 'list'" class="file-list-viewport"
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
          <div v-for="file in virtualList.items" :key="file.path" class="file-list-item"
            :class="{ 'is-selected': selectedFile?.path === file.path, 'is-reparse-point': file.isReparsePoint }"
            @click.stop="handleItemClick(file)" @contextmenu.stop.prevent="handleContextMenu($event, file)">
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
    </div>

    <!-- 资源管理器右键菜单 -->
    <div v-if="explorerMenu.visible" ref="contextMenuRef" class="explorer-context-menu" :style="{
      left: explorerMenu.x + 'px',
      top: explorerMenu.y + 'px',
    }">
      <ul class="context-menu-list">
        <template v-if="explorerMenu.file">
          <li class="context-menu-item" @click="handleOpen(explorerMenu.file)">
            打开
          </li>
          <li class="context-menu-item" @click="handleOpenFileLocation(explorerMenu.file)">
            打开文件所在位置
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
          <template v-if="explorerMenu.file.type === 'directory'">
            <li class="context-menu-divider"></li>
            <li class="context-menu-item" @click="handleCalculateFolderSizes(explorerMenu.file)">
              计算文件夹大小
            </li>
          </template>
        </template>
        <template v-else>
          <li class="context-menu-item has-submenu">
            展示方式
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
            计算所有文件夹大小
          </li>
        </template>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { ArrowLeft, HomeFilled, Loading, CaretTop, CaretBottom } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useFiles } from '@/composables/useFiles';
import { startFolderSizeTask, getFileIcon, openFileLocation } from '@/api/file';
import { ElMessage } from 'element-plus';

const { explorerPath } = useFiles();

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
    sortOrder.value = 'asc';
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
  return date.toLocaleString();
};

const clipboard = ref({
  path: '',
  isCut: false,
});

const selectedFile = ref(null);
const activeFolderSizeTaskId = ref('');

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

const loadDirectory = async (path) => {
  loading.value = true;
  // 切换路径时清空图标缓存和滚动位置
  iconCache.value.clear();
  if (explorerContentRef.value) {
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
  selectedFile.value = file;
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
    ElMessage.error(`打开文件所在位置失败: ${error}`);
  } finally {
    hideContextMenu();
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
  try {
    await invoke('delete_to_trash', { path: file.path });
    ElMessage.success('已移至回收站');
    loadDirectory(currentPath.value);
  } catch (error) {
    console.error('Failed to delete:', error);
    ElMessage.error(`删除失败: ${error}`);
  } finally {
    hideContextMenu();
  }
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

const hideContextMenu = () => {
  explorerMenu.value.visible = false;
};

onMounted(() => {
  loadDirectory(currentPath.value);
  window.addEventListener('click', hideContextMenu);
  window.addEventListener('resize', updateContainerHeight);
  updateContainerHeight();
  registerFolderSizeTaskListeners();
});

onUnmounted(() => {
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
  background: white;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  border: 1px solid #ebeef5;
  border-radius: 4px;
  padding: 4px 0;
  min-width: 150px;
}

.context-menu-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.context-menu-item {
  padding: 8px 16px;
  font-size: 13px;
  color: #606266;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.context-menu-item:hover {
  background-color: #f5f7fa;
  color: #409eff;
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
  background-color: #ebeef5;
  margin: 4px 0;
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

.context-menu-item.has-submenu::after {
  content: '▶';
  font-size: 10px;
  float: right;
  margin-left: 10px;
}

.submenu {
  display: none;
  position: absolute;
  left: 100%;
  top: 0;
  background: white;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  border: 1px solid #ebeef5;
  border-radius: 4px;
  padding: 4px 0;
  min-width: 100px;
  list-style: none;
  z-index: 1001;
}

.context-menu-item.has-submenu:hover>.submenu {
  display: block;
}

.submenu.submenu-left {
  left: auto;
  right: 100%;
}

.context-menu-item.active {
  color: #409eff;
  background-color: #ecf5ff;
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
</style>
