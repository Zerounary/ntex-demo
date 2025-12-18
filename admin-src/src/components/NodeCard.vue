<template>
  <div
    class="node-card bg-white rounded-2xl p-6 shadow-sm border border-gray-100 hover:shadow-xl transition-all duration-300 relative overflow-hidden group"
    @click="$emit('click')"
  >
    <!-- 渐变背景装饰 -->
    <div
      class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-300 -mr-16 -mt-16"
    ></div>

    <!-- 头部 -->
    <div class="flex items-start justify-between mb-4 relative z-10">
      <div>
        <h3 class="text-xl font-bold text-gray-800 mb-1">
          节点 {{ node.node_id }}
        </h3>
        <p class="text-xs text-gray-500">{{ node.node_type }}</p>
      </div>
      <span
        class="px-3 py-1 rounded-full text-xs font-medium shadow-sm"
        :class="statusClass"
      >
        {{ statusText }}
      </span>
    </div>

    <!-- 基本信息 -->
    <div class="space-y-2 text-sm text-gray-600 mb-4 relative z-10">
      <div class="flex justify-between items-center">
        <span class="text-gray-500">限速</span>
        <span class="font-semibold text-gray-800">{{ node.node_speed_limit }} Mbps</span>
      </div>
      <div class="flex justify-between items-center">
        <span class="text-gray-500">流量倍率</span>
        <span class="font-semibold text-gray-800">{{ node.traffic_rate }}x</span>
      </div>
    </div>

    <!-- 指标 -->
    <div v-if="hasMetrics" class="space-y-3 mb-4 relative z-10">
      <div v-if="node.cpu_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-gray-500">CPU</span>
          <span class="font-medium text-gray-700">{{ Math.round(node.cpu_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-blue-400 to-blue-600 rounded-full transition-all duration-500"
            :style="{ width: `${node.cpu_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.mem_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-gray-500">内存</span>
          <span class="font-medium text-gray-700">{{ Math.round(node.mem_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-green-400 to-green-600 rounded-full transition-all duration-500"
            :style="{ width: `${node.mem_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.disk_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-gray-500">磁盘</span>
          <span class="font-medium text-gray-700">{{ Math.round(node.disk_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-yellow-400 to-yellow-600 rounded-full transition-all duration-500"
            :style="{ width: `${node.disk_usage * 100}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- 底部信息 -->
    <div class="flex items-center justify-between pt-4 border-t border-gray-100 relative z-10">
      <div v-if="node.online_user_count !== undefined" class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
        <span class="text-xs text-gray-600">
          <span class="font-semibold text-gray-800">{{ node.online_user_count }}</span> 在线
        </span>
      </div>
      <div v-if="node.maintenance_mode" class="px-2 py-1 bg-yellow-50 text-yellow-700 text-xs rounded-lg border border-yellow-200">
        维护中
      </div>
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
  if (props.node.maintenance_mode) {
    return 'bg-yellow-100 text-yellow-700 border border-yellow-200';
  }
  if (props.node.online_user_count !== undefined && props.node.online_user_count > 0) {
    return 'bg-green-100 text-green-700 border border-green-200';
  }
  return 'bg-gray-100 text-gray-600 border border-gray-200';
});
</script>

<style scoped>
.node-card {
  min-height: 240px;
}
</style>
