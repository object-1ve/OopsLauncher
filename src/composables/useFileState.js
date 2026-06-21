import { ref, computed } from 'vue'

// 辅助函数：生成 display_name，去掉常见后缀
export const generateDisplayName = (fileName) => {
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

// 统一文件路径比较键，避免同文件因大小写/斜杠差异被重复插入
export const normalizePathKey = (rawPath) => {
  const path = String(rawPath || '').trim()
  if (!path) return ''

  const hasWindowsStyle = /^[a-zA-Z]:[\\/]/.test(path) || path.includes('\\')
  const normalizedSlashes = path.replace(/\//g, '\\').replace(/\\+/g, '\\')
  return hasWindowsStyle ? normalizedSlashes.toLowerCase() : normalizedSlashes
}

export const categoryHasSameFile = (categoryFiles = [], filePath = '') => {
  const targetKey = normalizePathKey(filePath)
  if (!targetKey) return false
  return categoryFiles.some(f => normalizePathKey(f.path) === targetKey)
}

export const getFileIcon = async (file) => {
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

export const generateId = () => {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

// 特殊分类 ID
export const SPECIAL_CATEGORIES = {
  ALL_FILES: 'all_files',
  FILE_EXPLORER: 'file_explorer',
  START_MENU: 'start_menu'
}

// Module-level singleton state
export const currentCategory = ref(SPECIAL_CATEGORIES.ALL_FILES)
export const explorerPath = ref('')
export const explorerHighlightPath = ref('')
export const dragTargetCategory = ref(null)
export const sortMethod = ref('openCount')
export const sortOrder = ref('desc')
export const classifyMethod = ref('none')

export const normalizeSortMethod = (val) => ['name', 'openCount', 'created_at'].includes(val) ? val : 'openCount'
export const normalizeSortOrder = (val) => ['asc', 'desc'].includes(val) ? val : 'desc'
export const normalizeClassifyMethod = (val) => ['none', 'type', 'letter'].includes(val) ? val : 'none'
export const normalizeCategory = (val) => val || SPECIAL_CATEGORIES.ALL_FILES

export const customCategories = ref([])
export const filesByCategory = ref({
  'main': []
})
export const searchQuery = ref('')
export const showSearchOverlay = ref(false)
export const refreshingStartMenu = ref(false)

// Module-level flags
export let hasLoaded = false
export let tauriListenersSet = false
export let isApplyingLauncherState = false

// Allow external mutation of module-level flags
export const setHasLoaded = (val) => { hasLoaded = val }
export const setTauriListenersSet = (val) => { tauriListenersSet = val }
export const setIsApplyingLauncherState = (val) => { isApplyingLauncherState = val }

export const sortFiles = (files) => {
  const getFileType = (file) => {
    if (file.type) return String(file.type).toLowerCase()
    const fileName = file.name || ''
    const lastDotIndex = fileName.lastIndexOf('.')
    return lastDotIndex > -1 ? fileName.substring(lastDotIndex + 1).toLowerCase() : ''
  }

  return [...files].sort((a, b) => {
    const aPinned = !!a.isPinned
    const bPinned = !!b.isPinned
    if (aPinned !== bPinned) {
      return aPinned ? -1 : 1
    }

    let result = 0
    if (classifyMethod.value === 'type') {
      result = getFileType(a).localeCompare(getFileType(b))
      if (result !== 0) return result
    } else if (classifyMethod.value === 'letter') {
      const getFirstLetter = (f) => {
        const name = (f.displayName || f.name || '').trim()
        const first = name.charAt(0).toUpperCase()
        return /^[A-Z]$/.test(first) ? first : '#'
      }
      const aL = getFirstLetter(a)
      const bL = getFirstLetter(b)
      if (aL !== bL) {
        if (aL === '#') return 1
        if (bL === '#') return -1
        return aL.localeCompare(bL)
      }
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

export function getCurrentCategoryFiles() {
  const filterUnwanted = (files) => {
    return files.filter(file => {
      if (file.category === SPECIAL_CATEGORIES.START_MENU || currentCategory.value === SPECIAL_CATEGORIES.START_MENU) {
        const name = (file.displayName || file.name || '').toLowerCase();
        const path = (file.path || '').toLowerCase();
        return !name.includes('uninstall') && !name.includes('卸载') &&
          !path.includes('uninstall') && !path.includes('卸载');
      }
      return true;
    });
  };

  if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
    const allFiles = Object.entries(filesByCategory.value)
      .filter(([catId]) => catId !== SPECIAL_CATEGORIES.START_MENU)
      .map(([_, files]) => files)
      .flat()
    const uniqueFilesMap = new Map()

    filterUnwanted(allFiles).forEach(file => {
      if (
        !uniqueFilesMap.has(file.path) ||
        (!!file.isPinned && !uniqueFilesMap.get(file.path)?.isPinned) ||
        (
          !!file.isPinned === !!uniqueFilesMap.get(file.path)?.isPinned &&
          (file.openCount || 0) > (uniqueFilesMap.get(file.path).openCount || 0)
        )
      ) {
        uniqueFilesMap.set(file.path, file)
      }
    })
    return Array.from(uniqueFilesMap.values())
  }

  const files = filesByCategory.value[currentCategory.value] || [];
  return filterUnwanted(files);
}

// Computed properties
export const globalSearchResults = computed(() => {
  if (!searchQuery.value) return []
  const query = searchQuery.value.toLowerCase()
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

export const currentFiles = computed(() => {
  return sortFiles(getCurrentCategoryFiles().filter(file => !file.isPinned))
})

export const pinnedCurrentFiles = computed(() => {
  return sortFiles(getCurrentCategoryFiles().filter(file => !!file.isPinned))
})

export const groupedCurrentFiles = computed(() => {
  const files = getCurrentCategoryFiles().filter(file => !file.isPinned)
  if (classifyMethod.value === 'none') {
    return [{
      type: 'all',
      label: '全部',
      files: sortFiles(files)
    }]
  }

  const groupedMap = new Map()

  if (classifyMethod.value === 'type') {
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
  }

  if (classifyMethod.value === 'letter') {
    files.forEach(file => {
      const name = (file.displayName || file.name || '').trim()
      let firstChar = name.charAt(0).toUpperCase()
      if (!/^[A-Z]$/.test(firstChar)) {
        firstChar = '#'
      }
      if (!groupedMap.has(firstChar)) {
        groupedMap.set(firstChar, [])
      }
      groupedMap.get(firstChar).push(file)
    })

    return [...groupedMap.entries()]
      .sort((a, b) => {
        if (a[0] === '#') return 1
        if (b[0] === '#') return -1
        return a[0].localeCompare(b[0])
      })
      .map(([letter, groupFiles]) => ({
        type: letter,
        label: letter,
        files: sortFiles(groupFiles)
      }))
  }

  return []
})

export const allCategories = computed({
  get: () => customCategories.value,
  set: (val) => {
    customCategories.value = val
  }
})
