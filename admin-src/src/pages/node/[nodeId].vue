<template>
  <div class="node-detail-page p-6">
    <div class="mb-4">
      <router-link to="/" class="text-blue-600 hover:text-blue-800">
        ← 返回节点列表
      </router-link>
    </div>

    <div v-if="!nodeStore.currentNode" class="text-center py-12 text-gray-500">
      节点不存在或加载中...
    </div>

    <div v-else>
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-3xl font-bold text-gray-800">
          节点 {{ nodeStore.currentNode.node_id }} 详情
        </h1>
        <div class="flex space-x-3">
          <router-link
            :to="`/node/${nodeStore.currentNode.node_id}/topology`"
            class="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600"
          >
            拓扑视图
          </router-link>
        </div>
      </div>

      <!-- 标签页 -->
      <div class="bg-white rounded-lg shadow-md">
        <div class="border-b border-gray-200">
          <nav class="flex -mb-px">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              @click="activeTab = tab.id"
              :class="[
                'px-6 py-3 text-sm font-medium border-b-2 transition-colors',
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300',
              ]"
            >
              {{ tab.label }}
            </button>
          </nav>
        </div>

        <div class="p-6">
          <!-- 概览 -->
          <div v-if="activeTab === 'overview'" class="space-y-6">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div class="bg-gray-50 rounded-lg p-4">
                <h3 class="text-sm font-medium text-gray-500 mb-2">节点类型</h3>
                <p class="text-xl font-semibold text-gray-800">
                  {{ nodeStore.currentNode.node_type }}
                </p>
              </div>
              <div class="bg-gray-50 rounded-lg p-4">
                <h3 class="text-sm font-medium text-gray-500 mb-2">限速</h3>
                <p class="text-xl font-semibold text-gray-800">
                  {{ nodeStore.currentNode.node_speed_limit }} Mbps
                </p>
              </div>
              <div class="bg-gray-50 rounded-lg p-4">
                <h3 class="text-sm font-medium text-gray-500 mb-2">流量倍率</h3>
                <p class="text-xl font-semibold text-gray-800">
                  {{ nodeStore.currentNode.traffic_rate }}x
                </p>
              </div>
              <div class="bg-gray-50 rounded-lg p-4">
                <h3 class="text-sm font-medium text-gray-500 mb-2">排序</h3>
                <p class="text-xl font-semibold text-gray-800">
                  {{ nodeStore.currentNode.sort }}
                </p>
              </div>
            </div>

            <div v-if="hasMetrics" class="space-y-4">
              <h3 class="text-lg font-semibold text-gray-800">实时状态</h3>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div v-if="nodeStore.currentNode.cpu_usage !== undefined" class="bg-gray-50 rounded-lg p-4">
                  <h4 class="text-sm font-medium text-gray-500 mb-2">CPU 使用率</h4>
                  <div class="space-y-2">
                    <div class="flex justify-between text-sm">
                      <span>{{ Math.round(nodeStore.currentNode.cpu_usage * 100) }}%</span>
                      <span v-if="nodeStore.currentNode.cpu_threads">
                        {{ nodeStore.currentNode.cpu_threads }} 线程
                      </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                      <div
                        class="bg-blue-500 h-2 rounded-full transition-all"
                        :style="{ width: `${nodeStore.currentNode.cpu_usage * 100}%` }"
                      ></div>
                    </div>
                  </div>
                </div>
                <div v-if="nodeStore.currentNode.mem_usage !== undefined" class="bg-gray-50 rounded-lg p-4">
                  <h4 class="text-sm font-medium text-gray-500 mb-2">内存使用率</h4>
                  <div class="space-y-2">
                    <div class="flex justify-between text-sm">
                      <span>{{ Math.round(nodeStore.currentNode.mem_usage * 100) }}%</span>
                      <span v-if="nodeStore.currentNode.mem_total">
                        {{ formatBytes(nodeStore.currentNode.mem_total) }}
                      </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                      <div
                        class="bg-green-500 h-2 rounded-full transition-all"
                        :style="{ width: `${nodeStore.currentNode.mem_usage * 100}%` }"
                      ></div>
                    </div>
                  </div>
                </div>
                <div v-if="nodeStore.currentNode.disk_usage !== undefined" class="bg-gray-50 rounded-lg p-4">
                  <h4 class="text-sm font-medium text-gray-500 mb-2">磁盘使用率</h4>
                  <div class="space-y-2">
                    <div class="flex justify-between text-sm">
                      <span>{{ Math.round(nodeStore.currentNode.disk_usage * 100) }}%</span>
                      <span v-if="nodeStore.currentNode.disk_total">
                        {{ formatBytes(nodeStore.currentNode.disk_total) }}
                      </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                      <div
                        class="bg-yellow-500 h-2 rounded-full transition-all"
                        :style="{ width: `${nodeStore.currentNode.disk_usage * 100}%` }"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="nodeStore.currentNode.online_user_count !== undefined" class="bg-gray-50 rounded-lg p-4">
              <h3 class="text-sm font-medium text-gray-500 mb-2">在线用户数</h3>
              <p class="text-2xl font-semibold text-blue-600">
                {{ nodeStore.currentNode.online_user_count }}
              </p>
            </div>

            <div v-if="nodeStore.currentNode.uptime" class="bg-gray-50 rounded-lg p-4">
              <h3 class="text-sm font-medium text-gray-500 mb-2">运行时间</h3>
              <p class="text-lg font-semibold text-gray-800">
                {{ formatUptime(nodeStore.currentNode.uptime) }}
              </p>
            </div>
          </div>

          <!-- 用户管理 -->
          <div v-if="activeTab === 'users'">
            <UserManagement />
          </div>

          <!-- 上游代理 -->
          <div v-if="activeTab === 'outbounds'">
            <OutboundManagement />
          </div>

          <!-- 路由配置 -->
          <div v-if="activeTab === 'routing'">
            <RoutingManagement />
          </div>

          <!-- 用户映射 -->
          <div v-if="activeTab === 'mapping'">
            <UserMapping />
          </div>

          <!-- 维护模式 -->
          <div v-if="activeTab === 'maintenance'">
            <MaintenanceMode />
          </div>
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

<style scoped></style>

