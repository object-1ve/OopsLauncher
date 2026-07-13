import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { isTauri } from '@/utils/env'
import { calculateDirSizes, calculateDirSize, getFileIcon as getNativeFileIcon, incrementOpenCount } from '@/api/file'
import {
  currentCategory, filesByCategory, customCategories,
  categoryHasSameFile, generateDisplayName, generateId, getFileIcon,
  SPECIAL_CATEGORIES, getCurrentCategoryFiles, dragTargetCategory,
  tauriListenersSet, setTauriListenersSet, refreshingStartMenu
} from './useFileState'
import { saveFiles } from './useFilePersistence'

// --- File Operations ---

const processFiles = async (fileList, overrideTargetCategory = null) => {
  let targetCategory = overrideTargetCategory || currentCategory.value;

  // 如果在特殊分类（全部文件、资源管理器、开始菜单），且没有指定目标分类，则默认添加到第一个自定义分类
  if (targetCategory === SPECIAL_CATEGORIES.ALL_FILES ||
    targetCategory === SPECIAL_CATEGORIES.FILE_EXPLORER ||
    targetCategory === SPECIAL_CATEGORIES.START_MENU) {
    if (customCategories.value.length > 0) {
      targetCategory = customCategories.value[0].id;
    } else {
      return { addedCount: 0, existingCount: 0, error: 'cannot_add_to_special_category' };
    }
  }

  if (!filesByCategory.value[targetCategory]) {
    filesByCategory.value[targetCategory] = []
  }

  let addedCount = 0;
  let existingCount = 0;
  let failedCount = 0;

  for (const file of fileList) {
    let fileInfo;
    let isError = false;

    if (isTauri() && (file.path || file.name)) {
      try {
        const path = file.path || file.name;
        fileInfo = await invoke('get_file_info', { path });
        fileInfo.id = generateId();
        if (!fileInfo.icon || fileInfo.icon === '') {
          fileInfo.icon = await getFileIcon({ name: fileInfo.name });
        }
        fileInfo.category = targetCategory;
        fileInfo.displayName = fileInfo.displayName || generateDisplayName(fileInfo.name);
        fileInfo.notes = fileInfo.notes || '';
        fileInfo.isPinned = !!fileInfo.isPinned;
        fileInfo.createdAt = fileInfo.createdAt || Date.now();
        fileInfo.created_at = fileInfo.createdAt;
      } catch (error) {
        console.error(`Failed to get file info for ${file.name}:`, error);
        isError = true;
        failedCount++;
      }
    }

    if (isError) continue;

    if (!fileInfo) {
      fileInfo = {
        id: generateId(),
        name: file.name,
        displayName: generateDisplayName(file.name),
        path: file.path || file.webkitRelativePath || file.name,
        size: file.size,
        type: file.type,
        icon: await getFileIcon(file),
        category: targetCategory,
        createdAt: Date.now(),
        created_at: Date.now(),
        notes: '',
        isPinned: false
      }
    }

    if (!categoryHasSameFile(filesByCategory.value[targetCategory], fileInfo.path)) {
      filesByCategory.value[targetCategory].push(fileInfo);
      addedCount++;
    } else {
      existingCount++;
      console.warn(`File already exists: ${fileInfo.path}`);
    }
  }

  if (addedCount > 0) {
    await saveFiles();
  }

  return { addedCount, existingCount, failedCount, targetCategory };
}

const deleteFile = async (id) => {
  if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
    for (const categoryId in filesByCategory.value) {
      filesByCategory.value[categoryId] = filesByCategory.value[categoryId].filter(file => file.id !== id)
    }
  } else {
    filesByCategory.value[currentCategory.value] = filesByCategory.value[currentCategory.value].filter(file => file.id !== id)
  }
  await saveFiles()
}

