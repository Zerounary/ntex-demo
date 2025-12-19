<template>
  <div
    class="toast-item pointer-events-auto w-full max-w-sm overflow-hidden rounded-xl shadow-lg border flex items-start gap-3 p-4 transition-all duration-300 relative group"
    :class="typeClasses"
    role="alert"
  >
    <!-- Icon -->
    <div class="flex-shrink-0 mt-0.5">
      <div :class="iconClass" class="text-lg"></div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <p class="text-sm font-medium leading-5">
        {{ toast.message }}
      </p>
    </div>

    <!-- Close Button -->
    <button
      @click="$emit('close')"
      class="flex-shrink-0 ml-4 text-current opacity-60 hover:opacity-100 transition-opacity focus:outline-none"
    >
      <div class="i-carbon-close text-lg"></div>
    </button>
    
    <!-- Progress Bar (Optional, for auto-dismiss) -->
    <div 
      v-if="toast.duration && toast.duration > 0"
      class="absolute bottom-0 left-0 h-1 bg-current opacity-20"
      :style="{ animation: `shrink ${toast.duration}ms linear forwards` }"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Toast } from '@/stores/toast';

interface Props {
  toast: Toast;
}

interface Emits {
  (e: 'close'): void;
}

const props = defineProps<Props>();
defineEmits<Emits>();

const typeClasses = computed(() => {
  switch (props.toast.type) {
    case 'success':
      return 'bg-white border-green-100 text-green-800 shadow-green-500/10';
    case 'error':
      return 'bg-white border-red-100 text-red-800 shadow-red-500/10';
    case 'warning':
      return 'bg-white border-yellow-100 text-yellow-800 shadow-yellow-500/10';
    case 'info':
    default:
      return 'bg-white border-blue-100 text-blue-800 shadow-blue-500/10';
  }
});

const iconClass = computed(() => {
  switch (props.toast.type) {
    case 'success':
      return 'i-carbon-checkmark-filled text-green-500';
    case 'error':
      return 'i-carbon-warning-filled text-red-500';
    case 'warning':
      return 'i-carbon-warning-alt-filled text-yellow-500';
    case 'info':
    default:
      return 'i-carbon-information-filled text-blue-500';
  }
});
</script>

<style scoped>
@keyframes shrink {
  from { width: 100%; }
  to { width: 0%; }
}
</style>
