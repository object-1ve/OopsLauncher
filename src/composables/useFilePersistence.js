import { watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '@/utils/env'
import {
  currentCategory, sortMethod, sortOrder, classifyMethod,
  customCategories, filesByCategory,
  generateDisplayName, normalizePathKey, generateId,
  normalizeCategory, normalizeSortMethod, normalizeSortOrder, normalizeClassifyMethod,
  SPECIAL_CATEGORIES,
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
  classify_method: normalizeClassifyMethod(classifyMethod.value)
})

const applyLauncherState = (state = {}) => {
  currentCategory.value = normalizeCategory(state.current_category ?? state.currentCategory)
  sortMethod.value = normalizeSortMethod(state.sort_method ?? state.sortMethod)
  sortOrder.value = normalizeSortOrder(state.sort_order ?? state.sortOrder)
  classifyMethod.value = normalizeClassifyMethod(state.classify_method ?? state.classifyMethod)
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

watch([currentCategory, sortMethod, sortOrder, classifyMethod], () => {
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
          seenCategoryPath.add(dedupeKey)

          const fileToSave = {
            id: String(file.id || generateId()),
            name: String(file.name || ''),
            display_name: String(file.displayName || file.display_name || generateDisplayName(file.name) || ''),
            path: String(file.path || ''),
            size: Number(file.size) || 0,
            type: String(file.type || ''),
            icon: String(file.icon || ''),
            content: file.content || null,
            category: String(categoryId || 'main'),
            open_count: Number(file.openCount || file.open_count || 0),
            created_at: Number(file.created_at || Date.now()),
            notes: file.notes || null,
            is_pinned: !!file.isPinned
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
    localStorage.setItem('oopslauncher_files', JSON.stringify(filesByCategory.value))
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
          parent_id: null,
          name: 'main',
          icon: null,
          sort_order: 0
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
        parent_id: null,
        name: 'main',
        icon: null,
        sort_order: 0
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

    if (!loaded || loaded.length === 0) {
      const saved = localStorage.getItem('oopslauncher_files')
      if (saved) {
        console.warn('Database is empty, falling back to localStorage');
        filesByCategory.value = JSON.parse(saved)
        setHasLoaded(true)
        await saveFiles()
        return
      }
      setHasLoaded(true)
    }

    const organizedFiles = {}

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

        const { open_count, display_name, created_at, notes, is_pinned, ...otherFields } = file
        const fileWithFormattedFields = {
          ...otherFields,
          openCount: open_count || 0,
          displayName: display_name || generateDisplayName(file.name),
          category: targetId,
          created_at: created_at || Date.now(),
          notes: notes || '',
          isPinned: !!is_pinned
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
  } catch (error) {
    console.error('Failed to load files from DB:', error)
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
