<template>
  <el-dialog
    :model-value="dialogVisible"
    title="发现新版本"
    width="440px"
    align-center
    :close-on-click-modal="false"
    :close-on-press-escape="phase !== 'downloading'"
    :show-close="phase !== 'downloading'"
    @update:model-value="onVisibleChange"
  >
    <!-- 确认阶段 -->
    <div v-if="phase === 'confirm'" class="update-confirm">
      <p class="update-tip">
        检测到新版本 <b>v{{ newVersion }}</b>，是否立即更新？
      </p>
      <div v-if="releaseNotes" class="release-notes">
        <p class="release-notes-title">更新说明</p>
        <div class="release-notes-body">{{ releaseNotes }}</div>
      </div>
    </div>

    <!-- 下载阶段 -->
    <div v-if="phase === 'downloading'" class="update-downloading">
      <el-progress
        :percentage="progress"
        :indeterminate="totalBytes <= 0"
        :duration="2"
        :stroke-width="16"
        :text-inside="true"
      />
      <p class="update-tip">正在下载更新，请稍候…</p>
    </div>

    <!-- 完成阶段 -->
    <div v-if="phase === 'done'" class="update-done">
      <p class="update-tip">更新已下载完成，即将退出应用并安装。</p>
    </div>

    <!-- 失败阶段 -->
    <div v-if="phase === 'error'" class="update-error">
      <p class="update-tip">更新失败，请稍后重试，或前往 GitHub Release 手动下载安装。</p>
    </div>

    <template #footer>
      <el-button v-if="phase === 'confirm'" @click="closeDialog">稍后再说</el-button>
      <el-button v-if="phase === 'confirm'" type="primary" @click="downloadAndInstall">
        立即更新
      </el-button>
      <el-button v-if="phase === 'error'" @click="closeDialog">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { useUpdateChecker } from "@/composables/useUpdateChecker";

const {
  dialogVisible,
  phase,
  newVersion,
  releaseNotes,
  progress,
  totalBytes,
  downloadAndInstall,
  closeDialog,
} = useUpdateChecker();

const onVisibleChange = (visible) => {
  // 下载中不允许关闭弹窗，其余阶段关闭即重置
  if (!visible && phase.value !== "downloading") {
    closeDialog();
  }
};
</script>

<style scoped>
.update-confirm,
.update-downloading,
.update-done,
.update-error {
  text-align: center;
}

.update-tip {
  margin: 8px 0;
  line-height: 1.6;
  color: var(--app-text-color, #303133);
}

.release-notes {
  margin-top: 12px;
  text-align: left;
  border: 1px solid var(--app-border-color, #e4e7ed);
  border-radius: 6px;
  padding: 10px 12px;
  background: var(--app-hover-color, #f5f7fa);
}

.release-notes-title {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-color, #303133);
}

.release-notes-body {
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
  line-height: 1.6;
  color: var(--app-text-secondary, #909399);
}
</style>
