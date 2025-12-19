<template>
  <div class="topology-page min-h-screen p-6 md:p-10 lg:p-14">
    <div class="mb-8 animate-slide-up">
      <router-link
        :to="`/node/${nodeId}`"
        class="inline-flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 transition-colors group font-medium"
      >
        <div class="i-carbon-arrow-left group-hover:-translate-x-1 transition-transform"></div>
        <span>Back to Node</span>
      </router-link>
    </div>

    <div v-if="loading" class="flex flex-col items-center justify-center py-32 animate-fade-in">
      <div class="loading-ring w-12 h-12 relative mb-6"></div>
      <p class="text-gray-400 font-medium tracking-wide">LOADING TOPOLOGY...</p>
    </div>

    <div v-else class="space-y-6 animate-fade-in">
      <div class="flex items-start justify-between gap-6">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 tracking-tight">
            Topology
          </h1>
          <p class="text-gray-500 mt-1">
            Node {{ nodeId }} network view
          </p>
        </div>
      </div>

      <div class="card-base p-4 md:p-6" style="height: calc(100vh - 240px)">
        <TopologyCanvas />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import { useAdminStore } from '@/stores/admin';
import TopologyCanvas from '@/components/TopologyCanvas.vue';

const route = useRoute();
const nodeStore = useNodeStore();
const adminStore = useAdminStore();

const loading = ref(true);

const nodeId = computed(() => {
  const id = route.params.nodeId;
  return typeof id === 'string' ? parseInt(id, 10) : Number(id);
});

const init = async () => {
  loading.value = true;
  try {
    if (Number.isNaN(nodeId.value)) return;
    if (nodeStore.nodes.length === 0) {
      await nodeStore.fetchNodes();
    }
    adminStore.reset();
    nodeStore.setCurrentNode(nodeId.value);
  } finally {
    loading.value = false;
  }
};

watch(
  () => nodeId.value,
  () => {
    init();
  },
  { immediate: true }
);
</script>

<style scoped></style>

