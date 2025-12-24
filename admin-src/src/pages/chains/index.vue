<template>
  <div class="chains-page min-h-screen p-4 md:p-6 lg:p-8">
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 tracking-tight mb-2">
            Chains
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium text-sm">Maintain chain routing tables per node</p>
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/')"
          >
            <div class="i-carbon-arrow-left text-lg"></div>
            <span>BACK TO NODES</span>
          </button>
        </div>
      </div>

      <div class="mt-4 grid grid-cols-1 md:grid-cols-12 gap-3">
        <div class="md:col-span-5">
          <NodeSelector v-model="selectedNodeId" />
        </div>

        <div class="md:col-span-7 flex items-center justify-end text-xs text-gray-400">
          <span v-if="!selectedNodeId">Select a node to edit chains</span>
          <span v-else>Editing node_id: <span class="font-mono">{{ selectedNodeId }}</span></span>
        </div>
      </div>
    </div>

    <div v-if="!selectedNodeId" class="flex flex-col items-center justify-center py-20 animate-fade-in">
      <div class="w-28 h-28 bg-gray-50 rounded-full flex items-center justify-center mb-6 border border-gray-100">
        <div class="i-carbon-direction-fork text-4xl text-gray-300"></div>
      </div>
      <h3 class="text-xl font-bold text-gray-700 mb-2">Choose a node</h3>
      <p class="text-gray-400">Chains are stored per node. Pick one to start editing.</p>
    </div>

    <ChainManagement v-else :node-id="selectedNodeId" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import ChainManagement from '@/components/ChainManagement.vue';
import NodeSelector from '@/components/NodeSelector.vue';

const router = useRouter();
const nodeStore = useNodeStore();

const selectedNodeId = ref<number | null>(null);

const ensureDefaultNode = () => {
  if (nodeStore.sortedNodes.length > 0 && selectedNodeId.value === null) {
    selectedNodeId.value = nodeStore.sortedNodes[0].node_id;
  }
};

onMounted(async () => {
  await nodeStore.fetchNodes();
  ensureDefaultNode();
});

watch(
  () => nodeStore.sortedNodes.length,
  () => {
    ensureDefaultNode();
  }
);
</script>
