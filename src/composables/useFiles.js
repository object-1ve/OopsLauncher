import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 检测是否在 Tauri 环境中运行
const isTauri = () => !!window.__TAURI_INTERNALS__;

// 辅助函数：生成 display_name，去掉常见后缀
const generateDisplayName = (fileName) => {
  if (fileName) {
    const lastDotIndex = fileName.lastIndexOf('.')
    if (lastDotIndex > 0) {
      const ext = fileName.substring(lastDotIndex + 1).toLowerCase()
      const commonExtensions = ['exe', 'js', 'ts', 'html', 'css', 'py', 'rs', 'c', 'cpp', 'h', 'hpp', 'go', 'sql', 'yml', 'yaml', 'toml', 'xml', 'txt', 'md', 'json']
      if (commonExtensions.includes(ext)) {
        return fileName.substring(0, lastDotIndex)
      }
    }
  }
  return fileName
}

// Global state
const currentCategory = ref('main') // 这里存储分类的 ID
const customCategories = ref([])
const filesByCategory = ref({
  'main': [] // key 是分类的 ID
})
const sortMethod = ref('openCount') // 默认按打开次数排序
const sortOrder = ref('desc') // 默认降序

// Computed
const currentFiles = computed(() => {
  const files = filesByCategory.value[currentCategory.value] || []
  
  // 应用排序逻辑
  return [...files].sort((a, b) => {
    let result = 0
    if (sortMethod.value === 'openCount') {
      result = (a.openCount || 0) - (b.openCount || 0)
    } else if (sortMethod.value === 'created_at') {
      result = (a.created_at || 0) - (b.created_at || 0)
    } else {
      // 默认按名称排序
      result = (a.displayName || a.name || '').localeCompare(b.displayName || b.name || '')
    }
    
    // 根据升降序翻转结果
    return sortOrder.value === 'asc' ? result : -result
  })
})

const allCategories = computed({
  get: () => customCategories.value,
  set: (val) => {
    customCategories.value = val
  }
})

