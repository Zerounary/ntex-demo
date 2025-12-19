<template>
  <div class="maintenance-mode">
    <div class="card-base p-8">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-6">
        <div class="flex-1">
          <h2 class="text-2xl font-bold text-gray-900 tracking-tight mb-2">
            Maintenance Mode
          </h2>
          <p class="text-gray-500">
            {{ description }}
          </p>
        </div>
        <div class="flex items-center gap-4">
          <div v-if="loading" class="loading-ring w-6 h-6"></div>
          <label class="relative inline-flex items-center cursor-pointer group">
            <input
              type="checkbox"
              :checked="enabled"
              :disabled="loading"
              @change="handleToggle"
              class="sr-only peer"
            />
            <div
              class="w-14 h-7 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-100 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all peer-checked:bg-primary-500 shadow-inner transition-colors"
            ></div>
            <span
              class="ml-4 text-base font-semibold transition-colors"
              :class="enabled ? 'text-primary-600' : 'text-gray-500'"
            >
              {{ enabled ? 'Enabled' : 'Disabled' }}
            </span>
          </label>
        </div>
      </div>

      <div v-if="error" class="mt-4 p-3 bg-red-50/80 border border-red-100 text-red-600 rounded-xl text-sm flex items-center gap-2">
        <div class="i-carbon-warning-filled"></div>
        {{ error }}
      </div>

      <transition name="fade">
        <div v-if="enabled" class="mt-8 p-6 bg-yellow-50/50 border border-yellow-100 rounded-2xl">
          <div class="flex items-start gap-4">
            <div class="p-2 bg-yellow-100 text-yellow-600 rounded-lg">
              <div class="i-carbon-warning-alt text-xl"></div>
            </div>
            <div>
              <p class="text-base font-bold text-yellow-800 mb-1">System in Maintenance</p>
              <p class="text-sm text-yellow-700/80 leading-relaxed">
                While maintenance mode is active, the system will skip adding new users. Only existing users will be allowed to connect. This helps protect system stability during maintenance operations.
              </p>
            </div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';
import { useToastStore } from '@/stores/toast';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();
const toastStore = useToastStore();

const loading = ref(false);

const enabled = computed(() => {
  return adminStore.maintenanceMode?.maintenance_mode || false;
});

const description = computed(() => {
  return (
    adminStore.maintenanceMode?.description ||
    'Maintenance Mode: Skips adding new users when enabled, allowing only existing users.'
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
    toastStore.success(`Maintenance mode ${newValue ? 'enabled' : 'disabled'}`);
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to set maintenance mode');
    // Revert state
    target.checked = !newValue;
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s var(--ease-smooth), transform 0.3s var(--ease-smooth);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
