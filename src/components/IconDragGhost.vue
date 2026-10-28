<template>
  <div
    v-if="iconDragState.active"
    class="icon-drag-ghost"
    :style="{ left: iconDragState.x + 'px', top: iconDragState.y + 'px' }"
  >
    <img
      v-if="isImageIcon"
      :src="iconDragState.file.icon"
      alt=""
      class="ghost-icon"
      draggable="false"
    />
    <span v-else class="ghost-emoji">{{ iconDragState.file.icon || "📄" }}</span>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { iconDragState } from "@/composables/useIconDrag";

const isImageIcon = computed(() =>
  iconDragState.file?.icon?.startsWith("data:image/")
);
</script>

<style scoped>
.icon-drag-ghost {
  position: fixed;
  z-index: 3000;
  transform: translate(-50%, -50%);
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.88);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
  opacity: 0.9;
}

.ghost-icon {
  width: 40px;
  height: 40px;
  object-fit: contain;
}

.ghost-emoji {
  font-size: 34px;
  line-height: 1;
}
</style>
