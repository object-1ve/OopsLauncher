import { watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { isTauri } from '@/utils/env'
import {
  currentCategory, sortMethod, sortOrder, classifyMethod, explorerPath,
  customCategories, filesByCategory,
  generateDisplayName, normalizePathKey, generateId,
  normalizeCategory, normalizeSortMethod, normalizeSortOrder, normalizeClassifyMethod,
  SPECIAL_CATEGORIES, getCurrentCategoryFiles,
  isApplyingLauncherState, setIsApplyingLauncherState,
  hasLoaded, setHasLoaded
} from './useFileState'

// --- Launcher State Persistence ---

let launcherStateSaveQueued = false
let isSavingLauncherState = false

const getLauncherStatePayload = () => ({
  current_category: normalizeCategory(currentCategory.value),
  sort_method: normalizeSortMethod(sortMethod.value),
  sort_order: normalizeSortOrder(sortOrder.value),
  classify_method: normalizeClassifyMethod(classifyMethod.value),
  explorer_path: explorerPath.value
})

const applyLauncherState = (state = {}) => {
  currentCategory.value = normalizeCategory(state.current_category ?? state.currentCategory)
  sortMethod.value = normalizeSortMethod(state.sort_method ?? state.sortMethod)
  sortOrder.value = normalizeSortOrder(state.sort_order ?? state.sortOrder)
  classifyMethod.value = normalizeClassifyMethod(state.classify_method ?? state.classifyMethod)
  explorerPath.value = state.explorer_path ?? state.explorerPath ?? ''
}

const saveLauncherState = async () => {
  if (isApplyingLauncherState) return
  if (isSavingLauncherState) {
    launcherStateSaveQueued = true
    return
  }
  isSavingLauncherState = true
  try {
    const payload = getLauncherStatePayload()
    if (isTauri()) {
      await invoke('save_launcher_state_to_db', { state: payload })
    } else {
      localStorage.setItem('oopslauncher_current_category', payload.current_category)
      localStorage.setItem('oopslauncher_sort_method', payload.sort_method)
      localStorage.setItem('oopslauncher_sort_order', payload.sort_order)
      localStorage.setItem('oopslauncher_classify_method', payload.classify_method)
    }
  } catch (error) {
    console.error('Failed to save launcher state:', error)
  } finally {
    isSavingLauncherState = false
    if (launcherStateSaveQueued) {
      launcherStateSaveQueued = false
      await saveLauncherState()
    }
  }
}

watch([currentCategory, sortMethod, sortOrder, classifyMethod, explorerPath], () => {
  saveLauncherState()
})

// --- File Persistence ---

let isSaving = false
let savePending = false

const saveFiles = async () => {
  if (!hasLoaded) {
    console.warn('Cannot save files before they are loaded from DB');
    return;
  }
  if (isSaving) {
    savePending = true;
    return;
  }
  isSaving = true;

  try {
    if (isTauri()) {
      const allFiles = []
      const seenCategoryPath = new Set()
      for (const [categoryId, categoryFiles] of Object.entries(filesByCategory.value)) {
        for (const file of categoryFiles) {
          const normalizedPath = normalizePathKey(file.path)
          const dedupeKey = `${categoryId}::${normalizedPath}`
          if (!normalizedPath || seenCategoryPath.has(dedupeKey)) {
            continue
          }

          // 过滤掉开始菜单中的卸载程序
          if (categoryId === SPECIAL_CATEGORIES.START_MENU) {
            const name = (file.displayName || file.display_name || file.name || '').toLowerCase();
            const lowerPath = normalizedPath.toLowerCase();
            if (name.includes('uninstall') || name.includes('卸载') ||
              lowerPath.includes('uninstall') || lowerPath.includes('卸载')) {
              continue;
            }
          }

          seenCategoryPath.add(dedupeKey)

          const createdAt = Number(file.createdAt ?? file.created_at ?? Date.now())
          const modifiedAt = Number(file.modifiedAt ?? file.modified_at ?? createdAt)
          const fileToSave = {
            id: String(file.id || generateId()),
            name: String(file.name || ''),
            displayName: String(file.displayName || file.display_name || generateDisplayName(file.name) || ''),
            path: String(file.path || ''),
            size: Number(file.size) || 0,
            type: String(file.type || ''),
            icon: String(file.icon || ''),
            content: file.content || null,
            category: String(categoryId || 'main'),
            openCount: Number(file.openCount ?? file.open_count ?? 0),
            lastOpened: file.lastOpened ?? file.last_opened ?? null,
            createdAt,
            modifiedAt,
            notes: file.notes || null,
            isPinned: !!(file.isPinned ?? file.is_pinned)
          }
          allFiles.push(fileToSave)
        }
      }
      await invoke('save_files_to_db', { files: allFiles })
      console.log('Files saved successfully to DB')
    } else {
      localStorage.setItem('oopslauncher_files', JSON.stringify(filesByCategory.value))
    }
  } catch (error) {
    console.error('Failed to save files:', error)
    if (!isTauri()) {
      localStorage.setItem('oopslauncher_files', JSON.stringify(filesByCategory.value))
    } else {
      ElMessage.error(`保存数据失败：${error?.message || error}`)
    }
  } finally {
    isSaving = false;
    if (savePending) {
      savePending = false;
      await saveFiles();
    }
  }
}

const loadFiles = async () => {
  try {
    if (!isTauri()) {
      setIsApplyingLauncherState(true)
      try {
        applyLauncherState({
          current_category: localStorage.getItem('oopslauncher_current_category'),
          sort_method: localStorage.getItem('oopslauncher_sort_method'),
          sort_order: localStorage.getItem('oopslauncher_sort_order'),
          classify_method: localStorage.getItem('oopslauncher_classify_method')
        })
      } finally {
        setIsApplyingLauncherState(false)
      }

      const savedFiles = localStorage.getItem('oopslauncher_files')
      if (savedFiles) {
        filesByCategory.value = JSON.parse(savedFiles)
      }
      const savedCats = localStorage.getItem('oopslauncher_categories')
      if (savedCats) {
        customCategories.value = JSON.parse(savedCats)
      }

      setHasLoaded(true)

      if (customCategories.value.length === 0) {
        customCategories.value.unshift({
          id: 'main',
          parentId: null,
          name: 'main',
          icon: null,
          sortOrder: 0
        })
      }

      if (
        currentCategory.value !== SPECIAL_CATEGORIES.ALL_FILES &&
        !customCategories.value.some(c => c.id === currentCategory.value)
      ) {
        currentCategory.value = customCategories.value[0].id
      }
      return
    }

    setIsApplyingLauncherState(true)
    try {
      const launcherState = await invoke('load_launcher_state_from_db')
      applyLauncherState(launcherState || {})
    } finally {
      setIsApplyingLauncherState(false)
    }

    let loadedCats = await invoke('load_categories_from_db')
    if (!loadedCats || loadedCats.length === 0) {
      loadedCats = [{
        id: 'main',
        parentId: null,
        name: 'main',
        icon: null,
        sortOrder: 0
      }]
      await invoke('save_categories_to_db', { categories: loadedCats })
    }

    customCategories.value = loadedCats

    if (
      currentCategory.value !== SPECIAL_CATEGORIES.ALL_FILES &&
      !customCategories.value.some(c => c.id === currentCategory.value)
    ) {
      currentCategory.value = customCategories.value[0].id
      await saveLauncherState()
    }

    const loaded = await invoke('load_files_from_db')

    const organizedFiles = {}

    // 初始化特殊分类
    organizedFiles[SPECIAL_CATEGORIES.START_MENU] = []

    customCategories.value.forEach(cat => {
      if (cat && cat.id) {
        organizedFiles[cat.id] = []
      }
    })

    if (loaded && loaded.length > 0) {
      console.log(`Processing ${loaded.length} loaded files...`)
      const seenCategoryPath = new Set()
      for (const file of loaded) {
        const categoryId = file.category

        const targetId = organizedFiles[categoryId] ? categoryId : customCategories.value[0].id

        const { openCount, displayName, createdAt, modifiedAt, notes, isPinned, ...otherFields } = file
        const fileWithFormattedFields = {
          ...otherFields,
          openCount: openCount || 0,
          displayName: displayName || generateDisplayName(file.name),
          category: targetId,
          createdAt: createdAt || Date.now(),
          created_at: createdAt || Date.now(),
          modifiedAt: modifiedAt || createdAt || Date.now(),
          modified_at: modifiedAt || createdAt || Date.now(),
          notes: notes || '',
          isPinned: !!isPinned
        }
        const dedupeKey = `${targetId}::${normalizePathKey(fileWithFormattedFields.path)}`
        if (seenCategoryPath.has(dedupeKey)) {
          continue
        }
        seenCategoryPath.add(dedupeKey)
        organizedFiles[targetId].push(fileWithFormattedFields)
      }
    }

    filesByCategory.value = organizedFiles
    setHasLoaded(true)
    console.log('Final organized files (by ID):', filesByCategory.value)
    console.log('Files loaded successfully from DB:', loaded?.length || 0, 'files')

    // 初始加载后，检测当前分类下的文件是否存在
    if (isTauri()) {
      const files = getCurrentCategoryFiles()
      if (files.length > 0) {
        const paths = files.map(f => f.path)
        const existence = await invoke('check_paths_exist', { paths })
        files.forEach((file, index) => {
          file.exists = existence[index]
        })
      }
    }
  } catch (error) {
    console.error('Failed to load files from DB:', error)
    if (isTauri()) {
      filesByCategory.value = {}
      setHasLoaded(true)
      return
    }
    const saved = localStorage.getItem('oopslauncher_files')
    if (saved) {
      filesByCategory.value = JSON.parse(saved)
      setHasLoaded(true)
    }
  }
}

export { saveFiles, loadFiles }

export const useFilePersistence = () => ({
  saveFiles,
  loadFiles
})
