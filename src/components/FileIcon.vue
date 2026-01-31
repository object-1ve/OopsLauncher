<template>
  <div :class="settings.appearance.itemLayout">
    <div
      class="icon-item"
      :style="
        settings.appearance.itemLayout === 'tile'
          ? {
              width: settings.appearance.iconSize * 1.6 + 'px',
              height: settings.appearance.iconSize * 2 + 'px',
            }
          : {
              width: '100%',
              height: settings.appearance.iconSize + 16 + 'px',
            }
      "
      :title="`名称: ${file.name}\n打开次数: ${file.openCount || 0}\n位置: ${
        file.path || '未知'
      }`"
      @click="handleFileClick"
      @contextmenu="handleContextMenu"
    >
      <!-- 显示真实图标或emoji -->
      <div
        class="icon-wrapper"
        :style="{
          width: settings.appearance.iconSize + 'px',
          height: settings.appearance.iconSize + 'px',
        }"
      >
        <img
          v-if="file.icon && file.icon.startsWith('data:image/')"
          :src="file.icon"
          :alt="file.name"
          class="file-icon-img"
        />
        <div
          v-else
          class="file-icon-emoji"
          :style="{ fontSize: settings.appearance.iconSize + 'px' }"
        >
          {{ file.icon || "📄" }}
        </div>
      </div>

      <div class="file-name">
        {{ file.displayName || getFileNameWithoutExtension(file.name) }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { useSettings } from "@/composables/useSettings";

const { settings } = useSettings();

// Props
const props = defineProps({
  file: {
    type: Object,
    required: true,
  },
});

// Emits
const emit = defineEmits(["open", "delete", "contextmenu"]);

// 方法：获取不带后缀的文件名
const getFileNameWithoutExtension = (fileName) => {
  const lastDotIndex = fileName.lastIndexOf(".");
  return lastDotIndex > 0 ? fileName.substring(0, lastDotIndex) : fileName;
};

// 方法：处理文件点击
const handleFileClick = () => {
  emit("open", props.file);
};

// 方法：处理右键菜单
const handleContextMenu = (e) => {
  e.preventDefault();
  e.stopPropagation(); // 阻止冒泡，避免触发空白区右键菜单
  emit("contextmenu", { event: e, fileId: props.file.id });
};
</script>

<style scoped>
.icon-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: v-bind("settings.appearance.css.borderRadius");
  cursor: pointer;
  transition: all 0.2s ease;
  margin: v-bind("settings.appearance.css.itemMargin");
  padding: v-bind("settings.appearance.css.itemPadding");
}

.list .icon-item {
  flex-direction: row;
  justify-content: flex-start;
  padding: 0 v-bind("settings.appearance.css.itemPadding");
}

.icon-item:hover {
  background-color: v-bind("settings.appearance.css.hoverColor");
  height: 60px;
}

.icon-wrapper {
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.list .icon-wrapper {
  margin-bottom: 0;
  margin-right: 12px;
}

.file-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background-color: transparent; /* 确保背景透明 */
  mix-blend-mode: multiply; /* 可选：如果是白色背景的jpg，可以尝试混合 */
  margin-top: 5px;
}

.file-icon-emoji {
  font-size: 48px;
  line-height: 1;
}

.file-name {
  font-size: v-bind("settings.appearance.css.fontSize");
  color: v-bind("settings.appearance.css.textColor");
  text-align: center;
  width: 100%;

  /* 单行显示逻辑 */
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap; /* 禁止换行 */
  line-height: v-bind("settings.appearance.css.lineHeight"); /* 优化行高 */
  height: v-bind("settings.appearance.css.lineHeight"); /* 固定高度，只显示一行 */
}

.list .file-name {
  text-align: left;
  height: v-bind("settings.appearance.css.lineHeight");
}

.icon-item:hover {
  opacity: 1;
}
</style>