const openFile = async (file) => {
  try {
    console.log(`Opening file: ${file.path}`)
    if (isTauri()) {
      await invoke('open_path', { path: file.path })
      // 递增点击次数（同时更新 files 和 favorites 表）
      await incrementOpenCount(file.path).catch(() => {})
    } else {
      window.open(file.path, '_blank')
    }

    const category = file.category || 'main'
    const filesInCategory = filesByCategory.value[category]
    if (filesInCategory) {
      const fileIndex = filesInCategory.findIndex(f => f.id === file.id)
      if (fileIndex !== -1) {
        const updatedFile = { ...filesInCategory[fileIndex] }
        updatedFile.openCount = (updatedFile.openCount || 0) + 1
        filesInCategory[fileIndex] = updatedFile
        await saveFiles()
      }
    }
  } catch (error) {
    console.error('Failed to open file:', error)
    ElMessage.error(`打开文件失败: ${error.message || error}`)
  }
}

const copyFileToCategory = async (file, targetCategoryId) => {
  if (!file || !targetCategoryId) return { success: false }
  if (targetCategoryId === SPECIAL_CATEGORIES.ALL_FILES) return { success: false, reason: 'cannot_copy_to_special_category' }
  if (file.category === targetCategoryId) return { success: false }
  if (!filesByCategory.value[targetCategoryId]) return { success: false }
  if (categoryHasSameFile(filesByCategory.value[targetCategoryId], file.path)) {
    return { success: false, reason: 'duplicate' }
  }
  const newId = generateId()
  const copiedFile = {
    ...file,
    id: newId,
    category: targetCategoryId,
    openCount: 0,
    createdAt: Date.now(),
    created_at: Date.now(),
    isPinned: false
  }
  filesByCategory.value[targetCategoryId].push(copiedFile)
  await saveFiles()
  return { success: true }
}

const togglePinFile = async (file) => {
  if (!file?.id || !file?.category) return { success: false }
  const categoryFiles = filesByCategory.value[file.category]
  if (!Array.isArray(categoryFiles)) return { success: false }
  const index = categoryFiles.findIndex((f) => f.id === file.id)
  if (index === -1) return { success: false }
  const updatedFile = { ...categoryFiles[index], isPinned: !categoryFiles[index].isPinned }
  categoryFiles[index] = updatedFile
  await saveFiles()
  return { success: true, isPinned: !!updatedFile.isPinned }
}

const calculateFolderSizes = async (files, updateCallback) => {
  if (!isTauri() || !files || files.length === 0) return;

  const folderPaths = files
    .filter(f => f.type === 'directory')
    .map(f => f.path);

  if (folderPaths.length === 0) return;

  try {
    const sizes = await calculateDirSizes(folderPaths);
    folderPaths.forEach(path => {
      const file = files.find(f => f.path === path);
      if (file && sizes[path] !== undefined) {
        file.size = sizes[path];
      }
    });
    if (updateCallback) updateCallback();
  } catch (error) {
    console.error('Failed to calculate folder sizes:', error);
    ElMessage.error('计算文件夹大小失败');
  }
}

const calculateSingleFolderSize = async (file) => {
  if (!isTauri() || !file || file.type !== 'directory') return;
  try {
    const size = await calculateDirSize(file.path);
    file.size = size;
    return size;
  } catch (error) {
    console.error(`Failed to calculate size for ${file.path}:`, error);
    throw error;
  }
}

// --- Tauri Drag-Drop Listener ---

// 根据原生拖放事件提供的物理坐标，命中测试出鼠标下方的目标分类。
// dragDropEnabled 为 true 时，外部文件不会触发 DOM 的 dragover/drop 事件，
// 因此必须用 position 反查 DOM，才能把文件投放到悬停的侧边栏分类。
const resolveCategoryFromPosition = (position) => {
  if (!position || typeof document === 'undefined') return null
  const dpr = window.devicePixelRatio || 1
  // Tauri 传来的是物理像素，elementFromPoint 需要 CSS 像素
  const x = position.x / dpr
  const y = position.y / dpr
  const el = document.elementFromPoint(x, y)
  if (!el) return null
  const dropEl = el.closest('[data-drop-category]')
  return dropEl ? dropEl.getAttribute('data-drop-category') : null
}

