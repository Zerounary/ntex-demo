<template>
  <div class="node-detail-page min-h-screen p-6 md:p-10 lg:p-14">
    <!-- 返回按钮 -->
    <div class="mb-8 animate-slide-up">
      <router-link
        to="/"
        class="inline-flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 transition-colors group font-medium"
      >
        <div class="i-carbon-arrow-left group-hover:-translate-x-1 transition-transform"></div>
        <span>Back to Nodes</span>
      </router-link>
    </div>

    <div v-if="!nodeStore.currentNode" class="flex flex-col items-center justify-center py-32 animate-fade-in">
      <div class="loading-ring w-12 h-12 relative mb-6"></div>
      <p class="text-gray-400 font-medium tracking-wide">LOADING DATA...</p>
    </div>

    <div v-else class="space-y-8 animate-fade-in">
      <!-- 页面头部 -->
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-3xl font-bold text-gray-900 tracking-tight">
              {{ nodeStore.currentNode.name || `Node ${nodeStore.currentNode.node_id}` }}
            </h1>
            <div class="flex items-center gap-2">
              <span
                :class="onlineBadgeClass"
              >
                {{ onlineBadgeText }}
              </span>
              <span
                v-if="nodeStore.currentNode.public_ip"
                class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 text-gray-600 border border-gray-200 font-mono"
              >
                {{ nodeStore.currentNode.public_ip }}
              </span>
              <span
                v-if="nodeStore.currentNode.region"
                class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-primary-50 text-primary-700 border border-primary-100"
              >
                {{ nodeStore.currentNode.region }}
              </span>
              <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 text-gray-600 border border-gray-200">
                #{{ nodeStore.currentNode.node_id }} · {{ nodeStore.currentNode.node_type }}
              </span>
            </div>
          </div>
          <p class="text-gray-500">
            {{ nodeStore.currentNode.description || 'Manage detailed configuration and view status' }}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-ghost flex items-center gap-2"
            @click="openMetaEditor"
          >
            <div class="i-carbon-edit text-lg"></div>
            <span>Edit Info</span>
          </button>
          <router-link
            :to="`/node/${nodeStore.currentNode.node_id}/topology`"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
          >
            <div class="i-carbon-chart-network text-lg"></div>
            <span>Topology View</span>
          </router-link>
        </div>
      </div>

      <!-- 标签页容器 -->
      <div class="card-base overflow-hidden">
        <!-- 标签页导航 -->
        <div class="border-b border-gray-100 bg-gray-50/40">
          <nav class="flex overflow-x-auto px-4 py-3">
            <div class="flex gap-1.5 p-1.5 rounded-2xl bg-white/70 border border-white/60 shadow-sm backdrop-blur-xl">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                type="button"
                @click="activeTab = tab.id"
                :class="[
                  'cursor-pointer px-4 py-2 rounded-xl text-sm font-semibold whitespace-nowrap transition-all border focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/40 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent active:scale-95',
                  activeTab === tab.id
                    ? 'bg-gradient-to-br from-primary-50 via-white to-white text-primary-700 border-primary-100 shadow-md shadow-primary-500/15 ring-1 ring-primary-200/50'
                    : 'bg-transparent border-transparent text-gray-500 hover:text-gray-900 hover:bg-white/90 hover:border-gray-100 hover:shadow-sm hover:shadow-primary-500/5',
                ]"
              >
                {{ tab.label }}
              </button>
            </div>
          </nav>
        </div>

        <!-- 标签页内容 -->
        <div class="p-6 md:p-8">
          <transition name="fade" mode="out-in">
            <!-- 概览 -->
            <div v-if="activeTab === 'overview'" key="overview" class="space-y-8">
              <!-- 基本信息卡片 -->
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-5">
                <div
                  v-for="info in basicInfo"
                  :key="info.label"
                  class="bg-gray-50/60 rounded-xl p-5 border border-gray-100 transition-all hover:bg-white hover:shadow-md hover:-translate-y-0.5"
                >
                  <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">{{ info.label }}</h3>
                  <p class="text-2xl font-bold text-gray-800 tracking-tight">{{ info.value }}</p>
                </div>
              </div>

              <!-- 实时状态 -->
              <div v-if="hasMetrics" class="space-y-5">
                <h3 class="text-lg font-bold text-gray-800 flex items-center gap-2">
                  <div class="w-1.5 h-1.5 rounded-full bg-primary-500 animate-pulse"></div>
                  System Metrics
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
                  <!-- CPU -->
                  <div
                    v-if="nodeStore.currentNode.cpu_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all"
                  >
                    <div class="flex justify-between items-start mb-4">
                      <div class="flex items-center gap-2">
                        <div class="p-2 rounded-lg bg-blue-50 text-blue-600">
                           <div class="i-carbon-chip text-lg"></div>
                        </div>
                        <h4 class="text-sm font-medium text-gray-700">CPU Usage</h4>
                      </div>
                      <span class="text-2xl font-bold text-gray-900">{{ Math.round(nodeStore.currentNode.cpu_usage * 100) }}%</span>
                    </div>
                    <div class="space-y-2">
                      <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
                        <div
                          class="h-full bg-blue-500 rounded-full transition-all duration-700 ease-spring"
                          :style="{ width: `${nodeStore.currentNode.cpu_usage * 100}%` }"
                        ></div>
                      </div>
                      <div class="text-right">
                        <span v-if="nodeStore.currentNode.cpu_threads" class="text-xs text-gray-400 font-mono">
                          {{ nodeStore.currentNode.cpu_threads }} Threads
                        </span>
                      </div>
                    </div>
                  </div>

                  <!-- Memory -->
                  <div
                    v-if="nodeStore.currentNode.mem_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all"
                  >
                    <div class="flex justify-between items-start mb-4">
                      <div class="flex items-center gap-2">
                        <div class="p-2 rounded-lg bg-indigo-50 text-indigo-600">
                           <div class="i-carbon-data-base text-lg"></div>
                        </div>
                        <h4 class="text-sm font-medium text-gray-700">Memory</h4>
                      </div>
                      <span class="text-2xl font-bold text-gray-900">{{ Math.round(nodeStore.currentNode.mem_usage * 100) }}%</span>
                    </div>
                    <div class="space-y-2">
                      <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
                        <div
                          class="h-full bg-indigo-500 rounded-full transition-all duration-700 ease-spring"
                          :style="{ width: `${nodeStore.currentNode.mem_usage * 100}%` }"
                        ></div>
                      </div>
                      <div class="text-right">
                        <span v-if="nodeStore.currentNode.mem_total" class="text-xs text-gray-400 font-mono">
                          {{ formatBytes(nodeStore.currentNode.mem_total) }} Total
                        </span>
                      </div>
                    </div>
                  </div>

                  <!-- Disk -->
                  <div
                    v-if="nodeStore.currentNode.disk_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all"
                  >
                    <div class="flex justify-between items-start mb-4">
                      <div class="flex items-center gap-2">
                        <div class="p-2 rounded-lg bg-purple-50 text-purple-600">
                           <div class="i-carbon-bare-metal-server text-lg"></div>
                        </div>
                        <h4 class="text-sm font-medium text-gray-700">Disk</h4>
                      </div>
                      <span class="text-2xl font-bold text-gray-900">{{ Math.round(nodeStore.currentNode.disk_usage * 100) }}%</span>
                    </div>
                    <div class="space-y-2">
                      <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
                        <div
                          class="h-full bg-purple-500 rounded-full transition-all duration-700 ease-spring"
                          :style="{ width: `${nodeStore.currentNode.disk_usage * 100}%` }"
                        ></div>
                      </div>
                      <div class="text-right">
                        <span v-if="nodeStore.currentNode.disk_total" class="text-xs text-gray-400 font-mono">
                          {{ formatBytes(nodeStore.currentNode.disk_total) }} Total
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 其他信息 -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                <div
                  v-if="nodeStore.currentNode.online_user_count !== undefined"
                  class="bg-gradient-to-br from-white to-gray-50 rounded-xl p-6 border border-gray-100 shadow-sm flex items-center justify-between"
                >
                  <div>
                    <h3 class="text-sm font-medium text-gray-500 mb-1">Online Users</h3>
                    <p class="text-3xl font-bold text-gray-900 tracking-tight">
                      {{ nodeStore.currentNode.online_user_count }}
                    </p>
                  </div>
                  <div class="w-12 h-12 rounded-full bg-green-50 flex items-center justify-center text-green-500">
                     <div class="i-carbon-user-multiple text-2xl"></div>
                  </div>
                </div>

                <div
                  v-if="nodeStore.currentNode.uptime"
                  class="bg-gradient-to-br from-white to-gray-50 rounded-xl p-6 border border-gray-100 shadow-sm flex items-center justify-between"
                >
                  <div>
                    <h3 class="text-sm font-medium text-gray-500 mb-1">Uptime</h3>
                    <p class="text-xl font-bold text-gray-900 tracking-tight font-mono">
                      {{ formatUptime(nodeStore.currentNode.uptime) }}
                    </p>
                  </div>
                  <div class="w-12 h-12 rounded-full bg-blue-50 flex items-center justify-center text-blue-500">
                     <div class="i-carbon-time text-2xl"></div>
                  </div>
                </div>
              </div>

              <div v-if="networkInterfaces.length" class="space-y-5">
                <div class="flex items-center justify-between">
                  <h3 class="text-lg font-bold text-gray-800 flex items-center gap-2">
                    <div class="w-1.5 h-1.5 rounded-full bg-primary-500 animate-pulse"></div>
                    Network Interfaces
                  </h3>
                  <button
                    :class="networkRefreshButtonClass"
                    @click="toggleNetworkAutoRefresh"
                    :title="autoRefreshNetwork ? 'Auto refresh on (5s)' : 'Auto refresh off'"
                    :aria-pressed="autoRefreshNetwork"
                  >
                    <div
                      class="i-carbon-renew w-5 h-5"
                      :class="refreshingNetwork ? 'animate-spin' : ''"
                    ></div>
                  </button>
                </div>
                <div class="overflow-x-auto rounded-xl border border-gray-100 bg-white shadow-sm">
                  <table class="min-w-full text-sm">
                    <thead class="bg-gray-50/60">
                      <tr class="text-left">
                        <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Name</th>
                        <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">RX</th>
                        <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">TX</th>
                        <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Packets</th>
                        <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Speed</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr
                        v-for="nic in networkInterfaces"
                        :key="nic.name"
                        class="border-t border-gray-100 hover:bg-gray-50/40 transition-colors"
                      >
                        <td class="px-4 py-3">
                          <div class="font-semibold text-gray-800 truncate max-w-[16rem]">{{ nic.name }}</div>
                        </td>
                        <td class="px-4 py-3 font-mono text-gray-700 whitespace-nowrap">{{ formatBytes(nic.bytes_recv) }}</td>
                        <td class="px-4 py-3 font-mono text-gray-700 whitespace-nowrap">{{ formatBytes(nic.bytes_sent) }}</td>
                        <td class="px-4 py-3 font-mono text-gray-500 whitespace-nowrap">
                          {{ nic.packets_recv }} ↓ / {{ nic.packets_sent }} ↑
                        </td>
                        <td class="px-4 py-3 font-mono text-gray-500 whitespace-nowrap">{{ formatSpeed(nic.speed) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>

            <!-- 其他标签页 -->
            <div v-else :key="activeTab" class="animate-fade-in">
              <UserManagement v-if="activeTab === 'users'" />
              <InboundManagement v-if="activeTab === 'inbounds'" />
              <OutboundManagement v-if="activeTab === 'outbounds'" />
              <RoutingManagement v-if="activeTab === 'routing'" />
              <UserMapping v-if="activeTab === 'mapping'" />
              <FirewallManagement v-if="activeTab === 'firewall'" />
              <MaintenanceMode v-if="activeTab === 'maintenance'" />
            </div>
          </transition>
        </div>
      </div>
    </div>

    <transition name="modal">
      <div
        v-if="showMetaEditor"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl overflow-hidden shadow-xl max-w-lg w-full animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <div class="space-y-1">
              <h3 class="text-lg font-bold text-gray-900">Edit Node Info</h3>
              <p class="text-xs text-gray-500">Update name, region and description</p>
            </div>
            <button @click="closeMetaEditor" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
          </div>
          <div class="p-6">
            <form @submit.prevent="saveMeta" class="space-y-5">
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">Name</label>
                <input
                  v-model="metaForm.name"
                  type="text"
                  placeholder="e.g. Tokyo Edge"
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all"
                />
              </div>
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">Region</label>
                <input
                  v-model="metaForm.region"
                  type="text"
                  placeholder="e.g. JP / SG / US-West"
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all"
                />
              </div>
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">Description</label>
                <textarea
                  v-model="metaForm.description"
                  rows="3"
                  placeholder="Short note about this node"
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all resize-none"
                ></textarea>
              </div>

              <div class="flex justify-end gap-3 pt-4 border-t border-gray-100">
                <button type="button" class="btn-ghost" @click="closeMetaEditor">Cancel</button>
                <button
                  type="submit"
                  class="btn-primary shadow-lg shadow-primary-500/20"
                  :disabled="savingMeta"
                >
                  <span v-if="savingMeta">Saving...</span>
                  <span v-else>Save</span>
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import { useAdminStore } from '@/stores/admin';
import type { NodeNetworkInterface } from '@/api/types';
import UserManagement from '@/components/UserManagement.vue';
import InboundManagement from '@/components/InboundManagement.vue';
import OutboundManagement from '@/components/OutboundManagement.vue';
import RoutingManagement from '@/components/RoutingManagement.vue';
import UserMapping from '@/components/UserMapping.vue';
import MaintenanceMode from '@/components/MaintenanceMode.vue';
import FirewallManagement from '@/components/FirewallManagement.vue';
import * as adminApi from '@/api/admin';
import { useToastStore } from '@/stores/toast';

const route = useRoute();
const nodeStore = useNodeStore();
const adminStore = useAdminStore();
const toast = useToastStore();

const activeTab = ref('overview');
const showMetaEditor = ref(false);
const savingMeta = ref(false);
const refreshingNetwork = ref(false);
const autoRefreshNetwork = ref(false);
const networkAutoRefreshTimerId = ref<number | null>(null);
const metaForm = ref({
  name: '',
  region: '',
  description: '',
});

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'users', label: 'Users' },
  { id: 'inbounds', label: 'Inbounds' },
  { id: 'outbounds', label: 'Outbounds' },
  { id: 'routing', label: 'Routing' },
  { id: 'mapping', label: 'User Map' },
  { id: 'firewall', label: 'Firewall' },
  { id: 'maintenance', label: 'Maintenance' },
];

