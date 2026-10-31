import { reactive } from 'vue'
import { ElMessage } from 'element-plus'
import {
  customCategories, dragTargetCategory, SPECIAL_CATEGORIES
} from './useFileState'
import { moveFileToCategory, copyFileToCategory } from './useFileActions'

// Windows 上 Tauri 的 dragDropEnabled（外部文件拖入所需）会禁用 WebView2 的 HTML5 拖放，
// 因此启动器图标改用指针事件实现拖拽，不依赖 dragstart/drop。
const DRAG_THRESHOLD = 4
const CLICK_SUPPRESS_MS = 250

const SPECIAL_TARGETS = new Set([
  SPECIAL_CATEGORIES.ALL_FILES,
  SPECIAL_CATEGORIES.FILE_EXPLORER,
  SPECIAL_CATEGORIES.START_MENU
])

export const iconDragState = reactive({
  active: false,
  file: null,
  x: 0,
  y: 0
})

let pending = null
let hoverCategoryId = null
let suppressClickUntil = 0

const onKeyDown = (event) => {
  if (event.key === 'Escape') reset()
}

const onBlur = () => reset()

const reset = () => {
  const element = pending?.element
  const pointerId = pending?.pointerId
  pending = null
  hoverCategoryId = null
  iconDragState.active = false
  iconDragState.file = null
  dragTargetCategory.value = null
  if (typeof document !== 'undefined') document.body.classList.remove('icon-dragging')
  window.removeEventListener('pointermove', onPointerMove, true)
  window.removeEventListener('pointerup', onPointerUp, true)
  window.removeEventListener('pointercancel', reset, true)
  window.removeEventListener('keydown', onKeyDown, true)
  window.removeEventListener('blur', onBlur)
  if (element && pointerId != null) {
    try {
      element.releasePointerCapture(pointerId)
    } catch (error) {
      // 指针已释放，忽略
    }
  }
}

const applyHover = (x, y) => {
  const element = document.elementFromPoint(x, y)
  const dropElement = element?.closest?.('[data-drop-category]')
  hoverCategoryId = dropElement?.getAttribute('data-drop-category') || null
  const isValidTarget = hoverCategoryId &&
    hoverCategoryId !== pending?.file?.category &&
    !SPECIAL_TARGETS.has(hoverCategoryId)
  dragTargetCategory.value = isValidTarget ? hoverCategoryId : null
}

const onPointerMove = (event) => {
  if (!pending) return
  if (!iconDragState.active) {
    if (Math.hypot(event.clientX - pending.startX, event.clientY - pending.startY) < DRAG_THRESHOLD) {
      return
    }
    iconDragState.active = true
    iconDragState.file = pending.file
    document.body.classList.add('icon-dragging')
  }
  iconDragState.x = event.clientX
  iconDragState.y = event.clientY
  applyHover(event.clientX, event.clientY)
}

const onPointerUp = async () => {
  if (!pending) return
  const file = pending.file
  const dragged = iconDragState.active
  const targetId = hoverCategoryId
  if (dragged) {
    suppressClickUntil = Date.now() + CLICK_SUPPRESS_MS
  }
  reset()
  if (!dragged || !targetId) return

  if (SPECIAL_TARGETS.has(targetId)) {
    ElMessage.warning('特殊分类不支持移动文件，请拖到自定义分类')
    return
  }

  // 开始菜单等特殊分类是自动生成/聚合视图，拖到自定义分类时复制而不是移动，
  // 避免从开始菜单里“消失”（刷新后又会回来）。
  const copyOnly = SPECIAL_TARGETS.has(file.category)
  const result = copyOnly
    ? await copyFileToCategory(file, targetId)
    : await moveFileToCategory(file, targetId)
  const category = customCategories.value.find(c => c.id === targetId)
  if (result.success) {
    ElMessage.success(`${copyOnly ? '已复制' : '已移动'}到「${category?.name || targetId}」`)
  } else if (result.reason === 'duplicate') {
    ElMessage.warning(`「${category?.name || targetId}」中已存在此文件`)
  } else if (result.reason === 'cannot_move_special_category_file') {
    ElMessage.warning('特殊分类中的项目不支持移动')
  }
}

export const startIconDrag = (file, event) => {
  if (!file || event.button !== 0) return
  if (pending) reset()
  pending = {
    file,
    startX: event.clientX,
    startY: event.clientY,
    pointerId: event.pointerId,
    element: event.currentTarget
  }
  window.addEventListener('pointermove', onPointerMove, true)
  window.addEventListener('pointerup', onPointerUp, true)
  window.addEventListener('pointercancel', reset, true)
  window.addEventListener('keydown', onKeyDown, true)
  window.addEventListener('blur', onBlur)
  try {
    pending.element.setPointerCapture(event.pointerId)
  } catch (error) {
    // 合成事件没有可捕获的指针，窗口监听已足够
  }
}

// 拖拽结束后浏览器仍会派发一次 click，用它避免误启动图标
export const shouldSuppressIconClick = () => {
  if (Date.now() < suppressClickUntil) {
    suppressClickUntil = 0
    return true
  }
  return false
}