const setupTauriListeners = async () => {
  if (tauriListenersSet) return;

  if (window.__TAURI_INTERNALS__?.invoke) {
    setTauriListenersSet(true);
    console.log('Setting up Tauri drag-drop listener...')

    // 外部文件拖入时实时高亮悬停的分类
    await listen('tauri://drag-enter', (event) => {
      dragTargetCategory.value = resolveCategoryFromPosition(event.payload?.position)
    })
    await listen('tauri://drag-over', (event) => {
      dragTargetCategory.value = resolveCategoryFromPosition(event.payload?.position)
    })
    await listen('tauri://drag-leave', () => {
      dragTargetCategory.value = null
    })

    await listen('tauri://drag-drop', async (event) => {
      const { paths, position } = event.payload
      if (paths && paths.length > 0) {
        // 优先使用落点命中的分类，其次是悬停期间记录的分类，最后回退到当前分类
        let targetCategory = resolveCategoryFromPosition(position) || dragTargetCategory.value || currentCategory.value;

        // 如果在特殊分类（全部文件、资源管理器、开始菜单），则默认添加到第一个自定义分类
        if (targetCategory === SPECIAL_CATEGORIES.ALL_FILES ||
          targetCategory === SPECIAL_CATEGORIES.FILE_EXPLORER ||
          targetCategory === SPECIAL_CATEGORIES.START_MENU) {
          if (customCategories.value.length > 0) {
            targetCategory = customCategories.value[0].id;
          } else {
            console.warn('No custom categories available to add files');
            ElMessage.warning('请先创建一个分类再添加文件');
            return;
          }
        }

        if (!filesByCategory.value[targetCategory]) {
          filesByCategory.value[targetCategory] = []
        }

        let addedCount = 0;
        let failedCount = 0;
        let existingCount = 0;

        for (const path of paths) {
          if (!categoryHasSameFile(filesByCategory.value[targetCategory], path)) {
            try {
              const fileInfo = await invoke('get_file_info', { path })
              fileInfo.id = generateId()
              if (!fileInfo.icon || fileInfo.icon === '') {
                fileInfo.icon = await getFileIcon({ name: fileInfo.name })
              }
              fileInfo.category = targetCategory
              fileInfo.displayName = fileInfo.displayName || generateDisplayName(fileInfo.name)
              fileInfo.createdAt = fileInfo.createdAt || Date.now()
              fileInfo.created_at = fileInfo.createdAt
              fileInfo.notes = fileInfo.notes || ''
              fileInfo.isPinned = !!fileInfo.isPinned
              filesByCategory.value[targetCategory].push(fileInfo)
              addedCount++;
            } catch (error) {
              console.error(`Failed to process path ${path}:`, error)
              failedCount++;
            }
          } else {
            existingCount++;
          }
        }

        if (addedCount > 0) {
          await saveFiles()
          const targetCat = customCategories.value.find(c => c.id === targetCategory);
          const catName = targetCat ? `「${targetCat.name}」` : '';

          if (failedCount === 0) {
            ElMessage.success(`成功添加 ${addedCount} 个文件到 ${catName}`)
          } else {
            ElMessage.success(`成功添加 ${addedCount} 个文件到 ${catName}，但有 ${failedCount} 个文件添加失败`)
          }
        } else if (failedCount > 0) {
          ElMessage.error(`${failedCount} 个文件添加失败，请检查文件是否存在或权限是否足够`)
        } else if (existingCount > 0) {
          ElMessage.warning(`当前分类已存在此文件`)
        }
      }
      dragTargetCategory.value = null
    })
  }
}

// --- Category Operations ---

const addCategory = async (name) => {
  if (!name) return

  const existing = customCategories.value.find(c => c.name === name)
  if (existing) return

  const id = generateId()
  const newCategory = {
    id: id,
    parent_id: null,
    name: name,
    icon: null,
    sort_order: customCategories.value.length
  }
  customCategories.value.push(newCategory)
  if (!filesByCategory.value[id]) {
    filesByCategory.value[id] = []
  }
  await saveCategories()
}

const renameCategory = async (id, newName) => {
  if (!id || !newName) return

  const category = customCategories.value.find(c => c.id === id)
  if (category) {
    category.name = newName

    try {
      if (isTauri()) {
        await invoke('rename_category_in_db', { id, newName })
      } else {
        localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
      }
    } catch (error) {
      console.error('Failed to rename category:', error)
    }
  }
}