const basicInfo = computed(() => {
  const node = nodeStore.currentNode;
  if (!node) return [];
  return [
    { label: 'Type', value: node.node_type },
    { label: 'Speed Limit', value: `${node.node_speed_limit} Mbps` },
    { label: 'Rate', value: `${node.traffic_rate}x` },
    { label: 'Sort Order', value: node.sort },
  ];
});

const hasMetrics = computed(() => {
  const node = nodeStore.currentNode;
  if (!node) return false;
  return (
    node.cpu_usage !== undefined ||
    node.mem_usage !== undefined ||
    node.disk_usage !== undefined
  );
});

const onlineBadgeText = computed(() => {
  const v = nodeStore.currentNode?.is_online;
  if (v === true) return 'Online';
  if (v === false) return 'Offline';
  return 'Unknown';
});

const onlineBadgeClass = computed(() => {
  const v = nodeStore.currentNode?.is_online;
  if (v === true) {
    return 'px-2.5 py-0.5 rounded-full text-xs font-semibold bg-green-50 text-green-700 border border-green-100';
  }
  if (v === false) {
    return 'px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 text-gray-600 border border-gray-200';
  }
  return 'px-2.5 py-0.5 rounded-full text-xs font-semibold bg-yellow-50 text-yellow-700 border border-yellow-100';
});

