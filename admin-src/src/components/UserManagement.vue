<template>
  <div class="user-management">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl font-bold text-gray-800">用户管理</h2>
      <button
        @click="showAddForm = true"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
      >
        添加用户
      </button>
    </div>

    <div v-if="adminStore.usersLoading" class="text-center py-8 text-gray-500">
      加载中...
    </div>

    <div v-else-if="adminStore.usersError" class="text-center py-8 text-red-500">
      {{ adminStore.usersError }}
    </div>

    <div v-else-if="adminStore.users.length === 0" class="text-center py-8 text-gray-500">
      暂无用户
    </div>

    <div v-else class="overflow-x-auto">
      <table class="min-w-full bg-white border border-gray-200 rounded-lg">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              UUID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              限速 (Mbps)
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              设备限制
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              操作
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr v-for="user in adminStore.users" :key="user.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ user.id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
              {{ user.uuid }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ user.st }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {{ user.dt || '无限制' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm">
              <button
                @click="editUser(user)"
                class="text-blue-600 hover:text-blue-800 mr-3"
              >
                编辑
              </button>
              <button
                @click="deleteUser(user.id)"
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
      v-if="showAddForm || editingUser"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeForm"
    >
      <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-xl font-bold mb-4">
          {{ editingUser ? '编辑用户' : '添加用户' }}
        </h3>
        <UserForm
          :user="editingUser"
          :is-edit="!!editingUser"
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

<style scoped></style>

