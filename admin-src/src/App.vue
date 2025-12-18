<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'App',
});
</script>
<template>
  <div class="app min-h-screen text-body selection:bg-primary-100 selection:text-primary-700">
    <router-view v-slot="{ Component, route }">
      <transition
        name="page"
        mode="out-in"
        appear
      >
        <component :is="Component" :key="route.path" />
      </transition>
    </router-view>
  </div>
</template>

<style>
/* 页面过渡动画 */
.page-enter-active,
.page-leave-active {
  transition: opacity var(--transition-base) var(--ease-smooth),
              transform var(--transition-base) var(--ease-smooth);
}

.page-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
