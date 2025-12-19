<template>
  <div class="index-page min-h-screen p-6 md:p-10 lg:p-14">
    <!-- 页面头部 -->
    <div class="mb-12 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-4xl font-bold text-gray-900 tracking-tight mb-3">
            Nodes
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium">Manage and monitor your infrastructure</p>
        </div>
        <button
          @click="refreshNodes"
          :disabled="nodeStore.loading"
          class="btn-primary flex items-center gap-2.5 px-6 py-2.5"
        >
          <div :class="nodeStore.loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
          <span class="text-sm font-semibold tracking-wide">REFRESH</span>
        </button>
      </div>

      <div class="mt-8 grid grid-cols-1 md:grid-cols-4 gap-4">
        <div class="relative">
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-search"></div>
          </div>
          <input
            v-model="filterName"
            type="text"
            placeholder="Filter by name"
            class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
          />
        </div>
        <div class="relative">
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-earth"></div>
          </div>
          <input
            v-model="filterRegion"
            type="text"
            placeholder="Filter by region"
            class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
          />
        </div>
        <div class="relative">
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-document"></div>
          </div>
          <input
            v-model="filterDescription"
            type="text"
            placeholder="Filter by description"
            class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
          />
        </div>

        <div>
          <BaseSelect
            v-model="filterOnline"
            :options="onlineOptions"
            placeholder="All status"
          >
            <template #icon>
              <div class="i-carbon-signal-strength text-lg"></div>
            </template>
          </BaseSelect>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <transition name="fade">
      <div
        v-if="nodeStore.error"
        class="mb-8 p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-2xl shadow-sm animate-slide-up flex items-center gap-3"
      >
        <div class="i-carbon-warning-alt text-xl"></div>
        <span class="font-medium">{{ nodeStore.error }}</span>
      </div>
    </transition>

    <!-- 加载状态 -->
    <div
      v-if="nodeStore.loading && nodeStore.nodes.length === 0"
      class="flex flex-col items-center justify-center py-32 animate-fade-in"
    >
      <div class="loading-ring w-12 h-12 relative mb-6"></div>
      <p class="text-gray-400 font-medium tracking-wide">SYNCING DATA...</p>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="nodeStore.sortedNodes.length === 0"
      class="flex flex-col items-center justify-center py-32 animate-fade-in"
    >
      <div class="w-32 h-32 bg-gray-50 rounded-full flex items-center justify-center mb-6 border border-gray-100">
        <div class="i-carbon-cloud-satellite text-4xl text-gray-300"></div>
      </div>
      <h3 class="text-xl font-bold text-gray-700 mb-2">No Nodes Found</h3>
      <p class="text-gray-400">Add a new node to get started</p>
    </div>

    <!-- 节点网格 -->
    <div
      v-else
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8"
    >
      <transition-group name="list" tag="div" class="contents">
        <NodeCard
          v-for="(node, index) in filteredNodes"
          :key="node.node_id"
          :node="node"
          :style="{ 'animation-delay': `${index * 50}ms` }"
          @click="goToNodeDetail(node.node_id)"
          class="animate-slide-up"
        />
      </transition-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import NodeCard from '@/components/NodeCard.vue';
import BaseSelect from '@/components/BaseSelect.vue';

const router = useRouter();
const nodeStore = useNodeStore();

const filterName = ref('');
const filterRegion = ref('');
const filterDescription = ref('');
const filterOnline = ref<'all' | 'online' | 'offline'>('all');

const onlineOptions = [
  { label: 'All status', value: 'all' },
  { label: 'Online', value: 'online' },
  { label: 'Offline', value: 'offline' },
];

const onlineParam = computed(() => {
  if (filterOnline.value === 'online') return true;
  if (filterOnline.value === 'offline') return false;
  return undefined;
});

const filteredNodes = computed(() => {
  const nameQ = filterName.value.trim().toLowerCase();
  const regionQ = filterRegion.value.trim().toLowerCase();
  const descQ = filterDescription.value.trim().toLowerCase();

  return nodeStore.sortedNodes.filter((n) => {
    const name = (n.name || '').toLowerCase();
    const region = (n.region || '').toLowerCase();
    const description = (n.description || '').toLowerCase();

    if (nameQ && !name.includes(nameQ)) return false;
    if (regionQ && !region.includes(regionQ)) return false;
    if (descQ && !description.includes(descQ)) return false;
    return true;
  });
});

onMounted(() => {
  nodeStore.fetchNodes(onlineParam.value);
});

watch(
  () => filterOnline.value,
  () => {
    nodeStore.fetchNodes(onlineParam.value);
  }
);

const refreshNodes = () => {
  nodeStore.fetchNodes(onlineParam.value);
};

const goToNodeDetail = (nodeId: number) => {
  router.push(`/node/${nodeId}`);
};
</script>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.4s var(--ease-spring);
}

.list-enter-from {
  opacity: 0;
  transform: translateY(30px) scale(0.95);
}

.list-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
</style>
