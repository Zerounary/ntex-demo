<template>
  <div class="outbound-management">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl font-bold text-gray-800">上游代理管理</h2>
      <button
        @click="showAddForm = true"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
      >
        添加代理
      </button>
    </div>

    <div v-if="adminStore.outboundsLoading" class="text-center py-8 text-gray-500">
      加载中...
    </div>

    <div v-else-if="adminStore.outboundsError" class="text-center py-8 text-red-500">
      {{ adminStore.outboundsError }}
    </div>

    <div v-else-if="adminStore.outbounds.length === 0" class="text-center py-8 text-gray-500">
      暂无上游代理
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="outbound in adminStore.outbounds"
        :key="outbound.tag"
        class="bg-white border border-gray-200 rounded-lg p-4"
      >
        <div class="flex justify-between items-start mb-2">
          <div>
            <h3 class="text-lg font-semibold text-gray-800">{{ outbound.tag }}</h3>
            <p class="text-sm text-gray-500">协议: {{ outbound.protocol }}</p>
          </div>
          <div class="flex space-x-2">
            <button
              @click="testLatency(outbound.tag)"
              :disabled="testingLatency === outbound.tag"
              class="px-3 py-1 text-sm bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-gray-400"
            >
              {{ testingLatency === outbound.tag ? '测试中...' : '测试延迟' }}
            </button>
            <button
              @click="editOutbound(outbound)"
              class="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
            >
              编辑
            </button>
            <button
              @click="deleteOutbound(outbound.tag)"
              class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
            >
              删除
            </button>
          </div>
        </div>
        <div class="mt-2">
          <details class="cursor-pointer">
            <summary class="text-sm text-gray-600">查看配置</summary>
            <pre
              class="mt-2 p-3 bg-gray-50 rounded text-xs overflow-x-auto"
            >{{ JSON.stringify(outbound.settings, null, 2) }}</pre>
          </details>
        </div>
        <div v-if="latencyResults[outbound.tag]" class="mt-2 text-sm">
          <span class="text-gray-600">UDP 延迟: </span>
          <span
            :class="
              latencyResults[outbound.tag].latency
                ? 'text-green-600 font-medium'
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
      </div>
    </div>

    <!-- 添加/编辑表单对话框 -->
    <div
      v-if="showAddForm || editingOutbound"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeForm"
    >
      <div class="bg-white rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <h3 class="text-xl font-bold mb-4">
          {{ editingOutbound ? '编辑代理' : '添加代理' }}
        </h3>
        <OutboundForm
          :outbound="editingOutbound"
          :is-edit="!!editingOutbound"
          @submit="handleSubmit"
          @cancel="closeForm"
        />
      </div>
    </div>
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

<style scoped></style>

