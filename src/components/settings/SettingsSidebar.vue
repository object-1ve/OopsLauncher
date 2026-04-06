<template>
  <el-aside width="160px" class="settings-aside">
    <el-menu :default-active="activeTab" class="settings-menu" @select="$emit('select', $event)">
      <el-menu-item index="general">
        <span>常规</span>
      </el-menu-item>
      <el-menu-item index="appearance">
        <span>外观</span>
      </el-menu-item>
      <el-menu-item index="shortcuts">
        <span>快捷键</span>
      </el-menu-item>
      <el-menu-item index="about">
        <span>关于</span>
      </el-menu-item>
    </el-menu>
  </el-aside>
</template>

<script setup>
defineProps({
  activeTab: {
    type: String,
    required: true
  }
});

defineEmits(["select"]);
</script>

<style scoped>
.settings-aside {
  background-color: #fff;
  border-right: 1px solid #e6e6e6;
  display: flex;
  flex-direction: column;
}

.settings-menu {
  border-right: none;
  flex: 1;
}
</style>
