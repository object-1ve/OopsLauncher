import {
  currentCategory, filesByCategory,
  currentFiles, pinnedCurrentFiles, groupedCurrentFiles, allCategories,
  sortMethod, sortOrder, classifyMethod,
  searchQuery, globalSearchResults, showSearchOverlay,
  SPECIAL_CATEGORIES
} from './useFileState'
import { saveFiles, loadFiles } from './useFilePersistence'
import { processFiles, deleteFile, openFile, copyFileToCategory, togglePinFile,
         setupTauriListeners, addCategory, renameCategory, deleteCategory,
         updateCategoryOrder, switchCategory } from './useFileActions'

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

    // Persistence
    saveFiles,
    loadFiles,

    // Listeners
    setupTauriListeners
  }
}
