<script setup>
import Sidebar from "@/components/Sidebar.vue";
import { useFiles } from "@/composables/useFiles";
import TitleBar from "@/components/TitleBar.vue";
const { currentCategory, switchCategory } = useFiles();
</script>

<template>
  <div class="layout-wrapper">
    <TitleBar />
    <div class="basic-layout">
      <Sidebar
        class="page-side"
        :currentCategory="currentCategory"
        @categoryChange="switchCategory"
      />
      <div class="page-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.layout-wrapper {
  height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  border: 1px solid #dcdfe6;
  box-sizing: border-box;
}

.basic-layout {
  display: grid;
  grid-template-columns: 150px 1fr;
  flex: 1;
  width: 100%;
  overflow: hidden;
}

.page-content {
  height: 100%;
  overflow: hidden;
  position: relative;
}
</style>
