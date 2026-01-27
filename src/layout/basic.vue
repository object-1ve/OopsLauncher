<script setup>
import Sidebar from "@/components/Sidebar.vue";
import { useFiles } from "@/composables/useFiles";
import TitleBar from '@/components/TitleBar.vue'
const { currentCategory, switchCategory } = useFiles();
</script>

<template>
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
</template>

<style scoped lang="scss">
.basic-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
    width: 100%;
}

.page-content {
    height: 100%;
    overflow: hidden;
    position: relative;
}

</style>