const formatBytes = (bytes: number): string => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(2)} ${units[unitIndex]}`;
};

const formatSpeed = (speed: number): string => {
  if (!speed) return '-';
  return `${formatBytes(speed)}/s`;
};

const networkInterfaces = computed<NodeNetworkInterface[]>(() => {
  const nics = nodeStore.currentNode?.network_interfaces || [];
  return [...nics].sort((a, b) => {
    const ta = (a.bytes_recv || 0) + (a.bytes_sent || 0);
    const tb = (b.bytes_recv || 0) + (b.bytes_sent || 0);
    return tb - ta;
  });
});

const networkRefreshButtonClass = computed(() => {
  return autoRefreshNetwork.value
    ? 'w-9 h-9 rounded-xl flex items-center justify-center bg-primary-50 text-primary-600 hover:bg-primary-100 transition-colors disabled:opacity-50'
    : 'w-9 h-9 rounded-xl flex items-center justify-center text-gray-600 hover:text-gray-800 hover:bg-gray-100/50 transition-colors disabled:opacity-50';
});

const stopNetworkAutoRefresh = () => {
  if (networkAutoRefreshTimerId.value !== null) {
    clearInterval(networkAutoRefreshTimerId.value);
    networkAutoRefreshTimerId.value = null;
  }
};

const refreshNetworkInterfaces = async () => {
  const id = nodeStore.currentNodeId;
  if (!id) return;
  if (refreshingNetwork.value) return;
  refreshingNetwork.value = true;
  try {
    const data = await adminApi.refreshNodeNetworkInterfaces(id);
    nodeStore.updateNode(id, {
      network_interfaces: data.network_interfaces || [],
    });
  } catch (err: any) {
    toast.error(err?.message || 'Failed to refresh network interfaces');
    if (autoRefreshNetwork.value) {
      autoRefreshNetwork.value = false;
      stopNetworkAutoRefresh();
    }
  } finally {
    refreshingNetwork.value = false;
  }
};

const toggleNetworkAutoRefresh = async () => {
  autoRefreshNetwork.value = !autoRefreshNetwork.value;
  if (!autoRefreshNetwork.value) {
    stopNetworkAutoRefresh();
    return;
  }

  await refreshNetworkInterfaces();
  stopNetworkAutoRefresh();
  networkAutoRefreshTimerId.value = window.setInterval(() => {
    if (!autoRefreshNetwork.value) return;
    refreshNetworkInterfaces();
  }, 5000);
};

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
};

const nodeId = computed(() => {
  const id = route.params.nodeId;
  return typeof id === 'string' ? parseInt(id, 10) : Number(id);
});

onMounted(() => {
  nodeStore.setCurrentNode(nodeId.value);
  nodeStore.fetchNodes();
});

onBeforeUnmount(() => {
  stopNetworkAutoRefresh();
});

const openMetaEditor = () => {
  const node = nodeStore.currentNode;
  if (!node) return;
  metaForm.value = {
    name: node.name ?? '',
    region: node.region ?? '',
    description: node.description ?? '',
  };
  showMetaEditor.value = true;
};

const closeMetaEditor = () => {
  showMetaEditor.value = false;
  savingMeta.value = false;
};

const saveMeta = async () => {
  const id = nodeStore.currentNodeId;
  if (!id) return;

  savingMeta.value = true;
  try {
    const name = metaForm.value.name.trim();
    const region = metaForm.value.region.trim();
    const description = metaForm.value.description.trim();

    await adminApi.updateNodeMeta(id, {
      name: name.length ? name : null,
      region: region.length ? region : null,
      description: description.length ? description : null,
    });

    nodeStore.updateNode(id, {
      name: name.length ? name : null,
      region: region.length ? region : null,
      description: description.length ? description : null,
    });
    toast.success('Node info updated');
    closeMetaEditor();
  } catch (err: any) {
    toast.error(err?.message || 'Failed to update node info');
  } finally {
    savingMeta.value = false;
  }
};

watch(
  () => route.params.nodeId,
  (newId) => {
    const id = typeof newId === 'string' ? parseInt(newId, 10) : Number(newId);
    nodeStore.setCurrentNode(id);
    adminStore.reset();

    autoRefreshNetwork.value = false;
    stopNetworkAutoRefresh();
  }
);
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s var(--ease-smooth), transform 0.3s var(--ease-smooth);
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
