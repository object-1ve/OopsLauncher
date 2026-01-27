import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Global state
const currentCategory = ref('main')
const filesByCategory = ref({
  'main': []
})

// Computed
const currentFiles = computed(() => {
  return filesByCategory.value[currentCategory.value] || []
})

export function useFiles() {
  
  // Methods
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
            allFiles.push({ ...file, category })
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
        const saved = localStorage.getItem('oopslauncher_files')
        if (saved) {
          filesByCategory.value = JSON.parse(saved)
        }
        return
      }

      const loaded = await invoke('load_files_from_db')
      if (loaded) {
        const organizedFiles = { 'main': [] }
        for (const file of loaded) {
          const category = file.category || 'main'
          if (!organizedFiles[category]) {
            organizedFiles[category] = []
          }
          const { category: _, ...fileWithoutCategory } = file
          organizedFiles[category].push(fileWithoutCategory)
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
      const fileInfo = {
        id: Date.now() + Math.random().toString(36).substr(2, 9),
        name: file.name,
        path: file.path || file.webkitRelativePath || file.name, 
        size: file.size,
        type: file.type,
        icon: await getFileIcon(file),
        category: currentCategory.value
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
        return
      }
      window.open(file.path, '_blank')
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
    switchCategory,
    loadFiles,
    processFiles,
    deleteFile,
    openFile,
    setupTauriListeners
  }
}
