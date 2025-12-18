<template>
  <div
    class="node-card bg-white rounded-lg shadow-md p-6 cursor-pointer transition-all hover:shadow-lg border border-gray-200"
    @click="$emit('click')"
  >
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-xl font-bold text-gray-800">节点 {{ node.node_id }}</h3>
      <span
        class="px-3 py-1 rounded-full text-sm font-medium"
        :class="statusClass"
      >
        {{ statusText }}
      </span>
    </div>

    <div class="space-y-2 text-sm text-gray-600">
      <div class="flex justify-between">
        <span>类型:</span>
        <span class="font-medium">{{ node.node_type }}</span>
      </div>
      <div class="flex justify-between">
        <span>限速:</span>
        <span class="font-medium">{{ node.node_speed_limit }} Mbps</span>
      </div>
      <div class="flex justify-between">
        <span>流量倍率:</span>
        <span class="font-medium">{{ node.traffic_rate }}x</span>
      </div>
    </div>

    <div v-if="hasMetrics" class="mt-4 space-y-2">
      <div v-if="node.cpu_usage !== undefined" class="space-y-1">
        <div class="flex justify-between text-xs text-gray-500">
          <span>CPU</span>
          <span>{{ Math.round(node.cpu_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-2">
          <div
            class="bg-blue-500 h-2 rounded-full transition-all"
            :style="{ width: `${node.cpu_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.mem_usage !== undefined" class="space-y-1">
        <div class="flex justify-between text-xs text-gray-500">
          <span>内存</span>
          <span>{{ Math.round(node.mem_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-2">
          <div
            class="bg-green-500 h-2 rounded-full transition-all"
            :style="{ width: `${node.mem_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.disk_usage !== undefined" class="space-y-1">
        <div class="flex justify-between text-xs text-gray-500">
          <span>磁盘</span>
          <span>{{ Math.round(node.disk_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-2">
          <div
            class="bg-yellow-500 h-2 rounded-full transition-all"
            :style="{ width: `${node.disk_usage * 100}%` }"
          ></div>
        </div>
      </div>
    </div>

    <div v-if="node.online_user_count !== undefined" class="mt-4 text-sm text-gray-600">
      <span>在线用户: </span>
      <span class="font-medium text-blue-600">{{ node.online_user_count }}</span>
    </div>

    <div v-if="node.maintenance_mode" class="mt-3">
      <span class="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded">
        维护模式
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { NodeInfo } from '@/api/types';

interface Props {
  node: NodeInfo;
}

const props = defineProps<Props>();

const hasMetrics = computed(() => {
  return (
    props.node.cpu_usage !== undefined ||
    props.node.mem_usage !== undefined ||
    props.node.disk_usage !== undefined
  );
});

const statusText = computed(() => {
  if (props.node.maintenance_mode) return '维护中';
  if (props.node.online_user_count !== undefined && props.node.online_user_count > 0) {
    return '运行中';
  }
  return '离线';
});

const statusClass = computed(() => {
  if (props.node.maintenance_mode) return 'bg-yellow-100 text-yellow-800';
  if (props.node.online_user_count !== undefined && props.node.online_user_count > 0) {
    return 'bg-green-100 text-green-800';
  }
  return 'bg-gray-100 text-gray-800';
});
</script>

<style scoped>
.node-card {
  min-height: 200px;
}
</style>

