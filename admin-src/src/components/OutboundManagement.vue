<template>
  <div class="outbound-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">
          Outbound Proxies
        </h2>
        <p class="text-sm text-gray-500 mt-1">Manage upstream proxy servers</p>
      </div>
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="w-9 h-9 rounded-xl flex items-center justify-center text-gray-600 hover:text-gray-800 hover:bg-gray-100/50 transition-colors disabled:opacity-50"
          @click="refreshOutbounds"
          :disabled="refreshing"
          title="Refresh"
        >
          <div class="i-carbon-renew w-5 h-5" :class="refreshing ? 'animate-spin' : ''"></div>
        </button>
        <button
          @click="showAddForm = true"
          class="btn-primary flex items-center gap-2 shadow-lg hover:shadow-xl hover:-translate-y-0.5"
        >
          <div class="i-carbon-add text-lg"></div>
          <span>Add Proxy</span>
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.outboundsLoading" class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="loading-ring w-10 h-10 relative mx-auto mb-4"></div>
        <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.outboundsError"
      class="p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-xl flex items-center gap-3"
    >
      <div class="i-carbon-warning-alt text-lg"></div>
      <span class="font-medium">{{ adminStore.outboundsError }}</span>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="adminStore.outbounds.length === 0"
      class="flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed"
    >
      <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 text-gray-400">
        <div class="i-carbon-link text-3xl"></div>
      </div>
      <h3 class="text-lg font-bold text-gray-700 mb-1">No Proxies Found</h3>
      <p class="text-gray-400 text-sm">Add an outbound proxy to get started</p>
    </div>

    <!-- 代理卡片列表 -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <transition-group name="list" tag="div" class="contents">
        <div
          v-for="(outbound, index) in adminStore.outbounds"
          :key="outbound.tag"
          :style="{ 'animation-delay': `${index * 50}ms` }"
          class="card-base p-6 hover:shadow-md transition-all animate-slide-up group"
        >
          <div class="flex justify-between items-start mb-4">
            <div class="flex-1">
              <h3 class="text-lg font-bold text-gray-900 mb-1 flex items-center gap-2">
                {{ outbound.tag }}
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-gray-100 text-gray-500 uppercase tracking-wider">
                  {{ outbound.protocol }}
                </span>
              </h3>
            </div>
            <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
               <button
                @click="testLatency(outbound.tag)"
                :disabled="testingLatency === outbound.tag"
                class="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded-lg transition-colors"
                :title="testingLatency === outbound.tag ? 'Testing...' : 'Test Latency'"
              >
                <div :class="testingLatency === outbound.tag ? 'animate-spin' : ''" class="i-carbon-activity text-lg"></div>
              </button>
              <button
                @click="editOutbound(outbound)"
                class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                title="Edit"
              >
                <div class="i-carbon-edit text-lg"></div>
              </button>
              <button
                @click="deleteOutbound(outbound.tag)"
                class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                title="Delete"
              >
                <div class="i-carbon-trash-can text-lg"></div>
              </button>
            </div>
          </div>
          
          <div v-if="latencyResults[outbound.tag]" class="mb-4 p-2.5 bg-gray-50/50 rounded-lg border border-gray-100 flex items-center justify-between">
            <span class="text-xs font-medium text-gray-500">UDP Latency</span>
            <span
              :class="
                latencyResults[outbound.tag].latency
                  ? 'text-green-600 font-bold'
                  : 'text-red-500 font-medium'
              "
              class="text-sm"
            >
              {{
                latencyResults[outbound.tag].latency
                  ? `${latencyResults[outbound.tag].latency} ms`
                  : latencyResults[outbound.tag].error || 'Failed'
              }}
            </span>
          </div>
          
          <details class="cursor-pointer group/details">
            <summary class="text-xs font-medium text-gray-400 hover:text-primary-600 transition-colors flex items-center gap-1 select-none">
              <div class="i-carbon-chevron-right group-open/details:rotate-90 transition-transform"></div>
              View Configuration
            </summary>
            <div class="mt-3 p-3 bg-gray-900 rounded-xl overflow-hidden shadow-inner">
               <pre class="text-[10px] text-gray-300 font-mono overflow-x-auto custom-scrollbar">{{ JSON.stringify(outbound.settings, null, 2) }}</pre>
            </div>
          </details>
        </div>
      </transition-group>
    </div>

    <!-- 添加/编辑表单对话框 -->
    <transition name="modal">
      <div
        v-if="showAddForm || editingOutbound"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-3xl w-full max-h-[90vh] overflow-y-auto animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <h3 class="text-lg font-bold text-gray-900">
              {{ editingOutbound ? 'Edit Proxy' : 'Add New Proxy' }}
            </h3>
            <button @click="closeForm" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
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
import { useToastStore } from '@/stores/toast';
import * as adminApi from '@/api/admin';
import OutboundForm from './OutboundForm.vue';
import type { OutboundConfig, UdpLatencyResult } from '@/api/types';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();
const toastStore = useToastStore();

const showAddForm = ref(false);
const editingOutbound = ref<OutboundConfig | null>(null);
const testingLatency = ref<string | null>(null);
const latencyResults = ref<Record<string, UdpLatencyResult>>({});
const refreshing = ref(false);

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchOutbounds(nodeStore.currentNodeId);
  }
});

const refreshOutbounds = async () => {
  const id = nodeStore.currentNodeId;
  if (!id) return;
  if (refreshing.value) return;
  refreshing.value = true;
  try {
    await adminStore.fetchOutbounds(id);
    if (adminStore.outboundsError) {
      toastStore.error(adminStore.outboundsError);
    }
  } finally {
    refreshing.value = false;
  }
};

const editOutbound = (outbound: OutboundConfig) => {
  editingOutbound.value = outbound;
  showAddForm.value = false;
};

const deleteOutbound = async (tag: string) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('Are you sure you want to delete this proxy?')) return;

  try {
    await adminStore.deleteOutbound(nodeStore.currentNodeId, tag);
    toastStore.success('Proxy deleted successfully');
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to delete proxy');
  }
};

const testLatency = async (tag: string) => {
  if (!nodeStore.currentNodeId) return;

  testingLatency.value = tag;
  try {
    const result = await adminApi.queryUdpLatency(nodeStore.currentNodeId, tag);
    latencyResults.value[tag] = result;
    if (result.error) {
      toastStore.warning(`Latency test failed: ${result.error}`);
    } else {
      toastStore.success(`Latency: ${result.latency}ms`);
    }
  } catch (err: any) {
    latencyResults.value[tag] = { error: err.message || 'Test failed' };
    toastStore.error(err.message || 'Test failed');
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
      toastStore.success('Proxy updated successfully');
    } else {
      await adminStore.addOutbound(nodeStore.currentNodeId, outboundData);
      toastStore.success('Proxy added successfully');
    }
    closeForm();
  } catch (err: any) {
    toastStore.error(err.message || 'Operation failed');
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
  transition: all 0.3s var(--ease-smooth);
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
  transition: opacity 0.3s var(--ease-smooth);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.custom-scrollbar::-webkit-scrollbar {
  height: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 4px;
}
</style>
