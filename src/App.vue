<template>
  <div id="app">
    <!-- 自定义标题栏 -->

    <!-- 路由视图 -->
    <div class="app-container">
      <router-view />
    </div>
  </div>
</template>

<script setup>

</script>

<style scoped>
/* 主容器 */
#app {
  height: 100vh;
  width: 100%;
  box-sizing: border-box;
  background-color: #fff;
}

/* 应用容器 */
.app-container {
  height: 100vh;
  width: 100%;
}
</style>
