<template>
  <div class="user-mapping space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
          用户映射管理
        </h2>
        <p class="text-sm text-gray-500 mt-1">配置用户到上游代理的映射关系</p>
      </div>
      <button
        @click="showAddForm = true"
        class="px-6 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl transition-all active-scale flex items-center gap-2"
      >
        <span>+</span>
        <span>添加映射</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.outboundsLoading" class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-3"></div>
        <p class="text-gray-500 text-sm">加载中...</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="Object.keys(adminStore.userMapping).length === 0"
      class="text-center py-16 bg-gradient-to-br from-gray-50 to-gray-100 rounded-2xl border border-gray-200"
    >
      <div class="w-20 h-20 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <span class="text-3xl">🔗</span>
      </div>
      <p class="text-gray-500">暂无用户映射</p>
    </div>

    <!-- 映射表格 -->
    <div v-else class="bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gradient-to-r from-gray-50 to-gray-100">
            <tr>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                用户 UUID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                上游代理 Tag
              </th>
              <th class="px-6 py-4 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider">
                操作
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-100">
            <tr
              v-for="(outboundTag, uuid) in adminStore.userMapping"
              :key="uuid"
              class="hover:bg-gray-50 transition-colors"
            >
              <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-700">
                {{ uuid }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm">
                <span class="px-3 py-1 bg-indigo-50 text-indigo-700 rounded-lg text-xs font-medium">
                  {{ outboundTag }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                <div class="flex justify-end gap-2">
                  <button
                    @click="editMapping(uuid, outboundTag)"
                    class="px-3 py-1.5 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-all text-xs font-medium"
                  >
                    编辑
                  </button>
                  <button
                    @click="deleteMapping(uuid)"
                    class="px-3 py-1.5 text-red-600 hover:bg-red-50 rounded-lg transition-all text-xs font-medium"
                  >
                    删除
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
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="closeForm"
      >
        <div class="bg-white rounded-2xl shadow-2xl max-w-md w-full animate-scale-in">
          <div class="p-6 border-b border-gray-200">
            <h3 class="text-xl font-bold text-gray-800">
              {{ editingUuid ? '编辑映射' : '添加映射' }}
            </h3>
          </div>
          <div class="p-6">
            <form @submit.prevent="handleSubmit" class="space-y-5">
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">
                  用户 UUID <span class="text-red-400">*</span>
                </label>
                <input
                  v-model="form.uuid"
                  type="text"
                  required
                  :disabled="!!editingUuid"
                  placeholder="输入用户 UUID"
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all disabled:bg-gray-100"
                />
              </div>
              <div>
                <label class="block text-sm font-semibold text-gray-700 mb-2">
                  上游代理 Tag <span class="text-red-400">*</span>
                </label>
                <select
                  v-model="form.outbound_tag"
                  required
                  class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
                >
                  <option value="">请选择代理</option>
                  <option
                    v-for="outbound in adminStore.outbounds"
                    :key="outbound.tag"
                    :value="outbound.tag"
                  >
                    {{ outbound.tag }} ({{ outbound.protocol }})
                  </option>
                </select>
              </div>
              <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
                <button
                  type="button"
                  @click="closeForm"
                  class="px-6 py-2.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-xl hover:bg-gray-50 hover:shadow-md transition-all active-scale"
                >
                  取消
                </button>
                <button
                  type="submit"
                  class="px-6 py-2.5 text-sm font-medium text-white bg-gradient-to-r from-indigo-500 to-purple-500 rounded-xl hover:shadow-lg transition-all active-scale"
                >
                  {{ editingUuid ? '更新' : '添加' }}
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
import { ref, onMounted } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const showAddForm = ref(false);
const editingUuid = ref<string | null>(null);

const form = ref({
  uuid: '',
  outbound_tag: '',
});

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchOutbounds(nodeStore.currentNodeId);
  }
});

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
  if (!confirm('确定要删除这个映射吗？')) return;

  try {
    await adminStore.deleteMapping(nodeStore.currentNodeId, uuid);
  } catch (err: any) {
    alert(err.message || '删除映射失败');
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
    } else {
      await adminStore.addMapping(
        nodeStore.currentNodeId,
        form.value.uuid,
        form.value.outbound_tag
      );
    }
    closeForm();
  } catch (err: any) {
    alert(err.message || '操作失败');
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
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
