
// 全局状态，存储当前活动的菜单关闭函数
let activeMenuClose = null;

export function useContextMenu() {
  /**
   * 注册当前打开的菜单
   * @param {Function} closeFn 关闭当前菜单的回调函数
   */
  const registerMenu = (closeFn) => {
    // 如果有其他菜单打开，先关闭它
    if (activeMenuClose && activeMenuClose !== closeFn) {
      activeMenuClose();
    }
    // 记录当前的关闭函数
    activeMenuClose = closeFn;
  };

  /**
   * 清除当前菜单记录（当菜单被手动关闭时调用）
   */
  const unregisterMenu = () => {
    activeMenuClose = null;
  };

  return {
    registerMenu,
    unregisterMenu
  };
}
