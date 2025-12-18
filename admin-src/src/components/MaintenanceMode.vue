<template>
  <div class="maintenance-mode">
    <div class="bg-white border border-gray-200 rounded-lg p-6">
      <div class="flex justify-between items-center mb-4">
        <div>
          <h2 class="text-2xl font-bold text-gray-800">维护模式</h2>
          <p class="text-sm text-gray-500 mt-1">
            {{ description }}
          </p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            :checked="enabled"
            :disabled="loading"
            @change="handleToggle"
            class="sr-only peer"
          />
          <div
            class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"
          ></div>
          <span class="ml-3 text-sm font-medium text-gray-700">
            {{ enabled ? '已启用' : '已禁用' }}
          </span>
        </label>
      </div>

      <div v-if="loading" class="text-sm text-gray-500">加载中...</div>
      <div v-else-if="error" class="text-sm text-red-500">{{ error }}</div>
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

