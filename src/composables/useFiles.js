import {
  currentCategory, filesByCategory,
  currentFiles, pinnedCurrentFiles, groupedCurrentFiles, allCategories,
  sortMethod, sortOrder, classifyMethod,
  searchQuery, globalSearchResults, showSearchOverlay,
  SPECIAL_CATEGORIES, explorerPath
} from './useFileState'
import { saveFiles, loadFiles } from './useFilePersistence'
import {
  processFiles, deleteFile, openFile, copyFileToCategory, togglePinFile, calculateFolderSizes, calculateSingleFolderSize,
  setupTauriListeners, addCategory, renameCategory, deleteCategory,
  updateCategoryOrder, switchCategory
} from './useFileActions'

export { SPECIAL_CATEGORIES }

export function useFiles() {
  return {
    // State
    currentCategory,
    filesByCategory,
    sortMethod,
    sortOrder,
    classifyMethod,
    searchQuery,
    showSearchOverlay,
    SPECIAL_CATEGORIES,
    explorerPath,

    // Computed
    currentFiles,
    pinnedCurrentFiles,
    groupedCurrentFiles,
    allCategories,
    globalSearchResults,

    // Category operations
    switchCategory,
    addCategory,
    renameCategory,
    deleteCategory,
    updateCategoryOrder,

    // File operations
    processFiles,
    deleteFile,
    openFile,
    copyFileToCategory,
    togglePinFile,
    calculateFolderSizes,
    calculateSingleFolderSize,

    // Persistence
    saveFiles,
    loadFiles,

    // Listeners
    setupTauriListeners
  }
}
