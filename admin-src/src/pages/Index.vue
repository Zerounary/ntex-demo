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
        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/flow')"
          >
            <div class="i-carbon-direction-right-01 text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">FLOW BUILDER</span>
          </button>
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/chains')"
          >
            <div class="i-carbon-direction-fork text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">CHAINS</span>
          </button>
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/games')"
          >
            <div class="i-carbon-game-console text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">GAME CONFIG</span>
          </button>
          <button
            @click="refreshNodes"
            :disabled="nodeStore.loading"
            class="btn-primary flex items-center gap-2.5 px-6 py-2.5"
          >
            <div :class="nodeStore.loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">REFRESH</span>
          </button>
          <button
            @click="showCreateNodeDialog = true"
            class="btn-secondary flex items-center gap-2.5 px-6 py-2.5"
          >
            <div class="i-carbon-add text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">NEW NODE</span>
          </button>
        </div>
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

    <!-- 新增节点对话框 -->
    <transition name="fade">
      <div
        v-if="showCreateNodeDialog"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click.self="showCreateNodeDialog = false"
      >
        <div class="bg-white rounded-3xl p-8 w-full max-w-md shadow-2xl animate-slide-up">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl font-bold text-gray-900">Create New Node</h2>
            <button
              @click="showCreateNodeDialog = false"
              class="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <div class="i-carbon-close text-2xl"></div>
            </button>
          </div>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Node ID</label>
              <input
                v-model="newNodeForm.nodeId"
                type="number"
                placeholder="Enter node ID"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Name</label>
              <input
                v-model="newNodeForm.name"
                type="text"
                placeholder="Enter node name"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Region</label>
              <input
                v-model="newNodeForm.region"
                type="text"
                placeholder="Enter region"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Description</label>
              <textarea
                v-model="newNodeForm.description"
                placeholder="Enter description"
                rows="3"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all resize-none"
              />
            </div>
          </div>

          <div class="flex gap-3 mt-8">
            <button
              @click="showCreateNodeDialog = false"
              class="flex-1 btn-secondary py-3"
            >
              Cancel
            </button>
            <button
              @click="createNode"
              :disabled="creatingNode"
              class="flex-1 btn-success py-3 flex items-center justify-center gap-2"
            >
              <div v-if="creatingNode" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              <span>{{ creatingNode ? 'Creating...' : 'Create' }}</span>
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import NodeCard from '@/components/NodeCard.vue';
import BaseSelect from '@/components/BaseSelect.vue';
import * as adminApi from '@/api/admin';

const router = useRouter();
const nodeStore = useNodeStore();

const filterName = ref('');
const filterRegion = ref('');
const filterDescription = ref('');
const filterOnline = ref<'all' | 'online' | 'offline'>('all');
const showCreateNodeDialog = ref(false);
const creatingNode = ref(false);

const newNodeForm = ref({
  nodeId: '',
  name: '',
  region: '',
  description: '',
});

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

const createNode = async () => {
  if (!newNodeForm.value.nodeId) {
    alert('Please enter Node ID');
    return;
  }

  creatingNode.value = true;
  try {
    await adminApi.createNode({
      node_id: parseInt(newNodeForm.value.nodeId),
      name: newNodeForm.value.name || undefined,
      region: newNodeForm.value.region || undefined,
      description: newNodeForm.value.description || undefined,
    });
    
    showCreateNodeDialog.value = false;
    newNodeForm.value = {
      nodeId: '',
      name: '',
      region: '',
      description: '',
    };
    
    nodeStore.fetchNodes(onlineParam.value);
  } catch (err: any) {
    alert(`Failed to create node: ${err.message}`);
  } finally {
    creatingNode.value = false;
  }
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
