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
        @click.self="handleCloseCreateDialog"
      >
        <div class="bg-white rounded-3xl p-8 w-full max-w-md shadow-2xl animate-slide-up">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-2xl font-bold text-gray-900">Create New Node</h2>
              <p class="text-sm text-gray-500" v-if="creationResult">
                Credentials are shown once. Copy them now to configure节点。
              </p>
            </div>
            <button
              @click="handleCloseCreateDialog"
              class="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <div class="i-carbon-close text-2xl"></div>
            </button>
          </div>

          <div v-if="!creationResult" class="space-y-4">
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

          <div v-else class="space-y-5">
            <div class="rounded-2xl border border-emerald-100 bg-emerald-50/70 p-4 flex items-start gap-3">
              <div class="i-carbon-checkmark-filled text-emerald-500 text-2xl"></div>
              <div>
                <p class="text-sm font-semibold text-emerald-700">Node created successfully</p>
                <p class="text-xs text-emerald-600">Copy the credentials below. They won’t be shown again.</p>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <label class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1">Node ID</label>
                <div class="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl font-mono text-sm flex items-center justify-between">
                  <span>{{ creationResult?.node_id }}</span>
                  <button class="text-primary-500 hover:text-primary-600" @click="copyField('Node ID', creationResult?.node_id?.toString() ?? '')">
                    <div class="i-carbon-copy text-lg"></div>
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1">Node Token</label>
                <div class="credential-tile">
                  <span class="truncate font-mono">{{ creationResult?.node_token }}</span>
                  <button class="btn-link" @click="copyField('Node Token', creationResult?.node_token ?? undefined)">
                    <div class="i-carbon-copy text-lg"></div>
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1">Node Shared Secret</label>
                <div class="credential-tile">
                  <span class="truncate font-mono">{{ creationResult?.node_shared_secret }}</span>
                  <button class="btn-link" @click="copyField('Shared Secret', creationResult?.node_shared_secret ?? undefined)">
                    <div class="i-carbon-copy text-lg"></div>
                  </button>
                </div>
              </div>
            </div>

            <p class="text-xs text-gray-500 leading-relaxed">
              Keep the token and shared secret safe. They are required for节点认证且此处仅展示一次。
            </p>
          </div>

          <div class="flex gap-3 mt-8">
            <button
              @click="handleCloseCreateDialog"
              class="flex-1 btn-secondary py-3"
            >
              {{ creationResult ? 'Done' : 'Cancel' }}
            </button>
            <button
              v-if="!creationResult"
              @click="createNode"
              :disabled="creatingNode"
              class="flex-1 btn-success py-3 flex items-center justify-center gap-2"
            >
              <div v-if="creatingNode" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              <span>{{ creatingNode ? 'Creating...' : 'Create' }}</span>
            </button>
            <button
              v-else
              class="flex-1 btn-primary py-3"
              @click="startAnotherCreation"
            >
              Create Another Node
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
import { useToastStore } from '@/stores/toast';
import type { NodeInfo } from '@/api/types';

const router = useRouter();
const nodeStore = useNodeStore();
const toast = useToastStore();

const filterName = ref('');
const filterRegion = ref('');
const filterDescription = ref('');
const filterOnline = ref<'all' | 'online' | 'offline'>('all');
const showCreateNodeDialog = ref(false);
const creatingNode = ref(false);
const creationResult = ref<NodeInfo | null>(null);

const newNodeForm = ref({
  nodeId: '',
  name: '',
  region: '',
  description: '',
});

const resetCreateForm = () => {
  newNodeForm.value = {
    nodeId: '',
    name: '',
    region: '',
    description: '',
  };
};

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
    toast.warning('Please enter Node ID');
    return;
  }

  creatingNode.value = true;
  try {
    const created = await adminApi.createNode({
      node_id: parseInt(newNodeForm.value.nodeId),
      name: newNodeForm.value.name || undefined,
      region: newNodeForm.value.region || undefined,
      description: newNodeForm.value.description || undefined,
    });

    creationResult.value = created;
    toast.success('Node created. Credentials ready to copy.');
    resetCreateForm();
    nodeStore.fetchNodes(onlineParam.value);
  } catch (err: any) {
    toast.error(err?.message || 'Failed to create node');
  } finally {
    creatingNode.value = false;
  }
};

const handleCloseCreateDialog = () => {
  showCreateNodeDialog.value = false;
  creationResult.value = null;
  resetCreateForm();
};

const startAnotherCreation = () => {
  creationResult.value = null;
};

watch(showCreateNodeDialog, (visible) => {
  if (visible) {
    creationResult.value = null;
    resetCreateForm();
  }
});

const copyField = async (label: string, value?: string) => {
  if (!value) {
    toast.error(`${label} unavailable`);
    return;
  }
  try {
    await navigator.clipboard.writeText(value);
    toast.success(`${label} copied`);
  } catch (error) {
    console.error('copy failed', error);
    toast.error('Copy failed');
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

.credential-tile {
  @apply w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl font-mono text-sm flex items-center justify-between gap-3;
}

.btn-link {
  @apply text-primary-500 hover:text-primary-600 transition-colors;
}
</style>
