<template>
  <div class="topology-page p-6">
    <div class="mb-4">
      <router-link
        :to="`/node/${nodeId}`"
        class="text-blue-600 hover:text-blue-800"
      >
        ← 返回节点详情
      </router-link>
    </div>
    <h1 class="text-3xl font-bold text-gray-800 mb-6">
      节点 {{ nodeId }} - 拓扑可视化
    </h1>
    <div class="bg-white rounded-lg shadow-md p-4" style="height: calc(100vh - 200px)">
      <TopologyCanvas />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import TopologyCanvas from '@/components/TopologyCanvas.vue';

const route = useRoute();
const nodeStore = useNodeStore();

const nodeId = computed(() => {
  const id = route.params.nodeId;
  return typeof id === 'string' ? parseInt(id, 10) : Number(id);
});

onMounted(() => {
  nodeStore.setCurrentNode(nodeId.value);
});
</script>

<style scoped></style>

