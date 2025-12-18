<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <div>
      <label class="block text-sm font-semibold text-gray-700 mb-2">
        UUID <span class="text-red-400">*</span>
      </label>
      <input
        v-model="form.uuid"
        type="text"
        required
        class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all font-mono text-gray-700"
        placeholder="Enter user UUID"
      />
    </div>

    <div class="grid grid-cols-2 gap-5">
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          Speed Limit (Mbps)
        </label>
        <div class="relative">
          <input
            v-model.number="form.st"
            type="number"
            min="0"
            class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10"
            placeholder="Default: 1"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-meter text-lg"></div>
          </div>
        </div>
      </div>

      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          Device Limit
        </label>
        <div class="relative">
          <input
            v-model.number="form.dt"
            type="number"
            min="0"
            class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10"
            placeholder="Default: 0"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-devices text-lg"></div>
          </div>
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-3 pt-4 border-t border-gray-100">
      <button
        type="button"
        @click="$emit('cancel')"
        class="btn-ghost"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="btn-primary flex items-center gap-2 shadow-lg shadow-primary-500/20"
      >
        <div v-if="isEdit" class="i-carbon-save text-lg"></div>
        <div v-else class="i-carbon-add text-lg"></div>
        <span>{{ isEdit ? 'Update User' : 'Add User' }}</span>
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { User } from '@/api/types';

interface Props {
  user?: User | null;
  isEdit?: boolean;
}

interface Emits {
  (e: 'submit', user: { uuid: string; st?: number; dt?: number }): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  user: null,
  isEdit: false,
});

const emit = defineEmits<Emits>();

const form = ref({
  uuid: '',
  st: 1,
  dt: 0,
});

watch(
  () => props.user,
  (user) => {
    if (user) {
      form.value = {
        uuid: user.uuid,
        st: user.st,
        dt: user.dt,
      };
    } else {
      form.value = {
        uuid: '',
        st: 1,
        dt: 0,
      };
    }
  },
  { immediate: true }
);

const handleSubmit = () => {
  emit('submit', {
    uuid: form.value.uuid,
    st: form.value.st || 1,
    dt: form.value.dt || 0,
  });
};
</script>

<style scoped></style>
