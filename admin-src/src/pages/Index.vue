<template>
  <div class="index-page min-h-screen p-6 md:p-8 lg:p-12">
    <!-- 页面头部 -->
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 class="text-4xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent mb-2">
            节点管理
          </h1>
          <p class="text-gray-500 text-sm">管理和监控您的 Xray-core 节点</p>
        </div>
        <button
          @click="refreshNodes"
          :disabled="nodeStore.loading"
          class="px-6 py-3 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed transition-all active-scale flex items-center gap-2"
        >
          <span v-if="!nodeStore.loading">刷新</span>
          <span v-else class="animate-spin">⟳</span>
        </button>
      </div>
    </div>

    <!-- 错误提示 -->
    <transition name="fade">
      <div
        v-if="nodeStore.error"
        class="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-xl shadow-sm animate-slide-up"
      >
        {{ nodeStore.error }}
      </div>
    </transition>

    <!-- 加载状态 -->
    <div
      v-if="nodeStore.loading && nodeStore.nodes.length === 0"
      class="flex items-center justify-center py-20"
    >
      <div class="text-center">
        <div class="w-16 h-16 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-4"></div>
        <p class="text-gray-500">加载中...</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="nodeStore.sortedNodes.length === 0"
      class="text-center py-20"
    >
      <div class="w-24 h-24 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <span class="text-4xl">📡</span>
      </div>
      <p class="text-gray-500 text-lg">暂无节点</p>
    </div>

    <!-- 节点网格 -->
    <div
      v-else
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
    >
      <transition-group name="list" tag="div" class="contents">
        <NodeCard
          v-for="(node, index) in nodeStore.sortedNodes"
          :key="node.node_id"
          :node="node"
          :style="{ 'animation-delay': `${index * 50}ms` }"
          @click="goToNodeDetail(node.node_id)"
          class="animate-slide-up hover-lift cursor-pointer"
        />
      </transition-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNodeStore } from '@/stores/node';
import NodeCard from '@/components/NodeCard.vue';

const router = useRouter();
const nodeStore = useNodeStore();

onMounted(() => {
  nodeStore.fetchNodes();
});

const refreshNodes = () => {
  nodeStore.fetchNodes();
};

const goToNodeDetail = (nodeId: number) => {
  router.push(`/node/${nodeId}`);
};
</script>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.list-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
</style>
