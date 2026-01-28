import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

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
const currentCategory = ref('main')
const customCategories = ref([])
const filesByCategory = ref({
  'main': []
})

// Computed
const currentFiles = computed(() => {
  return filesByCategory.value[currentCategory.value] || []
})

const allCategories = computed(() => {
  return customCategories.value
})

export function useFiles() {
  
  // Methods
  const addCategory = async (name) => {
    if (!name) return
    
    // Check if name already exists
    const existing = customCategories.value.find(c => c.name === name)
    if (existing) return

    const newCategory = {
      id: Date.now().toString(),
      name: name,
    }
    customCategories.value.push(newCategory)
    if (!filesByCategory.value[name]) {
      filesByCategory.value[name] = []
    }
    await saveCategories()
  }

  const renameCategory = async (id, newName) => {
    if (!id || !newName) return
    
    const category = customCategories.value.find(c => c.id === id)
    if (category) {
      const oldName = category.name
      category.name = newName
      
      // Update files that belong to this category
      if (filesByCategory.value[oldName]) {
        filesByCategory.value[newName] = filesByCategory.value[oldName]
        delete filesByCategory.value[oldName]
        
        // Update category property in file objects
        filesByCategory.value[newName].forEach(f => f.category = newName)
      }
      
      if (currentCategory.value === oldName) {
        currentCategory.value = newName
      }

      try {
        if (window.__TAURI_INTERNALS__?.invoke) {
          await invoke('rename_category_in_db', { id, newName })
          // We also need to save files because their category names changed
          await saveFiles()
        } else {
          localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
          localStorage.setItem('oopslauncher_files', JSON.stringify(filesByCategory.value))
        }
      } catch (error) {
        console.error('Failed to rename category:', error)
      }
    }
  }

  const saveCategories = async () => {
    try {
      if (window.__TAURI_INTERNALS__?.invoke) {
        await invoke('save_categories_to_db', { categories: customCategories.value })
      } else {
        localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
      }
    } catch (error) {
      console.error('Failed to save categories:', error)
      localStorage.setItem('oopslauncher_categories', JSON.stringify(customCategories.value))
    }
  }

  const switchCategory = (category) => {
    currentCategory.value = category
    if (!filesByCategory.value[category]) {
      filesByCategory.value[category] = []
    }
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
      if (window.__TAURI_INTERNALS__?.invoke) {
        const allFiles = []
        for (const [category, categoryFiles] of Object.entries(filesByCategory.value)) {
          for (const file of categoryFiles) {
            // 转换openCount为open_count
            const fileWithOpenCount = {
              ...file,
              open_count: file.openCount || 0,
              display_name: file.displayName || generateDisplayName(file.name),
              category
            }
            delete fileWithOpenCount.openCount
            delete fileWithOpenCount.displayName
            allFiles.push(fileWithOpenCount)
          }
        }
        await invoke('save_files_to_db', { files: allFiles })
        console.log('Files saved successfully')
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
      if (!window.__TAURI_INTERNALS__?.invoke) {
        const savedFiles = localStorage.getItem('oopslauncher_files')
        if (savedFiles) {
          filesByCategory.value = JSON.parse(savedFiles)
        }
        const savedCats = localStorage.getItem('oopslauncher_categories')
        if (savedCats) {
          customCategories.value = JSON.parse(savedCats)
        }
        
        // Ensure main exists
        if (!customCategories.value.some(c => c.name === 'main')) {
          customCategories.value.unshift({ id: 'main', name: 'main' })
        }
        return
      }

      // Load categories first
      let loadedCats = await invoke('load_categories_from_db')
      if (loadedCats) {
        // 确保 main 分类存在，既检查 name 也检查 id
        const hasMainByName = loadedCats.some(c => c.name === 'main')
        const hasMainById = loadedCats.some(c => c.id === 'main')
        
        if (!hasMainByName && !hasMainById) {
          const mainCat = { id: 'main', name: 'main' }
          loadedCats.unshift(mainCat)
          await invoke('save_categories_to_db', { categories: loadedCats })
        } else if (hasMainById && !hasMainByName) {
          // 如果 id 存在但 name 不是 main，更正它
          const mainIdx = loadedCats.findIndex(c => c.id === 'main')
          loadedCats[mainIdx].name = 'main'
          await invoke('save_categories_to_db', { categories: loadedCats })
        }
        customCategories.value = loadedCats
      } else {
        customCategories.value = [{ id: 'main', name: 'main' }]
        await invoke('save_categories_to_db', { categories: customCategories.value })
      }

      const loaded = await invoke('load_files_from_db')
      if (loaded) {
        const organizedFiles = { 'main': [] }
        // Ensure all custom categories are present in filesByCategory
        customCategories.value.forEach(cat => {
          if (cat && cat.name) {
            organizedFiles[cat.name] = []
          }
        })

        for (const file of loaded) {
          const category = file.category || 'main'
          if (!organizedFiles[category]) {
            organizedFiles[category] = []
          }
          // 转换open_count为openCount
          const { category: _, open_count, display_name, ...fileWithoutCategory } = file
          const fileWithOpenCount = {
            ...fileWithoutCategory,
            openCount: open_count || 0,
            displayName: display_name || generateDisplayName(file.name)
          }
          organizedFiles[category].push(fileWithOpenCount)
        }
        filesByCategory.value = organizedFiles
      }
      console.log('Files loaded successfully')
    } catch (error) {
      console.error('Failed to load files:', error)
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
    
    for (const file of fileList) {
      let fileInfo;
      
      if (window.__TAURI_INTERNALS__?.invoke && (file.path || file.name)) {
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
          category: currentCategory.value
        }
      }
      
      if (!filesByCategory.value[currentCategory.value].some(f => f.path === fileInfo.path)) {
        filesByCategory.value[currentCategory.value].push(fileInfo)
      }
    }
    await saveFiles()
  }

  const deleteFile = (id) => {
    filesByCategory.value[currentCategory.value] = filesByCategory.value[currentCategory.value].filter(file => file.id !== id)
    saveFiles()
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
    loadFiles,
    processFiles,
    deleteFile,
    openFile,
    setupTauriListeners
  }
}
