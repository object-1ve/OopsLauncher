import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'

// 检测是否在 Tauri 环境中运行
const isTauri = () => !!window.__TAURI_INTERNALS__;

// 辅助函数：生成 display_name，去掉常见后缀
const generateDisplayName = (fileName) => {
  if (fileName) {
    const lastDotIndex = fileName.lastIndexOf('.')
    if (lastDotIndex > 0) {
      const ext = fileName.substring(lastDotIndex + 1).toLowerCase()
      const commonExtensions = ['exe', 'lnk', 'js', 'ts', 'html', 'css', 'py', 'rs', 'c', 'cpp', 'h', 'hpp', 'go', 'sql', 'yml', 'yaml', 'toml', 'xml', 'txt', 'md', 'json']
      if (commonExtensions.includes(ext)) {
        return fileName.substring(0, lastDotIndex)
      }
    }
  }
  return fileName || ''
}

// 特殊分类 ID
export const SPECIAL_CATEGORIES = {
  ALL_FILES: 'all_files'
}

// Global state
const currentCategory = ref(SPECIAL_CATEGORIES.ALL_FILES)
const sortMethod = ref('openCount')
const sortOrder = ref('desc')
const classifyMethod = ref('none')

const normalizeSortMethod = (val) => ['name', 'openCount', 'created_at'].includes(val) ? val : 'openCount'
const normalizeSortOrder = (val) => ['asc', 'desc'].includes(val) ? val : 'desc'
const normalizeClassifyMethod = (val) => ['none', 'type'].includes(val) ? val : 'none'
const normalizeCategory = (val) => val || SPECIAL_CATEGORIES.ALL_FILES

const applyLauncherState = (state = {}) => {
  currentCategory.value = normalizeCategory(state.current_category ?? state.currentCategory)
  sortMethod.value = normalizeSortMethod(state.sort_method ?? state.sortMethod)
  sortOrder.value = normalizeSortOrder(state.sort_order ?? state.sortOrder)
  classifyMethod.value = normalizeClassifyMethod(state.classify_method ?? state.classifyMethod)
}

const getLauncherStatePayload = () => ({
  current_category: normalizeCategory(currentCategory.value),
  sort_method: normalizeSortMethod(sortMethod.value),
  sort_order: normalizeSortOrder(sortOrder.value),
  classify_method: normalizeClassifyMethod(classifyMethod.value)
})

let isApplyingLauncherState = false
let launcherStateSaveQueued = false
let isSavingLauncherState = false

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

const customCategories = ref([])
const filesByCategory = ref({
  'main': [] // key 是分类的 ID
})
const searchQuery = ref('') // 搜索关键词
const showSearchOverlay = ref(false) // 是否显示搜索遮罩层

// 标记数据是否已从数据库加载，防止未加载完成就触发保存导致数据丢失
let hasLoaded = false;

// 标记 Tauri 监听器是否已设置，防止重复注册
let tauriListenersSet = false;

const sortFiles = (files) => {
  return [...files].sort((a, b) => {
    let result = 0
    const getFileType = (file) => {
      if (file.type) return String(file.type).toLowerCase()
      const fileName = file.name || ''
      const lastDotIndex = fileName.lastIndexOf('.')
      return lastDotIndex > -1 ? fileName.substring(lastDotIndex + 1).toLowerCase() : ''
    }
    if (classifyMethod.value === 'type') {
      result = getFileType(a).localeCompare(getFileType(b))
      if (result !== 0) return result
    }

    if (sortMethod.value === 'openCount') {
      result = (a.openCount || 0) - (b.openCount || 0)
    } else if (sortMethod.value === 'created_at') {
      result = (a.created_at || 0) - (b.created_at || 0)
    } else {
      result = (a.displayName || a.name || '').localeCompare(b.displayName || b.name || '')
    }

    return sortOrder.value === 'asc' ? result : -result
  })
}

const getCurrentCategoryFiles = () => {
  if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
    const allFiles = Object.values(filesByCategory.value).flat()
    const uniqueFilesMap = new Map()
    allFiles.forEach(file => {
      if (!uniqueFilesMap.has(file.path) || (file.openCount || 0) > (uniqueFilesMap.get(file.path).openCount || 0)) {
        uniqueFilesMap.set(file.path, file)
      }
    })
    return Array.from(uniqueFilesMap.values())
  }
  return filesByCategory.value[currentCategory.value] || []
}

