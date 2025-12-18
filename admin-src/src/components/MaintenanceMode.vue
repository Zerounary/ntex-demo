<template>
  <div class="maintenance-mode">
    <div class="bg-white rounded-2xl p-8 border border-gray-100 shadow-lg">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-6">
        <div class="flex-1">
          <h2 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent mb-2">
            维护模式
          </h2>
          <p class="text-sm text-gray-500">
            {{ description }}
          </p>
        </div>
        <div class="flex items-center gap-4">
          <div v-if="loading" class="w-6 h-6 border-2 border-indigo-200 border-t-indigo-500 rounded-full animate-spin"></div>
          <label class="relative inline-flex items-center cursor-pointer group">
            <input
              type="checkbox"
              :checked="enabled"
              :disabled="loading"
              @change="handleToggle"
              class="sr-only peer"
            />
            <div
              class="w-14 h-7 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all peer-checked:bg-gradient-to-r peer-checked:from-yellow-400 peer-checked:to-orange-500 shadow-inner"
            ></div>
            <span
              class="ml-4 text-base font-semibold transition-colors"
              :class="enabled ? 'text-orange-600' : 'text-gray-600'"
            >
              {{ enabled ? '已启用' : '已禁用' }}
            </span>
          </label>
        </div>
      </div>

      <div v-if="error" class="mt-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-lg text-sm">
        {{ error }}
      </div>

      <div v-if="enabled" class="mt-6 p-4 bg-yellow-50 border border-yellow-200 rounded-xl">
        <div class="flex items-start gap-3">
          <span class="text-xl">⚠️</span>
          <div>
            <p class="text-sm font-medium text-yellow-800 mb-1">维护模式已启用</p>
            <p class="text-xs text-yellow-700">
              在此模式下，系统将跳过新用户添加，仅允许已存在的用户连接。这有助于在维护期间保护系统稳定性。
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const loading = ref(false);

const enabled = computed(() => {
  return adminStore.maintenanceMode?.maintenance_mode || false;
});

const description = computed(() => {
  return (
    adminStore.maintenanceMode?.description ||
    '维护模式：开启时跳过新用户添加，仅允许已存在用户'
  );
});

const error = computed(() => {
  return adminStore.maintenanceError;
});

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchMaintenanceMode(nodeStore.currentNodeId);
  }
});

const handleToggle = async (event: Event) => {
  if (!nodeStore.currentNodeId) return;

  const target = event.target as HTMLInputElement;
  const newValue = target.checked;

  loading.value = true;
  try {
    await adminStore.setMaintenanceMode(nodeStore.currentNodeId, newValue);
  } catch (err: any) {
    alert(err.message || '设置维护模式失败');
    // 恢复原状态
    target.checked = !newValue;
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped></style>
