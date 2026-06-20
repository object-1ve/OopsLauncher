<template>
  <div class="sidebar-item-container">
    <div class="menu-item" :data-drop-category="item.id" :class="{
      active: currentCategory === item.id,
      'is-editing': isEditing,
      'is-dragover': isDragOver || dragTargetCategory === item.id
    }" @click="handleSelect" @contextmenu.prevent="handleContextMenu" @dragover.prevent="handleDragOver"
      @dragleave="handleDragLeave" @drop="handleDrop">
      <div class="item-content">
        <template v-if="isEditing">
          <el-input v-model="editName" size="small" class="edit-input" @keyup.enter="saveRename" @blur="cancelRename"
            ref="editInputRef" @click.stop />
        </template>
        <span v-else class="item-name">{{ item.name }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick } from "vue";
import { useFiles } from "@/composables/useFiles";
import { ElMessage } from "element-plus";

const props = defineProps({
  item: Object,
  currentCategory: String,
});

const emit = defineEmits(["categoryChange", "itemContextMenu"]);

const { renameCategory, processFiles, dragTargetCategory } = useFiles();

const isEditing = ref(false);
const isDragOver = ref(false);
const editName = ref(props.item.name);
const editInputRef = ref(null);

const handleDragOver = (e) => {
  e.preventDefault();
  isDragOver.value = true;
  dragTargetCategory.value = props.item.id;
};

const handleDragLeave = () => {
  isDragOver.value = false;
  dragTargetCategory.value = null;
};

const handleDrop = async (e) => {
  e.preventDefault();
  e.stopPropagation();
  isDragOver.value = false;
  dragTargetCategory.value = null;

  // 1. 处理来自本程序资源管理器的内部拖拽
  const internalData = e.dataTransfer.getData('application/x-oops-file');
  if (internalData) {
    try {
      const file = JSON.parse(internalData);
      // 临时修改当前分类以确保 processFiles 添加到目标分类
      // 实际上更好的做法是让 processFiles 接受一个可选的 targetCategory 参数
      // 但为了不改动太多 processFiles，我们先简单处理
      await processFilesIntoCategory([file], props.item.id);
      return;
    } catch (err) {
      console.error('Failed to process internal drop:', err);
    }
  }

  // 2. 处理来自系统资源管理器的外部拖拽 (针对非 Tauri 环境的降级处理)
  // 在 Tauri 环境下，系统拖放通常由全局监听器处理，但如果用户正好放在了分类上，
  // 我们也可以在这里拦截并定向到该分类。
  const files = Array.from(e.dataTransfer.files);
  if (files.length > 0) {
    await processFilesIntoCategory(files, props.item.id);
  }
};

// 辅助函数：将文件处理到指定分类
const processFilesIntoCategory = async (files, categoryId) => {
  const result = await processFiles(files, categoryId);

  if (result.addedCount > 0) {
    const msg = `成功添加 ${result.addedCount} 个文件到 「${props.item.name}」`;
    ElMessage.success(msg);
  } else if (result.existingCount > 0) {
    ElMessage.warning(`文件已存在于 「${props.item.name}」`);
  } else if (result.failedCount > 0) {
    ElMessage.error(`添加失败`);
  }
};

const handleSelect = () => {
  if (!isEditing.value) {
    emit("categoryChange", props.item.id);
  }
};

const handleContextMenu = (e) => {
  emit("itemContextMenu", { event: e, item: props.item, startRename });
};

const startRename = () => {
  editName.value = props.item.name;
  isEditing.value = true;
  nextTick(() => {
    editInputRef.value?.focus();
  });
};

const saveRename = async () => {
  if (editName.value.trim() && editName.value !== props.item.name) {
    await renameCategory(props.item.id, editName.value.trim());
  }
  isEditing.value = false;
};

const cancelRename = () => {
  isEditing.value = false;
};

defineExpose({
  startRename,
});
</script>

<style scoped>
.menu-item {
  min-height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 20px;
  font-size: 14px;
  color: #303133;
  cursor: pointer;
  transition: all 0.2s;
}

.menu-item:hover {
  background-color: #f0f0f0;
}

.menu-item.active {
  background-color: #ececec;
  border-right: 3px solid #409eff;
  font-size: 14px;
}

.menu-item.is-dragover {
  background-color: #ecf5ff;
  border-right: 3px solid #409eff;
  box-shadow: inset 0 0 0 1px #409eff;
}

.item-content {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
  flex: 1;
}

.edit-input {
  margin-right: 10px;
}

.item-name {
  overflow: hidden;
  white-space: normal;
  line-height: 1.3;
  word-break: break-all;
}
</style>
