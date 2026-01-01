<template>
  <div class="user-mapping space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">
          User Mapping
        </h2>
        <p class="text-sm text-gray-500 mt-1">Map users to specific outbound proxies</p>
      </div>
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="w-9 h-9 rounded-xl flex items-center justify-center text-gray-600 hover:text-gray-800 hover:bg-gray-100/50 transition-colors disabled:opacity-50"
          @click="refreshMapping"
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
          <span>Add Mapping</span>
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

    <!-- 空状态 -->
    <div
      v-else-if="Object.keys(adminStore.userMapping).length === 0"
      class="flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed"
    >
      <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 text-gray-400">
        <div class="i-carbon-flow-stream text-3xl"></div>
      </div>
      <h3 class="text-lg font-bold text-gray-700 mb-1">No Mappings Configured</h3>
      <p class="text-gray-400 text-sm">Add a mapping to route user traffic</p>
    </div>

    <!-- 映射表格 -->
    <div v-else class="card-base overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-100">
          <thead class="bg-gray-50/80">
            <tr>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                User UUID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Outbound Tag
              </th>
              <th class="px-6 py-4 text-right text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-50">
            <tr
              v-for="(outboundTag, uuid) in adminStore.userMapping"
              :key="uuid"
              class="hover:bg-gray-50/80 transition-colors group"
            >
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="text-sm font-mono text-gray-600 bg-gray-100 px-2 py-1 rounded select-all">{{ uuid }}</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2.5 py-1 bg-indigo-50 text-indigo-700 rounded-lg text-xs font-semibold border border-indigo-100 flex items-center gap-1 w-fit">
                   <div class="i-carbon-arrow-right text-[10px]"></div>
                  {{ outboundTag }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                <div class="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    @click="editMapping(uuid as string, outboundTag as string)"
                    class="p-1.5 text-gray-500 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <div class="i-carbon-edit text-lg"></div>
                  </button>
                  <button
                    @click="deleteMapping(uuid as string)"
                    class="p-1.5 text-gray-500 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <div class="i-carbon-trash-can text-lg"></div>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 添加/编辑表单对话框 -->
    <transition name="modal">
      <div
        v-if="showAddForm || editingUuid"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-md w-full animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center rounded-t-2xl">
            <h3 class="text-lg font-bold text-gray-900">
              {{ editingUuid ? 'Edit Mapping' : 'Add Mapping' }}
            </h3>
            <button @click="closeForm" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
          </div>
          <div class="p-6">
            <form @submit.prevent="handleSubmit" class="space-y-6">
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">
                  User UUID <span class="text-red-400">*</span>
                </label>
                <input
                  v-model="form.uuid"
                  type="text"
                  required
                  :disabled="!!editingUuid"
                  placeholder="Enter User UUID"
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm font-mono focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all disabled:bg-gray-100 disabled:text-gray-500"
                />
              </div>
              <div>
                <BaseSelect
                  v-model="form.outbound_tag"
                  :options="outboundOptions"
                  label="Outbound Tag"
                  required
                />
              </div>
              <div class="flex justify-end gap-3 pt-4 border-t border-gray-100">
                <button
                  type="button"
                  @click="closeForm"
                  class="btn-ghost"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  class="btn-primary shadow-lg shadow-primary-500/20"
                >
                  {{ editingUuid ? 'Update' : 'Add' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';
import { useToastStore } from '@/stores/toast';
import BaseSelect from '@/components/BaseSelect.vue';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();
const toastStore = useToastStore();

const showAddForm = ref(false);
const editingUuid = ref<string | null>(null);
const refreshing = ref(false);

const form = ref({
  uuid: '',
  outbound_tag: '',
});

const outboundOptions = computed(() => {
  return adminStore.outbounds.map(outbound => ({
    label: `${outbound.tag} (${outbound.protocol})`,
    value: outbound.tag,
  }));
});

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchOutbounds(nodeStore.currentNodeId);
  }
});

const refreshMapping = async () => {
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

const editMapping = (uuid: string, outboundTag: string) => {
  editingUuid.value = uuid;
  form.value = {
    uuid,
    outbound_tag: outboundTag,
  };
  showAddForm.value = false;
};

const deleteMapping = async (uuid: string) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('Are you sure you want to delete this mapping?')) return;

  try {
    await adminStore.deleteMapping(nodeStore.currentNodeId, uuid);
    toastStore.success('Mapping deleted successfully');
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to delete mapping');
  }
};

const handleSubmit = async () => {
  if (!nodeStore.currentNodeId) return;

  try {
    if (editingUuid.value) {
      await adminStore.updateMapping(
        nodeStore.currentNodeId,
        form.value.uuid,
        form.value.outbound_tag
      );
      toastStore.success('Mapping updated successfully');
    } else {
      await adminStore.addMapping(
        nodeStore.currentNodeId,
        form.value.uuid,
        form.value.outbound_tag
      );
      toastStore.success('Mapping added successfully');
    }
    closeForm();
  } catch (err: any) {
    toastStore.error(err.message || 'Operation failed');
  }
};

const closeForm = () => {
  showAddForm.value = false;
  editingUuid.value = null;
  form.value = {
    uuid: '',
    outbound_tag: '',
  };
};
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s var(--ease-smooth);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
