<template>
  <div class="outbound-management space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">
          Inbounds
        </h2>
        <p class="text-sm text-gray-500 mt-1">Manage inbound listeners</p>
      </div>
      <button
        @click="showAddForm = true"
        class="btn-primary flex items-center gap-2 shadow-lg hover:shadow-xl hover:-translate-y-0.5"
      >
        <div class="i-carbon-add text-lg"></div>
        <span>Add Inbound</span>
      </button>
    </div>

    <div v-if="adminStore.inboundsLoading" class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="loading-ring w-10 h-10 relative mx-auto mb-4"></div>
        <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
      </div>
    </div>

    <div
      v-else-if="adminStore.inboundsError"
      class="p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-xl flex items-center gap-3"
    >
      <div class="i-carbon-warning-alt text-lg"></div>
      <span class="font-medium">{{ adminStore.inboundsError }}</span>
    </div>

    <div
      v-else-if="adminStore.inbounds.length === 0"
      class="flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed"
    >
      <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 text-gray-400">
        <div class="i-carbon-plug text-3xl"></div>
      </div>
      <h3 class="text-lg font-bold text-gray-700 mb-1">No Inbounds Found</h3>
      <p class="text-gray-400 text-sm">Add an inbound listener to get started</p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <transition-group name="list" tag="div" class="contents">
        <div
          v-for="(inbound, index) in adminStore.inbounds"
          :key="inbound.tag"
          :style="{ 'animation-delay': `${index * 50}ms` }"
          class="card-base p-6 hover:shadow-md transition-all animate-slide-up group"
        >
          <div class="flex justify-between items-start mb-4">
            <div class="flex-1">
              <h3 class="text-lg font-bold text-gray-900 mb-1 flex items-center gap-2">
                {{ inbound.tag }}
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-gray-100 text-gray-500 uppercase tracking-wider">
                  {{ inbound.protocol }}
                </span>
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-primary-50 text-primary-600 uppercase tracking-wider">
                  :{{ inbound.port }}
                </span>
              </h3>
              <p v-if="inbound.listen" class="text-xs text-gray-400 font-mono">{{ inbound.listen }}</p>
            </div>
            <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click="editInbound(inbound)"
                class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                title="Edit"
              >
                <div class="i-carbon-edit text-lg"></div>
              </button>
              <button
                @click="deleteInbound(inbound.tag)"
                class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                title="Delete"
              >
                <div class="i-carbon-trash-can text-lg"></div>
              </button>
            </div>
          </div>

          <details class="cursor-pointer group/details">
            <summary class="text-xs font-medium text-gray-400 hover:text-primary-600 transition-colors flex items-center gap-1 select-none">
              <div class="i-carbon-chevron-right group-open/details:rotate-90 transition-transform"></div>
              View Configuration
            </summary>
            <div class="mt-3 p-3 bg-gray-900 rounded-xl overflow-hidden shadow-inner space-y-3">
              <pre class="text-[10px] text-gray-300 font-mono overflow-x-auto custom-scrollbar">{{ JSON.stringify(inbound.settings, null, 2) }}</pre>
              <pre class="text-[10px] text-gray-300 font-mono overflow-x-auto custom-scrollbar">{{ JSON.stringify(inbound.stream_settings ?? null, null, 2) }}</pre>
              <pre class="text-[10px] text-gray-300 font-mono overflow-x-auto custom-scrollbar">{{ JSON.stringify(inbound.sniffing ?? null, null, 2) }}</pre>
            </div>
          </details>
        </div>
      </transition-group>
    </div>

    <transition name="modal">
      <div
        v-if="showAddForm || editingInbound"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-3xl w-full max-h-[90vh] overflow-y-auto animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <h3 class="text-lg font-bold text-gray-900">
              {{ editingInbound ? 'Edit Inbound' : 'Add New Inbound' }}
            </h3>
            <button @click="closeForm" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
          </div>
          <div class="p-6">
            <InboundForm
              :inbound="editingInbound"
              :is-edit="!!editingInbound"
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
import type { InboundConfig } from '@/api/types';
import InboundForm from './InboundForm.vue';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();
const toastStore = useToastStore();

const showAddForm = ref(false);
const editingInbound = ref<InboundConfig | null>(null);

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchInbounds(nodeStore.currentNodeId);
  }
});

const editInbound = (inbound: InboundConfig) => {
  editingInbound.value = inbound;
  showAddForm.value = false;
};

const deleteInbound = async (tag: string) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('Are you sure you want to delete this inbound?')) return;

  try {
    await adminStore.deleteInbound(nodeStore.currentNodeId, tag);
    toastStore.success('Inbound deleted successfully');
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to delete inbound');
  }
};

const handleSubmit = async (inboundData: {
  tag: string;
  protocol: string;
  port: number;
  listen?: string | null;
  settings: any;
  stream_settings?: any;
  sniffing?: any;
}) => {
  if (!nodeStore.currentNodeId) return;

  try {
    if (editingInbound.value) {
      await adminStore.updateInbound(nodeStore.currentNodeId, editingInbound.value.tag, {
        protocol: inboundData.protocol,
        port: inboundData.port,
        listen: inboundData.listen,
        settings: inboundData.settings,
        stream_settings: inboundData.stream_settings,
        sniffing: inboundData.sniffing,
      });
      toastStore.success('Inbound updated successfully');
    } else {
      await adminStore.addInbound(nodeStore.currentNodeId, inboundData);
      toastStore.success('Inbound added successfully');
    }
    closeForm();
  } catch (err: any) {
    toastStore.error(err.message || 'Operation failed');
  }
};

const closeForm = () => {
  showAddForm.value = false;
  editingInbound.value = null;
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
