<template>
  <div class="node-detail-page min-h-screen p-6 md:p-8 lg:p-12">
    <!-- 返回按钮 -->
    <div class="mb-6 animate-slide-up">
      <router-link
        to="/"
        class="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900 transition-colors group"
      >
        <span class="group-hover:-translate-x-1 transition-transform">←</span>
        <span>返回节点列表</span>
      </router-link>
    </div>

    <div v-if="!nodeStore.currentNode" class="text-center py-20">
      <div class="w-16 h-16 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-4"></div>
      <p class="text-gray-500">加载中...</p>
    </div>

    <div v-else class="space-y-6 animate-fade-in">
      <!-- 页面头部 -->
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 class="text-4xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent mb-2">
            节点 {{ nodeStore.currentNode.node_id }} 详情
          </h1>
          <p class="text-gray-500 text-sm">{{ nodeStore.currentNode.node_type }}</p>
        </div>
        <router-link
          :to="`/node/${nodeStore.currentNode.node_id}/topology`"
          class="px-6 py-3 bg-gradient-to-r from-green-500 to-emerald-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl transition-all active-scale flex items-center gap-2"
        >
          <span>📊</span>
          <span>拓扑视图</span>
        </router-link>
      </div>

      <!-- 标签页容器 -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">
        <!-- 标签页导航 -->
        <div class="border-b border-gray-200 bg-gradient-to-r from-gray-50 to-gray-100">
          <nav class="flex overflow-x-auto">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              @click="activeTab = tab.id"
              :class="[
                'px-6 py-4 text-sm font-medium border-b-2 transition-all whitespace-nowrap relative',
                activeTab === tab.id
                  ? 'border-indigo-500 text-indigo-600 bg-white'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:bg-white/50',
              ]"
            >
              {{ tab.label }}
              <span
                v-if="activeTab === tab.id"
                class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-indigo-500 to-purple-500"
              ></span>
            </button>
          </nav>
        </div>

        <!-- 标签页内容 -->
        <div class="p-6 md:p-8">
          <transition name="fade" mode="out-in">
            <!-- 概览 -->
            <div v-if="activeTab === 'overview'" key="overview" class="space-y-6">
              <!-- 基本信息卡片 -->
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div
                  v-for="info in basicInfo"
                  :key="info.label"
                  class="bg-gradient-to-br from-indigo-50 to-purple-50 rounded-xl p-5 border border-indigo-100 hover:shadow-md transition-all"
                >
                  <h3 class="text-xs font-medium text-gray-500 mb-2">{{ info.label }}</h3>
                  <p class="text-2xl font-bold text-gray-800">{{ info.value }}</p>
                </div>
              </div>

              <!-- 实时状态 -->
              <div v-if="hasMetrics" class="space-y-4">
                <h3 class="text-lg font-semibold text-gray-800 flex items-center gap-2">
                  <span class="w-1.5 h-1.5 rounded-full bg-indigo-500"></span>
                  实时状态
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div
                    v-if="nodeStore.currentNode.cpu_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-200 shadow-sm hover:shadow-md transition-all"
                  >
                    <h4 class="text-sm font-medium text-gray-500 mb-3">CPU 使用率</h4>
                    <div class="space-y-2">
                      <div class="flex justify-between items-center text-sm">
                        <span class="font-semibold text-gray-800">
                          {{ Math.round(nodeStore.currentNode.cpu_usage * 100) }}%
                        </span>
                        <span v-if="nodeStore.currentNode.cpu_threads" class="text-gray-500">
                          {{ nodeStore.currentNode.cpu_threads }} 线程
                        </span>
                      </div>
                      <div class="w-full bg-gray-100 rounded-full h-3 overflow-hidden">
                        <div
                          class="h-full bg-gradient-to-r from-blue-400 to-blue-600 rounded-full transition-all duration-500"
                          :style="{ width: `${nodeStore.currentNode.cpu_usage * 100}%` }"
                        ></div>
                      </div>
                    </div>
                  </div>
                  <div
                    v-if="nodeStore.currentNode.mem_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-200 shadow-sm hover:shadow-md transition-all"
                  >
                    <h4 class="text-sm font-medium text-gray-500 mb-3">内存使用率</h4>
                    <div class="space-y-2">
                      <div class="flex justify-between items-center text-sm">
                        <span class="font-semibold text-gray-800">
                          {{ Math.round(nodeStore.currentNode.mem_usage * 100) }}%
                        </span>
                        <span v-if="nodeStore.currentNode.mem_total" class="text-gray-500 text-xs">
                          {{ formatBytes(nodeStore.currentNode.mem_total) }}
                        </span>
                      </div>
                      <div class="w-full bg-gray-100 rounded-full h-3 overflow-hidden">
                        <div
                          class="h-full bg-gradient-to-r from-green-400 to-green-600 rounded-full transition-all duration-500"
                          :style="{ width: `${nodeStore.currentNode.mem_usage * 100}%` }"
                        ></div>
                      </div>
                    </div>
                  </div>
                  <div
                    v-if="nodeStore.currentNode.disk_usage !== undefined"
                    class="bg-white rounded-xl p-5 border border-gray-200 shadow-sm hover:shadow-md transition-all"
                  >
                    <h4 class="text-sm font-medium text-gray-500 mb-3">磁盘使用率</h4>
                    <div class="space-y-2">
                      <div class="flex justify-between items-center text-sm">
                        <span class="font-semibold text-gray-800">
                          {{ Math.round(nodeStore.currentNode.disk_usage * 100) }}%
                        </span>
                        <span v-if="nodeStore.currentNode.disk_total" class="text-gray-500 text-xs">
                          {{ formatBytes(nodeStore.currentNode.disk_total) }}
                        </span>
                      </div>
                      <div class="w-full bg-gray-100 rounded-full h-3 overflow-hidden">
                        <div
                          class="h-full bg-gradient-to-r from-yellow-400 to-yellow-600 rounded-full transition-all duration-500"
                          :style="{ width: `${nodeStore.currentNode.disk_usage * 100}%` }"
                        ></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 其他信息 -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div
                  v-if="nodeStore.currentNode.online_user_count !== undefined"
                  class="bg-gradient-to-br from-blue-50 to-cyan-50 rounded-xl p-5 border border-blue-100"
                >
                  <h3 class="text-sm font-medium text-gray-500 mb-2">在线用户数</h3>
                  <p class="text-3xl font-bold text-blue-600">
                    {{ nodeStore.currentNode.online_user_count }}
                  </p>
                </div>
                <div
                  v-if="nodeStore.currentNode.uptime"
                  class="bg-gradient-to-br from-purple-50 to-pink-50 rounded-xl p-5 border border-purple-100"
                >
                  <h3 class="text-sm font-medium text-gray-500 mb-2">运行时间</h3>
                  <p class="text-xl font-semibold text-gray-800">
                    {{ formatUptime(nodeStore.currentNode.uptime) }}
                  </p>
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
  { id: 'overview', label: '概览' },
  { id: 'users', label: '用户管理' },
  { id: 'outbounds', label: '上游代理' },
  { id: 'routing', label: '路由配置' },
  { id: 'mapping', label: '用户映射' },
  { id: 'maintenance', label: '维护模式' },
];

const basicInfo = computed(() => {
  const node = nodeStore.currentNode;
  if (!node) return [];
  return [
    { label: '节点类型', value: node.node_type },
    { label: '限速', value: `${node.node_speed_limit} Mbps` },
    { label: '流量倍率', value: `${node.traffic_rate}x` },
    { label: '排序', value: node.sort },
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
  if (days > 0) return `${days} 天 ${hours} 小时`;
  if (hours > 0) return `${hours} 小时 ${minutes} 分钟`;
  return `${minutes} 分钟`;
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
  transition: opacity 0.2s ease, transform 0.2s ease;
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
