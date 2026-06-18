import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { isTauri } from '@/utils/env'
import {
  currentCategory, filesByCategory, customCategories,
  categoryHasSameFile, generateDisplayName, generateId, getFileIcon,
  SPECIAL_CATEGORIES,
  tauriListenersSet, setTauriListenersSet
} from './useFileState'
import { saveFiles } from './useFilePersistence'

// --- File Operations ---

const processFiles = async (fileList) => {
  if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
    return { addedCount: 0, existingCount: 0, error: 'cannot_add_to_special_category' };
  }

  if (!filesByCategory.value[currentCategory.value]) {
    filesByCategory.value[currentCategory.value] = []
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
        fileInfo.category = currentCategory.value;
        fileInfo.displayName = fileInfo.display_name || generateDisplayName(fileInfo.name);
        fileInfo.notes = fileInfo.notes || '';
        fileInfo.isPinned = !!fileInfo.is_pinned;
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
        display_name: generateDisplayName(file.name),
        path: file.path || file.webkitRelativePath || file.name,
        size: file.size,
        type: file.type,
        icon: await getFileIcon(file),
        category: currentCategory.value,
        created_at: Date.now(),
        notes: '',
        isPinned: false
      }
    }

    if (!categoryHasSameFile(filesByCategory.value[currentCategory.value], fileInfo.path)) {
      filesByCategory.value[currentCategory.value].push(fileInfo);
      addedCount++;
    } else {
      existingCount++;
      console.warn(`File already exists: ${fileInfo.path}`);
    }
  }

  if (addedCount > 0) {
    await saveFiles();
  }

  return { addedCount, existingCount, failedCount };
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

// --- Tauri Drag-Drop Listener ---

const setupTauriListeners = async () => {
  if (tauriListenersSet) return;

  if (window.__TAURI_INTERNALS__?.invoke) {
    setTauriListenersSet(true);
    console.log('Setting up Tauri drag-drop listener...')
    await listen('tauri://drag-drop', async (event) => {
      const { paths } = event.payload
      if (paths && paths.length > 0) {
        if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
          console.warn('Cannot add files to special category via drag-drop');
          ElMessage.warning('不能直接向"全部文件"分类中添加文件')
          return
        }

        if (!filesByCategory.value[currentCategory.value]) {
          filesByCategory.value[currentCategory.value] = []
        }

        let addedCount = 0;
        let failedCount = 0;
        let existingCount = 0;

        for (const path of paths) {
          if (!categoryHasSameFile(filesByCategory.value[currentCategory.value], path)) {
            try {
              const fileInfo = await invoke('get_file_info', { path })
              fileInfo.id = generateId()
              if (!fileInfo.icon || fileInfo.icon === '') {
                fileInfo.icon = await getFileIcon({ name: fileInfo.name })
              }
              fileInfo.category = currentCategory.value
              fileInfo.displayName = fileInfo.display_name || generateDisplayName(fileInfo.name)
              fileInfo.created_at = fileInfo.created_at || Date.now()
              fileInfo.notes = fileInfo.notes || ''
              fileInfo.isPinned = !!fileInfo.is_pinned
              filesByCategory.value[currentCategory.value].push(fileInfo)
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
          if (failedCount === 0) {
            ElMessage.success(`成功添加 ${addedCount} 个文件`)
          } else {
            ElMessage.success(`成功添加 ${addedCount} 个文件，但有 ${failedCount} 个文件添加失败`)
          }
        } else if (failedCount > 0) {
          ElMessage.error(`${failedCount} 个文件添加失败，请检查文件是否存在或权限是否足够`)
        } else if (existingCount > 0) {
          ElMessage.warning(`当前分类已存在此文件`)
        }
      }
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

const switchCategory = (categoryId) => {
  currentCategory.value = categoryId
  if (categoryId !== SPECIAL_CATEGORIES.ALL_FILES && !filesByCategory.value[categoryId]) {
    filesByCategory.value[categoryId] = []
  }
}

export { processFiles, deleteFile, openFile, copyFileToCategory, togglePinFile,
         setupTauriListeners, addCategory, renameCategory, deleteCategory,
         saveCategories, updateCategoryOrder, switchCategory }

export const useFileActions = () => ({
  processFiles,
  deleteFile,
  openFile,
  copyFileToCategory,
  togglePinFile,
  setupTauriListeners,
  addCategory,
  renameCategory,
  deleteCategory,
  saveCategories,
  updateCategoryOrder,
  switchCategory
})
