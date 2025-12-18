<template>
  <div
    class="node-card card-base hover-lift group relative overflow-hidden cursor-pointer"
    @click="$emit('click')"
  >
    <!-- 装饰背景 -->
    <div
      class="absolute top-0 right-0 w-40 h-40 bg-gradient-to-br from-primary-50 to-purple-50 rounded-bl-full opacity-0 group-hover:opacity-100 transition-opacity duration-500 ease-smooth -mr-10 -mt-10"
    ></div>

    <!-- 头部 -->
    <div class="flex items-start justify-between mb-5 relative z-10">
      <div>
        <div class="flex items-center gap-2 mb-1">
          <h3 class="text-xl font-bold text-gray-800 tracking-tight">
            Node {{ node.node_id }}
          </h3>
          <div v-if="node.online_user_count !== undefined" class="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-green-50/80 border border-green-100">
            <span class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"></span>
            <span class="text-[10px] font-medium text-green-700">{{ node.online_user_count }}</span>
          </div>
        </div>
        <p class="text-sm text-gray-400 font-medium">{{ node.node_type }}</p>
      </div>
      
      <span
        class="px-3 py-1 rounded-lg text-xs font-semibold tracking-wide transition-colors duration-300"
        :class="statusClass"
      >
        {{ statusText }}
      </span>
    </div>

    <!-- 基本信息 -->
    <div class="grid grid-cols-2 gap-4 mb-6 relative z-10">
      <div class="bg-gray-50/80 rounded-xl p-3 border border-gray-100/50">
        <span class="text-xs text-gray-400 block mb-1">限速</span>
        <span class="text-sm font-bold text-gray-700">{{ node.node_speed_limit }} <span class="text-xs font-normal text-gray-400">Mbps</span></span>
      </div>
      <div class="bg-gray-50/80 rounded-xl p-3 border border-gray-100/50">
        <span class="text-xs text-gray-400 block mb-1">倍率</span>
        <span class="text-sm font-bold text-gray-700">{{ node.traffic_rate }}x</span>
      </div>
    </div>

    <!-- 指标 -->
    <div v-if="hasMetrics" class="space-y-4 relative z-10">
      <div v-if="node.cpu_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs font-medium">
          <span class="text-gray-400">CPU</span>
          <span class="text-gray-600">{{ Math.round(node.cpu_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-1.5 overflow-hidden">
          <div
            class="h-full bg-primary-400 rounded-full transition-all duration-700 ease-spring"
            :style="{ width: `${node.cpu_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.mem_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs font-medium">
          <span class="text-gray-400">RAM</span>
          <span class="text-gray-600">{{ Math.round(node.mem_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-1.5 overflow-hidden">
          <div
            class="h-full bg-indigo-400 rounded-full transition-all duration-700 ease-spring"
            :style="{ width: `${node.mem_usage * 100}%` }"
          ></div>
        </div>
      </div>
      <div v-if="node.disk_usage !== undefined" class="space-y-1.5">
        <div class="flex justify-between text-xs font-medium">
          <span class="text-gray-400">Disk</span>
          <span class="text-gray-600">{{ Math.round(node.disk_usage * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-100 rounded-full h-1.5 overflow-hidden">
          <div
            class="h-full bg-purple-400 rounded-full transition-all duration-700 ease-spring"
            :style="{ width: `${node.disk_usage * 100}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- 维护状态遮罩 -->
    <div 
      v-if="node.maintenance_mode" 
      class="absolute inset-0 bg-white/60 backdrop-blur-[1px] flex items-center justify-center z-20"
    >
      <div class="bg-white/90 px-4 py-2 rounded-xl shadow-lg border border-yellow-100 text-yellow-600 text-sm font-bold flex items-center gap-2">
        <div class="i-carbon-warning-filled animate-pulse"></div>
        维护模式
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
  if (props.node.maintenance_mode) return 'MAINTENANCE';
  if (props.node.online_user_count !== undefined && props.node.online_user_count > 0) {
    return 'ONLINE';
  }
  return 'OFFLINE';
});

const statusClass = computed(() => {
  if (props.node.maintenance_mode) {
    return 'bg-yellow-50 text-yellow-600';
  }
  if (props.node.online_user_count !== undefined && props.node.online_user_count > 0) {
    return 'bg-green-50 text-green-600';
  }
  return 'bg-gray-100 text-gray-400';
});
</script>

<style scoped>
.node-card {
  min-height: 220px;
}
</style>
