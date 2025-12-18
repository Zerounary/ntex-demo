<template>
  <div class="user-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">
          User Management
        </h2>
        <p class="text-sm text-gray-500 mt-1">Configure node users and limits</p>
      </div>
      <button
        @click="showAddForm = true"
        class="btn-primary flex items-center gap-2 shadow-lg hover:shadow-xl hover:-translate-y-0.5"
      >
        <div class="i-carbon-add text-lg"></div>
        <span>Add User</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.usersLoading" class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="loading-ring w-10 h-10 relative mx-auto mb-4"></div>
        <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.usersError"
      class="p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-xl flex items-center gap-3"
    >
      <div class="i-carbon-warning-alt text-lg"></div>
      <span class="font-medium">{{ adminStore.usersError }}</span>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="adminStore.users.length === 0"
      class="flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed"
    >
      <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 text-gray-400">
        <div class="i-carbon-user text-3xl"></div>
      </div>
      <h3 class="text-lg font-bold text-gray-700 mb-1">No Users Found</h3>
      <p class="text-gray-400 text-sm">Add a user to get started</p>
    </div>

    <!-- 用户表格 -->
    <div v-else class="card-base overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-100">
          <thead class="bg-gray-50/80">
            <tr>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                ID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                UUID
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Speed Limit
              </th>
              <th class="px-6 py-4 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Device Limit
              </th>
              <th class="px-6 py-4 text-right text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-50">
            <tr
              v-for="user in adminStore.users"
              :key="user.id"
              class="hover:bg-gray-50/80 transition-colors group"
            >
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                #{{ user.id }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-mono text-gray-600 bg-gray-100 px-2 py-1 rounded select-all">{{ user.uuid }}</span>
                </div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center gap-1.5">
                  <span class="px-2.5 py-1 bg-blue-50 text-blue-700 rounded-lg text-xs font-semibold border border-blue-100">
                    {{ user.st }} Mbps
                  </span>
                </div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span v-if="user.dt" class="px-2.5 py-1 bg-purple-50 text-purple-700 rounded-lg text-xs font-semibold border border-purple-100">
                  {{ user.dt }} Devices
                </span>
                <span v-else class="text-gray-400 text-xs font-medium px-2 py-1 bg-gray-50 rounded-lg">Unlimited</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                <div class="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    @click="editUser(user)"
                    class="p-1.5 text-gray-500 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <div class="i-carbon-edit text-lg"></div>
                  </button>
                  <button
                    @click="deleteUser(user.id)"
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
        v-if="showAddForm || editingUser"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
        @click.self="closeForm"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-md w-full max-h-[90vh] overflow-y-auto animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <h3 class="text-lg font-bold text-gray-900">
              {{ editingUser ? 'Edit User' : 'Add New User' }}
            </h3>
            <button @click="closeForm" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
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
  if (!confirm('Are you sure you want to delete this user?')) return;

  try {
    await adminStore.deleteUser(nodeStore.currentNodeId, userId);
  } catch (err: any) {
    alert(err.message || 'Failed to delete user');
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
    alert(err.message || 'Operation failed');
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
  transition: opacity 0.3s var(--ease-smooth);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
