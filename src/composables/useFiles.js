import {
  currentCategory, filesByCategory,
  currentFiles, pinnedCurrentFiles, groupedCurrentFiles, allCategories,
  sortMethod, sortOrder, classifyMethod,
  searchQuery, globalSearchResults, recentFiles, showSearchOverlay, refreshingStartMenu,
  SPECIAL_CATEGORIES, explorerPath, explorerHighlightPath, dragTargetCategory
} from './useFileState'
import { saveFiles, loadFiles } from './useFilePersistence'
import {
  processFiles, deleteFile, openFile, copyFileToCategory, togglePinFile, calculateFolderSizes, calculateSingleFolderSize,
  setupTauriListeners, addCategory, renameCategory, deleteCategory,
  updateCategoryOrder, switchCategory, refreshStartMenuPrograms
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
    refreshingStartMenu,
    SPECIAL_CATEGORIES,
    explorerPath,
    explorerHighlightPath,
    dragTargetCategory,

    // Computed
    currentFiles,
    pinnedCurrentFiles,
    groupedCurrentFiles,
    allCategories,
    globalSearchResults,
    recentFiles,

    // Category operations
    switchCategory,
    refreshStartMenuPrograms,
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