// Computed
const globalSearchResults = computed(() => {
  if (!searchQuery.value) return []
  const query = searchQuery.value.toLowerCase()
  // 展平所有分类的文件
  const allFiles = Object.values(filesByCategory.value).flat()
  
  const results = []
  const seenPaths = new Set()

  for (const file of allFiles) {
    if (seenPaths.has(file.path)) continue

    let matchReason = ''
    if (file.displayName && file.displayName.toLowerCase().includes(query)) {
      matchReason = '名称匹配'
    } else if (file.name && file.name.toLowerCase().includes(query)) {
      matchReason = '文件名匹配'
    } else if (file.notes && file.notes.toLowerCase().includes(query)) {
      matchReason = '备注匹配'
    } else if (file.path && file.path.toLowerCase().includes(query)) {
      matchReason = '路径匹配'
    }

    if (matchReason) {
      results.push({
        ...file,
        matchReason
      })
      seenPaths.add(file.path)
    }
  }
  
  return results
})

const currentFiles = computed(() => {
  return sortFiles(getCurrentCategoryFiles())
})

const groupedCurrentFiles = computed(() => {
  const files = getCurrentCategoryFiles()
  if (classifyMethod.value !== 'type') {
    return [{
      type: 'all',
      label: '全部',
      files: sortFiles(files)
    }]
  }

  const groupedMap = new Map()
  files.forEach(file => {
    const fileName = file.name || ''
    const fileType = (file.type || (fileName.lastIndexOf('.') > -1 ? fileName.substring(fileName.lastIndexOf('.') + 1) : '') || 'null').toLowerCase()
    if (!groupedMap.has(fileType)) {
      groupedMap.set(fileType, [])
    }
    groupedMap.get(fileType).push(file)
  })

  return [...groupedMap.entries()]
    .sort((a, b) => {
      const countDiff = b[1].length - a[1].length
      if (countDiff !== 0) return countDiff
      return a[0].localeCompare(b[0])
    })
    .map(([type, groupFiles]) => ({
      type,
      label: type,
      files: sortFiles(groupFiles)
    }))
})

const allCategories = computed({
  get: () => customCategories.value,
  set: (val) => {
    customCategories.value = val
  }
})

