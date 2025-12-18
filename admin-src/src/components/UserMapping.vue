<template>
  <div class="user-mapping">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl font-bold text-gray-800">用户映射管理</h2>
      <button
        @click="showAddForm = true"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
      >
        添加映射
      </button>
    </div>

    <div v-if="adminStore.outboundsLoading" class="text-center py-8 text-gray-500">
      加载中...
    </div>

    <div v-else-if="Object.keys(adminStore.userMapping).length === 0" class="text-center py-8 text-gray-500">
      暂无用户映射
    </div>

    <div v-else class="overflow-x-auto">
      <table class="min-w-full bg-white border border-gray-200 rounded-lg">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              用户 UUID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              上游代理 Tag
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              操作
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr v-for="(outboundTag, uuid) in adminStore.userMapping" :key="uuid">
            <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
              {{ uuid }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ outboundTag }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm">
              <button
                @click="editMapping(uuid, outboundTag)"
                class="text-blue-600 hover:text-blue-800 mr-3"
              >
                编辑
              </button>
              <button
                @click="deleteMapping(uuid)"
                class="text-red-600 hover:text-red-800"
              >
                删除
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 添加/编辑表单对话框 -->
    <div
      v-if="showAddForm || editingUuid"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeForm"
    >
      <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-xl font-bold mb-4">
          {{ editingUuid ? '编辑映射' : '添加映射' }}
        </h3>
        <form @submit.prevent="handleSubmit" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              用户 UUID <span class="text-red-500">*</span>
            </label>
            <input
              v-model="form.uuid"
              type="text"
              required
              :disabled="!!editingUuid"
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
              placeholder="输入用户 UUID"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              上游代理 Tag <span class="text-red-500">*</span>
            </label>
            <select
              v-model="form.outbound_tag"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
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
          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              @click="closeForm"
              class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50"
            >
              取消
            </button>
            <button
              type="submit"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
            >
              {{ editingUuid ? '更新' : '添加' }}
            </button>
          </div>
        </form>
      </div>
    </div>
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

<style scoped></style>

