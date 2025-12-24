<template>
  <div class="chains-page min-h-screen p-4 md:p-6 lg:p-8">
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 tracking-tight mb-2">
            Chains
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium text-sm">Maintain global chain templates and apply them to nodes</p>
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

      <div class="mt-4 flex items-center justify-end text-xs text-gray-400">
        <span>Global templates (node_id: <span class="font-mono">0</span>)</span>
      </div>
    </div>

    <ChainManagement :node-id="0" />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import ChainManagement from '@/components/ChainManagement.vue';

const router = useRouter();
const nodeStore = useNodeStore();

onMounted(async () => {
  await nodeStore.fetchNodes();
});
</script>
