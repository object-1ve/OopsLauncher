<template>
  <div class="settings-section">
    <h2 class="section-title">快捷键设置</h2>
    <el-form label-position="top">
      <el-form-item label="显示/隐藏主窗口">
        <el-input
          v-model="settings.shortcuts.showHide"
          placeholder="点击设置快捷键"
          readonly
          @keydown.prevent="onKeyDown($event, 'showHide')"
        />
        <div class="shortcut-tip">按下想要设置的组合键</div>
      </el-form-item>
      <el-form-item label="复制当前时间到剪切板">
        <el-input
          v-model="settings.shortcuts.copyTime"
          placeholder="点击设置快捷键"
          readonly
          @keydown.prevent="onKeyDown($event, 'copyTime')"
        />
        <div class="shortcut-tip">按下想要设置的组合键</div>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup>
defineProps({
  settings: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(["shortcutKeyDown"]);

const onKeyDown = (event, keyType) => {
  emit("shortcutKeyDown", event, keyType);
};
</script>

<style scoped>
.settings-section {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.section-title {
  margin-bottom: 30px;
  font-size: 22px;
  font-weight: 600;
  color: #303133;
}

.shortcut-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
}

:deep(.el-form-item__label) {
  font-weight: 600;
  padding-bottom: 8px;
}
</style>
