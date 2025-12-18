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
              Node {{ nodeStore.currentNode.node_id }}
            </h1>
            <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 text-gray-600 border border-gray-200">
              {{ nodeStore.currentNode.node_type }}
            </span>
          </div>
          <p class="text-gray-500">Manage detailed configuration and view status</p>
        </div>
        <router-link
          :to="`/node/${nodeStore.currentNode.node_id}/topology`"
          class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
        >
          <div class="i-carbon-chart-network text-lg"></div>
          <span>Topology View</span>
        </router-link>
      </div>

      <!-- 标签页容器 -->
      <div class="card-base overflow-hidden">
        <!-- 标签页导航 -->
        <div class="border-b border-gray-100 bg-gray-50/50">
          <nav class="flex overflow-x-auto px-2">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              @click="activeTab = tab.id"
              :class="[
                'px-6 py-4 text-sm font-medium transition-all whitespace-nowrap relative',
                activeTab === tab.id
                  ? 'text-primary-600'
                  : 'text-gray-500 hover:text-gray-700 hover:bg-gray-100/50 rounded-t-lg',
              ]"
            >
              {{ tab.label }}
              <span
                v-if="activeTab === tab.id"
                class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-500 shadow-[0_-2px_6px_rgba(99,102,241,0.4)]"
              ></span>
            </button>
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
                  <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all">
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
                  <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all">
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
                  <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all">
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
            </div>

            <!-- 其他标签页 -->
            <div v-else :key="activeTab" class="animate-fade-in">
              <UserManagement v-if="activeTab === 'users'" />
              <OutboundManagement v-if="activeTab === 'outbounds'" />
              <RoutingManagement v-if="activeTab === 'routing'" />
              <UserMapping v-if="activeTab === 'mapping'" />
              <MaintenanceMode v-if="activeTab === 'maintenance'" />
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import { useAdminStore } from '@/stores/admin';
import UserManagement from '@/components/UserManagement.vue';
import OutboundManagement from '@/components/OutboundManagement.vue';
import RoutingManagement from '@/components/RoutingManagement.vue';
import UserMapping from '@/components/UserMapping.vue';
import MaintenanceMode from '@/components/MaintenanceMode.vue';

const route = useRoute();
const nodeStore = useNodeStore();
const adminStore = useAdminStore();

const activeTab = ref('overview');

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'users', label: 'Users' },
  { id: 'outbounds', label: 'Outbounds' },
  { id: 'routing', label: 'Routing' },
  { id: 'mapping', label: 'User Map' },
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

watch(
  () => route.params.nodeId,
  (newId) => {
    const id = typeof newId === 'string' ? parseInt(newId, 10) : Number(newId);
    nodeStore.setCurrentNode(id);
    adminStore.reset();
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
