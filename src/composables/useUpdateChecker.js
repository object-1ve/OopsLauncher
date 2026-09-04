// 应用更新检查：启动时查询 GitHub Release，发现新版本则弹窗提示。
// 状态为模块级单例（与 useFileState 同模式），由 UpdateDialog.vue 消费。
import { ref } from "vue";
import { check } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { isTauri } from "@/utils/env";

// 模块级状态
const dialogVisible = ref(false);
// idle | confirm | downloading | done | error
const phase = ref("idle");
const newVersion = ref("");
const releaseNotes = ref("");
const progress = ref(0);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const checking = ref(false);

let currentUpdate = null;
let checkedOnce = false;

export function useUpdateChecker() {
  // 弹出更新确认框（自动 / 手动共用）
  const showUpdateDialog = (update) => {
    currentUpdate = update;
    newVersion.value = update.version;
    releaseNotes.value = update.body || "";
    phase.value = "confirm";
    dialogVisible.value = true;
  };

  /**
   * 启动时检查是否有新版本，仅执行一次；失败时静默（无 latest.json 或网络异常不打扰用户）
   */
  const checkForUpdate = async () => {
    if (!isTauri() || checkedOnce) return;
    checkedOnce = true;
    try {
      const update = await check();
      if (!update) return;
      showUpdateDialog(update);
    } catch (err) {
      // 检查失败不打扰用户，仅记录日志
      console.error("检查更新失败:", err);
    }
  };

  /**
   * 设置页手动触发：不受 checkedOnce 限制，有明确的成功 / 无更新 / 失败反馈
   */
  const manualCheck = async () => {
    if (!isTauri()) {
      ElMessage.info("当前为浏览器预览环境，不支持检查更新");
      return;
    }
    if (checking.value) return;
    checking.value = true;
    checkedOnce = true;
    try {
      const update = await check();
      if (!update) {
        ElMessage.success("当前已是最新版本");
        return;
      }
      showUpdateDialog(update);
    } catch (err) {
      console.error("手动检查更新失败:", err);
      ElMessage.error("检查更新失败，请稍后重试");
    } finally {
      checking.value = false;
    }
  };

  /**
   * 下载并安装更新；完成后退出应用，由安装器接管安装流程
   */
  const downloadAndInstall = async () => {
    if (!currentUpdate || phase.value !== "confirm") return;
    phase.value = "downloading";
    progress.value = 0;
    downloadedBytes.value = 0;
    totalBytes.value = 0;
    try {
      await currentUpdate.downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalBytes.value = event.data.contentLength || 0;
        } else if (event.event === "Progress") {
          downloadedBytes.value += event.data.chunkLength;
          if (totalBytes.value > 0) {
            progress.value = Math.min(
              100,
              Math.round((downloadedBytes.value / totalBytes.value) * 100)
            );
          }
        }
      });
      phase.value = "done";
      // 退出应用，NSIS/WiX 安装器在应用退出后完成安装
      await invoke("exit_app");
    } catch (err) {
      console.error("更新失败:", err);
      phase.value = "error";
    }
  };

  /**
   * 关闭弹窗并重置状态
   */
  const closeDialog = () => {
    dialogVisible.value = false;
    phase.value = "idle";
    currentUpdate = null;
  };

  return {
    dialogVisible,
    phase,
    newVersion,
    releaseNotes,
    progress,
    totalBytes,
    checking,
    checkForUpdate,
    manualCheck,
    downloadAndInstall,
    closeDialog,
  };
}