const generateId = () => {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

export function useFiles() {
  
  // Methods
  const addCategory = async (name) => {
    if (!name) return
    
    // Check if name already exists
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
    
    // 检查是否是最后一个分类，不允许删除最后一个分类以保证系统运行
    if (customCategories.value.length <= 1) {
      console.warn('Cannot delete the last category.')
      return
    }
    
    const index = customCategories.value.findIndex(c => c.id === id)
    if (index !== -1) {
      customCategories.value.splice(index, 1)
      
      // 删除该分类下的所有文件映射
      if (filesByCategory.value[id]) {
        delete filesByCategory.value[id]
      }
      
      // 如果删除的是当前选中的分类，切换到列表中的第一个分类
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

  const switchCategory = (categoryId) => {
    currentCategory.value = categoryId
    if (categoryId !== SPECIAL_CATEGORIES.ALL_FILES && !filesByCategory.value[categoryId]) {
      filesByCategory.value[categoryId] = []
    }
  }

  const updateCategoryOrder = async (newOrder) => {
    customCategories.value = newOrder.map((cat, index) => ({
      ...cat,
      sort_order: index
    }))
    await saveCategories()
  }

  const getFileIcon = async (file) => {
    const ext = file.name.split('.').pop().toLowerCase()
    const iconMap = {
      'exe': '📦', 'pdf': '📄', 'doc': '📄', 'docx': '📄',
      'xls': '📊', 'xlsx': '📊', 'ppt': '📋', 'pptx': '📋',
      'txt': '📝', 'md': '📝', 'jpg': '🖼️', 'jpeg': '🖼️',
      'png': '🖼️', 'gif': '🖼️', 'svg': '🖼️', 'mp4': '🎬',
      'mp3': '🎵', 'wav': '🎵', 'zip': '📦', 'rar': '📦',
      '7z': '📦', 'json': '🔧', 'js': '🔧', 'ts': '🔧',
      'html': '🌐', 'css': '🎨', 'py': '🐍', 'java': '☕',
      'c': '📟', 'cpp': '📟', 'h': '📟', 'hpp': '📟',
      'go': '🐹', 'rs': '🦀', 'php': '🐘', 'rb': '🐰',
      'swift': '🍎', 'kt': '🤖', 'xml': '📰', 'yml': '📋',
      'yaml': '📋', 'toml': '📋'
    }
    return iconMap[ext] || '📄'
  }

  let isSaving = false;
  let savePending = false;

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
        for (const [categoryId, categoryFiles] of Object.entries(filesByCategory.value)) {
          for (const file of categoryFiles) {
            // 转换openCount为open_count, displayName为display_name
            // 显式指定所有 FileInfo 要求的字段，确保不丢失且不为 undefined/null (针对非 Option 字段)
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
              notes: file.notes || null
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
        isApplyingLauncherState = true
        try {
          applyLauncherState({
            current_category: localStorage.getItem('oopslauncher_current_category'),
            sort_method: localStorage.getItem('oopslauncher_sort_method'),
            sort_order: localStorage.getItem('oopslauncher_sort_order'),
            classify_method: localStorage.getItem('oopslauncher_classify_method')
          })
        } finally {
          isApplyingLauncherState = false
        }

        const savedFiles = localStorage.getItem('oopslauncher_files')
        if (savedFiles) {
          filesByCategory.value = JSON.parse(savedFiles)
        }
        const savedCats = localStorage.getItem('oopslauncher_categories')
        if (savedCats) {
          customCategories.value = JSON.parse(savedCats)
        }
        
        hasLoaded = true;

        // 如果没有任何分类，则创建一个默认的
        if (customCategories.value.length === 0) {
          customCategories.value.unshift({ 
            id: 'main', 
            parent_id: null,
            name: 'main', 
            icon: null,
            sort_order: 0 
          })
        }
        
        // 确保当前选中的分类有效
        if (
          currentCategory.value !== SPECIAL_CATEGORIES.ALL_FILES &&
          !customCategories.value.some(c => c.id === currentCategory.value)
        ) {
          currentCategory.value = customCategories.value[0].id
        }
        return
      }

      isApplyingLauncherState = true
      try {
        const launcherState = await invoke('load_launcher_state_from_db')
        applyLauncherState(launcherState || {})
      } finally {
        isApplyingLauncherState = false
      }

      // Load categories first
      let loadedCats = await invoke('load_categories_from_db')
      if (!loadedCats || loadedCats.length === 0) {
        // 只有在完全没有分类时才创建默认分类
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
      
      // 确保当前选中的分类在加载后的列表中存在，否则切换到第一个
      if (
        currentCategory.value !== SPECIAL_CATEGORIES.ALL_FILES &&
        !customCategories.value.some(c => c.id === currentCategory.value)
      ) {
        currentCategory.value = customCategories.value[0].id
        await saveLauncherState()
      }

      const loaded = await invoke('load_files_from_db')
      
      // 如果数据库返回空列表，但 localStorage 有数据，说明可能发生了数据库丢失或同步问题
      if (!loaded || loaded.length === 0) {
        const saved = localStorage.getItem('oopslauncher_files')
        if (saved) {
          console.warn('Database is empty, falling back to localStorage');
          filesByCategory.value = JSON.parse(saved)
          hasLoaded = true;
          // 重新保存到数据库
          await saveFiles()
          return
        }
        // First run, no data in DB or localStorage
        hasLoaded = true;
      }

      const organizedFiles = {}
      
      // Ensure all custom categories are present in filesByCategory
      customCategories.value.forEach(cat => {
        if (cat && cat.id) {
          organizedFiles[cat.id] = []
        }
      })

      if (loaded && loaded.length > 0) {
        console.log(`Processing ${loaded.length} loaded files...`)
        for (const file of loaded) {
          const categoryId = file.category
          
          // 如果文件所属的分类已不存在，可以考虑移动到当前第一个分类或丢弃
          const targetId = organizedFiles[categoryId] ? categoryId : customCategories.value[0].id
          
          // 转换字段名并保留 category 属性 (存储的是 ID)
          const { open_count, display_name, created_at, notes, ...otherFields } = file
          const fileWithFormattedFields = {
            ...otherFields,
            openCount: open_count || 0,
            displayName: display_name || generateDisplayName(file.name),
            category: targetId,
            // 如果数据库里没时间（老数据），加载时补全，避免每次保存都变
            created_at: created_at || Date.now(),
            notes: notes || ''
          }
          organizedFiles[targetId].push(fileWithFormattedFields)
        }
      }
      
      filesByCategory.value = organizedFiles
      hasLoaded = true;
      console.log('Final organized files (by ID):', filesByCategory.value)
      console.log('Files loaded successfully from DB:', loaded?.length || 0, 'files')
    } catch (error) {
      console.error('Failed to load files from DB:', error)
      // Fallback to localStorage on error
      const saved = localStorage.getItem('oopslauncher_files')
      if (saved) {
        filesByCategory.value = JSON.parse(saved)
        hasLoaded = true;
      }
    }
  }

  const processFiles = async (fileList) => {
    // 如果是特殊分类，不允许直接添加文件
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
          notes: ''
        }
      }
      
      if (!filesByCategory.value[currentCategory.value].some(f => f.path === fileInfo.path)) {
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
    
    // 返回添加结果，以便调用者可以显示适当的消息
    return {
      addedCount,
      existingCount,
      failedCount
    };
  }

  const deleteFile = async (id) => {
    // 如果是特殊分类，需要在所有分类中查找并删除该文件
    if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
      for (const categoryId in filesByCategory.value) {
        filesByCategory.value[categoryId] = filesByCategory.value[categoryId].filter(file => file.id !== id)
      }
    } else {
      filesByCategory.value[currentCategory.value] = filesByCategory.value[currentCategory.value].filter(file => file.id !== id)
    }
    await saveFiles()
  }

  // 复制文件到指定分类
  const copyFileToCategory = async (file, targetCategoryId) => {
    if (!file || !targetCategoryId) return { success: false }
    if (targetCategoryId === SPECIAL_CATEGORIES.ALL_FILES) return { success: false, reason: 'cannot_copy_to_special_category' }
    if (file.category === targetCategoryId) return { success: false }
    // 确保目标分类存在
    if (!filesByCategory.value[targetCategoryId]) return { success: false }
    // 去重检查：目标分类是否已有相同 path 的文件
    if (filesByCategory.value[targetCategoryId].some(f => f.path === file.path)) {
      return { success: false, reason: 'duplicate' }
    }
    // 深拷贝 + 新 ID + 新分类 + 重置 openCount 和 created_at
    const newId = generateId()
    const copiedFile = {
      ...file,
      id: newId,
      category: targetCategoryId,
      openCount: 0,
      created_at: Date.now()
    }
    filesByCategory.value[targetCategoryId].push(copiedFile)
    await saveFiles()
    return { success: true }
  }

  const openFile = async (file) => {
    try {
      console.log(`Opening file: ${file.path}`)
      if (isTauri()) {
        await invoke('open_path', { path: file.path })
      } else {
        window.open(file.path, '_blank')
      }
      
      // 增加打开次数
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
      alert(`打开文件失败: ${error.message}`)
    }
  }

  const setupTauriListeners = async () => {
    if (tauriListenersSet) return;
    
    if (window.__TAURI_INTERNALS__?.invoke) {
      tauriListenersSet = true;
      console.log('Setting up Tauri drag-drop listener...')
      await listen('tauri://drag-drop', async (event) => {
        const { paths } = event.payload
        if (paths && paths.length > 0) {
          // 如果是特殊分类，拦截拖拽添加
          if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
            console.warn('Cannot add files to special category via drag-drop');
            ElMessage.warning('不能直接向“全部文件”分类中添加文件')
            return
          }

          if (!filesByCategory.value[currentCategory.value]) {
            filesByCategory.value[currentCategory.value] = []
          }
          
          let addedCount = 0;
          let failedCount = 0;
          let existingCount = 0;

          for (const path of paths) {
            if (!filesByCategory.value[currentCategory.value].some(f => f.path === path)) {
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
            ElMessage.warning(`所有文件都已存在`)
          }
        }
      })
    }
  }

  return {
    currentCategory,
    filesByCategory,
    currentFiles,
    groupedCurrentFiles,
    allCategories,
    switchCategory,
    addCategory,
    renameCategory,
    deleteCategory,
    updateCategoryOrder,
    loadFiles,
    processFiles,
    deleteFile,
    openFile,
    copyFileToCategory,
    setupTauriListeners,
    saveFiles,
    sortMethod,
    sortOrder,
    classifyMethod,
    searchQuery,
    globalSearchResults,
    showSearchOverlay,
    SPECIAL_CATEGORIES
  }
}
