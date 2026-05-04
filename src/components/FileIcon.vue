<template>
  <div :class="settings.appearance.itemLayout">
    <div
      class="icon-item"
      :data-file-id="file.id"
      :style="
        settings.appearance.itemLayout === 'tile'
          ? {
              width: settings.appearance.iconSize * 1.6 + 'px',
            }
          : {
              width: '100%',
              height: settings.appearance.iconSize + 20 + 'px',
            }
      "
      :title="fileHoverTitle"
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

      <div class="file-name" v-if="settings.appearance.showFileName">
        {{ file.displayName || getFileNameWithoutExtension(file.name) }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { useSettings } from "@/composables/useSettings";
import { useFiles, SPECIAL_CATEGORIES } from "@/composables/useFiles";

const { settings } = useSettings();
const { currentCategory, allCategories } = useFiles();

// Props
const props = defineProps({
  file: {
    type: Object,
    required: true,
  },
});

const sourceCategoryName = computed(() => {
  const categoryId = props.file?.category;
  if (!categoryId) return "未知";
  const matchedCategory = allCategories.value.find((cat) => cat.id === categoryId);
  return matchedCategory?.name || categoryId;
});

const fileHoverTitle = computed(() => {
  const baseLines = [
    `名称: ${props.file.displayName || props.file.name || "未知"}`,
    `打开次数: ${props.file.openCount || 0}`,
    `位置: ${props.file.path || "未知"}`,
  ];

  if (currentCategory.value === SPECIAL_CATEGORIES.ALL_FILES) {
    baseLines.push(`来源分类: ${sourceCategoryName.value}`);
  }

  return baseLines.join("\n");
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
  justify-content: flex-start;
  border-radius: v-bind("settings.appearance.css.borderRadius");
  cursor: pointer;
  transition: all 0.2s ease;
  margin: v-bind("settings.appearance.css.itemMargin");
  padding: calc(v-bind("settings.appearance.css.itemPadding") / 2);
}

.list .icon-item {
  flex-direction: row;
  justify-content: flex-start;
  padding: 0 calc(v-bind("settings.appearance.css.itemPadding") / 2);
}

.icon-item:hover {
  background-color: v-bind("settings.appearance.css.hoverColor");
  opacity: 1;
}

.icon-item.locating-highlight {
  outline: 2px solid #409eff;
  outline-offset: -2px;
  background-color: rgba(64, 158, 255, 0.12);
}

.icon-wrapper {
  margin-bottom: 4px;
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
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: v-bind("settings.appearance.css.lineHeight");
  max-height: calc(v-bind("settings.appearance.css.lineHeight") * 2em);
  word-break: break-all;
  padding: 0 4px;
  box-sizing: border-box;
}

.list .file-name {
  text-align: left;
  max-height: calc(v-bind("settings.appearance.css.lineHeight") * 2em);
}
</style>
