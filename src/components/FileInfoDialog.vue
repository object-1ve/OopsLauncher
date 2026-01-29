<template>
  <!-- 文件信息编辑弹窗 -->
  <el-dialog
    v-model="dialogVisible"
    title="编辑文件信息"
    width="500px"
    :close-on-click-modal="false"
    @close="handleCancel"
  >
    <div class="file-info-form">
      <el-form label-width="80px">
        <!-- 显示名称 -->
        <el-form-item label="显示名称">
          <el-input
            v-model="editForm.displayName"
            placeholder="请输入显示名称"
          />
        </el-form-item>
        
        <!-- 文件名称 -->
        <el-form-item label="文件名称">
          <el-input
            v-model="editForm.name"
            placeholder="请输入文件名称"
            disabled
          />
        </el-form-item>
        
        <!-- 文件路径 -->
        <el-form-item label="文件路径">
          <el-input
            v-model="editForm.path"
            placeholder="文件路径"
            disabled
            type="textarea"
            :rows="3"
          />
        </el-form-item>
        
        <!-- 文件大小 -->
        <el-form-item label="文件大小">
          <el-input
            v-model="editForm.size"
            placeholder="文件大小"
            disabled
          />
        </el-form-item>
        
        <!-- 文件类型 -->
        <el-form-item label="文件类型">
          <el-input
            v-model="editForm.type"
            placeholder="文件类型"
            disabled
          />
        </el-form-item>
        
        <!-- 打开次数 -->
        <el-form-item label="打开次数">
          <el-input
            v-model="editForm.openCount"
            placeholder="打开次数"
            disabled
          />
        </el-form-item>
      </el-form>
    </div>
    
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button type="primary" @click="handleSave">保存</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, computed, watch } from "vue";

// Props
const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  currentFile: {
    type: Object,
    default: null,
  },
});

// Emits
const emit = defineEmits(["update:visible", "save", "cancel"]);

// 计算属性：控制弹窗显示状态
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit("update:visible", value),
});

// 编辑表单
const editForm = ref({
  displayName: "",
  name: "",
  path: "",
  size: "",
  type: "",
  openCount: 0,
});

// 监听 currentFile 变化，更新表单数据
watch(() => props.currentFile, (newFile) => {
  if (newFile) {
    editForm.value = {
      displayName: newFile.displayName || newFile.name || "",
      name: newFile.name || "",
      path: newFile.path || "",
      size: formatFileSize(newFile.size || 0),
      type: newFile.type || getFileType(newFile.name || ""),
      openCount: newFile.openCount || 0,
    };
  }
}, { immediate: true });

// 格式化文件大小
const formatFileSize = (bytes) => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

// 获取文件类型
const getFileType = (fileName) => {
  if (!fileName) return "";
  const lastDotIndex = fileName.lastIndexOf(".");
  if (lastDotIndex > 0) {
    return fileName.substring(lastDotIndex + 1).toLowerCase();
  }
  return "";
};

// 方法：处理保存
const handleSave = () => {
  if (props.currentFile) {
    const updatedFile = {
      ...props.currentFile,
      displayName: editForm.value.displayName || props.currentFile.name,
    };
    emit("save", updatedFile);
    emit("update:visible", false);
  }
};

// 方法：处理取消
const handleCancel = () => {
  emit("cancel");
  emit("update:visible", false);
};
</script>

<style scoped>
.file-info-form {
  max-height: 400px;
  overflow-y: auto;
}

.dialog-footer {
  width: 100%;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
