<template>
  <div class="outbound-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
          上游代理管理
        </h2>
        <p class="text-sm text-gray-500 mt-1">配置和管理上游代理服务器</p>
      </div>
      <button
        @click="showAddForm = true"
        class="px-6 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl transition-all active-scale flex items-center gap-2"
      >
        <span>+</span>
        <span>添加代理</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.outboundsLoading" class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-3"></div>
        <p class="text-gray-500 text-sm">加载中...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.outboundsError"
      class="p-4 bg-red-50 border border-red-200 text-red-700 rounded-xl"
    >
      {{ adminStore.outboundsError }}
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="adminStore.outbounds.length === 0"
      class="text-center py-16 bg-gradient-to-br from-gray-50 to-gray-100 rounded-2xl border border-gray-200"
    >
      <div class="w-20 h-20 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <span class="text-3xl">🔗</span>
      </div>
      <p class="text-gray-500">暂无上游代理</p>
    </div>

    <!-- 代理卡片列表 -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <transition-group name="list" tag="div" class="contents">
        <div
          v-for="(outbound, index) in adminStore.outbounds"
          :key="outbound.tag"
          :style="{ 'animation-delay': `${index * 50}ms` }"
          class="bg-white rounded-2xl p-5 border border-gray-100 shadow-sm hover:shadow-lg transition-all animate-slide-up"
        >
          <div class="flex justify-between items-start mb-3">
            <div class="flex-1">
              <h3 class="text-lg font-bold text-gray-800 mb-1">{{ outbound.tag }}</h3>
              <p class="text-xs text-gray-500">
                协议: <span class="font-medium text-gray-700">{{ outbound.protocol }}</span>
              </p>
            </div>
            <div class="flex gap-2">
              <button
                @click="testLatency(outbound.tag)"
                :disabled="testingLatency === outbound.tag"
                class="px-3 py-1.5 text-xs font-medium bg-green-50 text-green-700 rounded-lg hover:bg-green-100 disabled:opacity-50 transition-all"
              >
                {{ testingLatency === outbound.tag ? '测试中...' : '测试延迟' }}
              </button>
              <button
                @click="editOutbound(outbound)"
                class="px-3 py-1.5 text-xs font-medium bg-indigo-50 text-indigo-700 rounded-lg hover:bg-indigo-100 transition-all"
              >
                编辑
              </button>
              <button
                @click="deleteOutbound(outbound.tag)"
                class="px-3 py-1.5 text-xs font-medium bg-red-50 text-red-700 rounded-lg hover:bg-red-100 transition-all"
              >
                删除
              </button>
            </div>
          </div>
          
          <div v-if="latencyResults[outbound.tag]" class="mb-3 p-2 bg-gray-50 rounded-lg">
            <span class="text-xs text-gray-600">UDP 延迟: </span>
            <span
              :class="
                latencyResults[outbound.tag].latency
                  ? 'text-green-600 font-semibold'
                  : 'text-red-600'
              "
            >
              {{
                latencyResults[outbound.tag].latency
                  ? `${latencyResults[outbound.tag].latency}ms`
                  : latencyResults[outbound.tag].error || '未知'
              }}
            </span>
          </div>
          
          <details class="cursor-pointer group">
            <summary class="text-xs text-gray-500 group-hover:text-gray-700 transition-colors">
              查看配置
            </summary>
            <pre
              class="mt-2 p-3 bg-gray-50 rounded-lg text-xs overflow-x-auto border border-gray-200"
            >{{ JSON.stringify(outbound.settings, null, 2) }}</pre>
          </details>
        </div>
      </transition-group>
    </div>

    <!-- 添加/编辑表单对话框 -->
    <transition name="modal">
      <div
        v-if="showAddForm || editingOutbound"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="closeForm"
      >
        <div class="bg-white rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] overflow-y-auto animate-scale-in">
          <div class="p-6 border-b border-gray-200">
            <h3 class="text-xl font-bold text-gray-800">
              {{ editingOutbound ? '编辑代理' : '添加代理' }}
            </h3>
          </div>
          <div class="p-6">
            <OutboundForm
              :outbound="editingOutbound"
              :is-edit="!!editingOutbound"
              @submit="handleSubmit"
              @cancel="closeForm"
            />
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';
import * as adminApi from '@/api/admin';
import OutboundForm from './OutboundForm.vue';
import type { OutboundConfig, UdpLatencyResult } from '@/api/types';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const showAddForm = ref(false);
const editingOutbound = ref<OutboundConfig | null>(null);
const testingLatency = ref<string | null>(null);
const latencyResults = ref<Record<string, UdpLatencyResult>>({});

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchOutbounds(nodeStore.currentNodeId);
  }
});

const editOutbound = (outbound: OutboundConfig) => {
  editingOutbound.value = outbound;
  showAddForm.value = false;
};

const deleteOutbound = async (tag: string) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('确定要删除这个代理吗？')) return;

  try {
    await adminStore.deleteOutbound(nodeStore.currentNodeId, tag);
  } catch (err: any) {
    alert(err.message || '删除代理失败');
  }
};

const testLatency = async (tag: string) => {
  if (!nodeStore.currentNodeId) return;

  testingLatency.value = tag;
  try {
    const result = await adminApi.queryUdpLatency(nodeStore.currentNodeId, tag);
    latencyResults.value[tag] = result;
  } catch (err: any) {
    latencyResults.value[tag] = { error: err.message || '测试失败' };
  } finally {
    testingLatency.value = null;
  }
};

const handleSubmit = async (outboundData: {
  tag: string;
  protocol: string;
  settings: any;
  stream_settings?: any;
}) => {
  if (!nodeStore.currentNodeId) return;

  try {
    if (editingOutbound.value) {
      await adminStore.updateOutbound(
        nodeStore.currentNodeId,
        editingOutbound.value.tag,
        {
          protocol: outboundData.protocol,
          settings: outboundData.settings,
        }
      );
    } else {
      await adminStore.addOutbound(nodeStore.currentNodeId, outboundData);
    }
    closeForm();
  } catch (err: any) {
    alert(err.message || '操作失败');
  }
};

const closeForm = () => {
  showAddForm.value = false;
  editingOutbound.value = null;
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

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