const deleteCategory = async (id) => {
  if (!id) return

  if (customCategories.value.length <= 1) {
    console.warn('Cannot delete the last category.')
    return
  }

  const index = customCategories.value.findIndex(c => c.id === id)
  if (index !== -1) {
    customCategories.value.splice(index, 1)

    if (filesByCategory.value[id]) {
      delete filesByCategory.value[id]
    }

    if (currentCategory.value === id) {
      currentCategory.value = SPECIAL_CATEGORIES.ALL_FILES
    }

    try {
      if (isTauri()) {
        await invoke('delete_category_from_db', { id })
      } else {
        localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
        localStorage.setItem('oopslauncher_files', JSON.stringify(filesByCategory.value))
      }
      console.log(`Category ${id} deleted successfully`)
    } catch (error) {
      console.error('Failed to delete category:', error)
    }
  }
}

const saveCategories = async () => {
  try {
    if (isTauri()) {
      await invoke('save_categories_to_db', { categories: customCategories.value })
    } else {
      localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
    }
  } catch (error) {
    console.error('Failed to save categories:', error)
    localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
  }
}

const updateCategoryOrder = async (newOrder) => {
  customCategories.value = newOrder.map((cat, index) => ({
    ...cat,
    sort_order: index
  }))
  await saveCategories()
}

const switchCategory = async (categoryId) => {
  currentCategory.value = categoryId

  // 如果切换到开始菜单特殊分类，且当前没有数据，执行首次扫描
  if (categoryId === SPECIAL_CATEGORIES.START_MENU) {
    if (!filesByCategory.value[SPECIAL_CATEGORIES.START_MENU] || filesByCategory.value[SPECIAL_CATEGORIES.START_MENU].length === 0) {
      await refreshStartMenuPrograms();
    }
    return
  }

  if (categoryId !== SPECIAL_CATEGORIES.ALL_FILES && !filesByCategory.value[categoryId]) {
    filesByCategory.value[categoryId] = []
  }

  // 检测当前分类下的所有文件是否存在
  if (isTauri()) {
    const files = getCurrentCategoryFiles()
    if (files.length > 0) {
      try {
        const paths = files.map(f => f.path)
        const existence = await invoke('check_paths_exist', { paths })

        // 更新文件存在状态
        files.forEach((file, index) => {
          file.exists = existence[index]
        })
      } catch (error) {
        console.error('Failed to check file existence:', error)
      }
    }
  }
}

// 刷新开始菜单程序
const refreshStartMenuPrograms = async () => {
  if (refreshingStartMenu.value) return;
  
  refreshingStartMenu.value = true;
  const loadingMessage = ElMessage({
    message: '正在扫描开始菜单...',
    duration: 0,
    type: 'info'
  });

  try {
    const results = await invoke('scan_start_menu_programs')
    filesByCategory.value[SPECIAL_CATEGORIES.START_MENU] = results
    await saveFiles()
    
    loadingMessage.close();
    ElMessage.success(`刷新成功，共发现 ${results.length} 个程序`);
    return results
  } catch (error) {
    loadingMessage.close();
    console.error('Failed to scan start menu:', error)
    ElMessage.error('扫描开始菜单失败')
  } finally {
    refreshingStartMenu.value = false;
  }
}

export {
  processFiles, deleteFile, openFile, copyFileToCategory, togglePinFile, calculateFolderSizes, calculateSingleFolderSize,
  setupTauriListeners, addCategory, renameCategory, deleteCategory, saveCategories,
  updateCategoryOrder, switchCategory, refreshStartMenuPrograms
}

export const useFileActions = () => ({
  processFiles,
  deleteFile,
  openFile,
  copyFileToCategory,
  togglePinFile,
  calculateFolderSizes,
  calculateSingleFolderSize,
  setupTauriListeners,
  addCategory,
  renameCategory,
  deleteCategory,
  saveCategories,
  updateCategoryOrder,
  switchCategory,
  refreshStartMenuPrograms,
  getCurrentCategoryFiles
})
