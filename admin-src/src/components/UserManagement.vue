<template>
  <div class="user-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
          用户管理
        </h2>
        <p class="text-sm text-gray-500 mt-1">管理节点的用户配置</p>
      </div>
      <button
        @click="showAddForm = true"
        class="px-6 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl transition-all active-scale flex items-center gap-2"
      >
        <span>+</span>
        <span>添加用户</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.usersLoading" class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-3"></div>
        <p class="text-gray-500 text-sm">加载中...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.usersError"
      class="p-4 bg-red-50 border border-red-200 text-red-700 rounded-xl"
    >
      {{ adminStore.usersError }}
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="adminStore.users.length === 0"
      class="text-center py-16 bg-gradient-to-br from-gray-50 to-gray-100 rounded-2xl border border-gray-200"
    >
      <div class="w-20 h-20 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <span class="text-3xl">👤</span>
      </div>
      <p class="text-gray-500">暂无用户</p>
    </div>

    <!-- 用户表格 -->
    <div v-else class="bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gradient-to-r from-gray-50 to-gray-100">
            <tr>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                ID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                UUID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                限速 (Mbps)
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                设备限制
              </th>
              <th class="px-6 py-4 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider">
                操作
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-100">
            <tr
              v-for="user in adminStore.users"
              :key="user.id"
              class="hover:bg-gray-50 transition-colors"
            >
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {{ user.id }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-700">
                {{ user.uuid }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">
                <span class="px-2 py-1 bg-blue-50 text-blue-700 rounded-lg text-xs font-medium">
                  {{ user.st }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">
                <span v-if="user.dt" class="px-2 py-1 bg-purple-50 text-purple-700 rounded-lg text-xs font-medium">
                  {{ user.dt }}
                </span>
                <span v-else class="text-gray-400 text-xs">无限制</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                <div class="flex justify-end gap-2">
                  <button
                    @click="editUser(user)"
                    class="px-3 py-1.5 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-all text-xs font-medium"
                  >
                    编辑
                  </button>
                  <button
                    @click="deleteUser(user.id)"
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
        v-if="showAddForm || editingUser"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="closeForm"
      >
        <div class="bg-white rounded-2xl shadow-2xl max-w-md w-full max-h-[90vh] overflow-y-auto animate-scale-in">
          <div class="p-6 border-b border-gray-200">
            <h3 class="text-xl font-bold text-gray-800">
              {{ editingUser ? '编辑用户' : '添加用户' }}
            </h3>
          </div>
          <div class="p-6">
            <UserForm
              :user="editingUser"
              :is-edit="!!editingUser"
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
import UserForm from './UserForm.vue';
import type { User } from '@/api/types';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const showAddForm = ref(false);
const editingUser = ref<User | null>(null);

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchUsers(nodeStore.currentNodeId);
  }
});

const editUser = (user: User) => {
  editingUser.value = user;
  showAddForm.value = false;
};

const deleteUser = async (userId: number) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('确定要删除这个用户吗？')) return;

  try {
    await adminStore.deleteUser(nodeStore.currentNodeId, userId);
  } catch (err: any) {
    alert(err.message || '删除用户失败');
  }
};

const handleSubmit = async (userData: { uuid: string; st?: number; dt?: number }) => {
  if (!nodeStore.currentNodeId) return;

  try {
    if (editingUser.value) {
      await adminStore.updateUser(
        nodeStore.currentNodeId,
        editingUser.value.id,
        userData
      );
    } else {
      await adminStore.addUser(nodeStore.currentNodeId, userData);
    }
    closeForm();
  } catch (err: any) {
    alert(err.message || '操作失败');
  }
};

const closeForm = () => {
  showAddForm.value = false;
  editingUser.value = null;
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