export function useFiles() {
  
  // Methods
  const addCategory = async (name) => {
    if (!name) return
    
    // Check if name already exists
    const existing = customCategories.value.find(c => c.name === name)
    if (existing) return

    const id = Date.now().toString()
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
        currentCategory.value = customCategories.value[0].id
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
    if (!filesByCategory.value[categoryId]) {
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

  const saveFiles = async () => {
    try {
      if (isTauri()) {
        const allFiles = []
        for (const [categoryId, categoryFiles] of Object.entries(filesByCategory.value)) {
          for (const file of categoryFiles) {
            // 转换openCount为open_count, displayName为display_name
            const fileToSave = {
              ...file,
              open_count: file.openCount || 0,
              display_name: file.displayName || generateDisplayName(file.name),
              category: categoryId,
              created_at: file.created_at || Date.now()
            }
            delete fileToSave.openCount
            delete fileToSave.displayName
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
    }
  }

  const loadFiles = async () => {
    try {
      if (!isTauri()) {
        const savedFiles = localStorage.getItem('oopslauncher_files')
        if (savedFiles) {
          filesByCategory.value = JSON.parse(savedFiles)
        }
        const savedCats = localStorage.getItem('oopslauncher_categories')
        if (savedCats) {
          customCategories.value = JSON.parse(savedCats)
        }
        
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
        if (!customCategories.value.some(c => c.id === currentCategory.value)) {
          currentCategory.value = customCategories.value[0].id
        }
        return
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
      if (!customCategories.value.some(c => c.id === currentCategory.value)) {
        currentCategory.value = customCategories.value[0].id
      }

      const loaded = await invoke('load_files_from_db')
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
          const { open_count, display_name, created_at, ...otherFields } = file
          const fileWithFormattedFields = {
            ...otherFields,
            openCount: open_count || 0,
            displayName: display_name || generateDisplayName(file.name),
            category: targetId,
            // 如果数据库里没时间（老数据），加载时补全，避免每次保存都变
            created_at: created_at || Date.now() 
          }
          organizedFiles[targetId].push(fileWithFormattedFields)
        }
      }
      
      filesByCategory.value = organizedFiles
      console.log('Final organized files (by ID):', filesByCategory.value)
      console.log('Files loaded successfully from DB:', loaded?.length || 0, 'files')
    } catch (error) {
      console.error('Failed to load files from DB:', error)
      // Fallback to localStorage on error
      const saved = localStorage.getItem('oopslauncher_files')
      if (saved) {
        filesByCategory.value = JSON.parse(saved)
      }
    }
  }

  const processFiles = async (fileList) => {
    if (!filesByCategory.value[currentCategory.value]) {
      filesByCategory.value[currentCategory.value] = []
    }
    
    let addedCount = 0;
    let existingCount = 0;
    
    for (const file of fileList) {
      let fileInfo;
      
      if (isTauri() && (file.path || file.name)) {
        try {
          const path = file.path || file.name;
          fileInfo = await invoke('get_file_info', { path });
          fileInfo.id = Date.now() + Math.random().toString(36).substr(2, 9);
          if (!fileInfo.icon || fileInfo.icon === '') {
            fileInfo.icon = await getFileIcon({ name: fileInfo.name });
          }
          fileInfo.category = currentCategory.value;
        } catch (error) {
          console.error(`Failed to get file info for ${file.name}:`, error);
        }
      }

      if (!fileInfo) {
        fileInfo = {
          id: Date.now() + Math.random().toString(36).substr(2, 9),
          name: file.name,
          display_name: generateDisplayName(file.name),
          path: file.path || file.webkitRelativePath || file.name, 
          size: file.size,
          type: file.type,
          icon: await getFileIcon(file),
          category: currentCategory.value,
          created_at: Date.now()
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
      existingCount
    };
  }

  const deleteFile = (id) => {
    filesByCategory.value[currentCategory.value] = filesByCategory.value[currentCategory.value].filter(file => file.id !== id)
    saveFiles()
  }

  // 复制文件到指定分类
  const copyFileToCategory = async (file, targetCategoryId) => {
    if (!file || !targetCategoryId) return { success: false }
    if (file.category === targetCategoryId) return { success: false }
    // 确保目标分类存在
    if (!filesByCategory.value[targetCategoryId]) return { success: false }
    // 去重检查：目标分类是否已有相同 path 的文件
    if (filesByCategory.value[targetCategoryId].some(f => f.path === file.path)) {
      return { success: false, reason: 'duplicate' }
    }
    // 深拷贝 + 新 ID + 新分类 + 重置 openCount 和 created_at
    const newId = Date.now() + Math.random().toString(36).substr(2, 9)
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
      if (window.__TAURI_INTERNALS__?.invoke) {
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
          const updatedFile = filesInCategory[fileIndex]
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
    if (window.__TAURI_INTERNALS__?.invoke) {
      await listen('tauri://drag-drop', async (event) => {
        const { paths } = event.payload
        if (paths && paths.length > 0) {
          if (!filesByCategory.value[currentCategory.value]) {
            filesByCategory.value[currentCategory.value] = []
          }
          
          for (const path of paths) {
            if (!filesByCategory.value[currentCategory.value].some(f => f.path === path)) {
              try {
                const fileInfo = await invoke('get_file_info', { path })
                fileInfo.id = Date.now() + Math.random().toString(36).substr(2, 9)
                if (!fileInfo.icon || fileInfo.icon === '') {
                  fileInfo.icon = await getFileIcon({ name: fileInfo.name })
                }
                fileInfo.category = currentCategory.value
                fileInfo.displayName = fileInfo.display_name || generateDisplayName(fileInfo.name)
                fileInfo.created_at = fileInfo.created_at || Date.now()
                filesByCategory.value[currentCategory.value].push(fileInfo)
              } catch (error) {
                console.error(`Failed to process path ${path}:`, error)
              }
            }
          }
          await saveFiles()
        }
      })
    }
  }

  return {
    currentCategory,
    filesByCategory,
    currentFiles,
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
    sortOrder
  }
}
