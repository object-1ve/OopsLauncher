<template>
  <div class="sidebar-item-container">
    <div
      class="menu-item"
      :class="{
        active: currentCategory === item.name,
        'is-editing': isEditing,
      }"
      @click="handleSelect"
      @contextmenu.prevent="handleContextMenu"
    >
      <div class="item-content">
        <template v-if="isEditing">
          <el-input
            v-model="editName"
            size="small"
            class="edit-input"
            @keyup.enter="saveRename"
            @blur="cancelRename"
            ref="editInputRef"
            @click.stop
          />
        </template>
        <span v-else class="item-name">{{ item.name }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick } from "vue";
import { useFiles } from "@/composables/useFiles";

const props = defineProps({
  item: Object,
  currentCategory: String,
});

const emit = defineEmits(["categoryChange", "itemContextMenu"]);

const { renameCategory } = useFiles();

const isEditing = ref(false);
const editName = ref(props.item.name);
const editInputRef = ref(null);

const handleSelect = () => {
  if (!isEditing.value) {
    emit("categoryChange", props.item.name);
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
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px 0 20px;
  font-size: 14px;
  color: #303133;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.menu-item:hover {
  background-color: #f0f0f0;
}

.menu-item.active {
  background-color: #ececec;
  border-right: 3px solid #409eff;
  font-size: 14px;
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
  text-overflow: ellipsis;
}
</style>
