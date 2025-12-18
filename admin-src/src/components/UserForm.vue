<template>
  <form @submit.prevent="handleSubmit" class="space-y-5">
    <div>
      <label class="block text-sm font-semibold text-gray-700 mb-2">
        UUID <span class="text-red-400">*</span>
      </label>
      <input
        v-model="form.uuid"
        type="text"
        required
        class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
        placeholder="输入用户 UUID"
      />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          限速 (Mbps)
        </label>
        <input
          v-model.number="form.st"
          type="number"
          min="0"
          class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
          placeholder="默认: 1"
        />
      </div>

      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          设备限制
        </label>
        <input
          v-model.number="form.dt"
          type="number"
          min="0"
          class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
          placeholder="默认: 0 (无限制)"
        />
      </div>
    </div>

    <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
      <button
        type="button"
        @click="$emit('cancel')"
        class="px-6 py-2.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-xl hover:bg-gray-50 hover:shadow-md transition-all active-scale"
      >
        取消
      </button>
      <button
        type="submit"
        class="px-6 py-2.5 text-sm font-medium text-white bg-gradient-to-r from-indigo-500 to-purple-500 rounded-xl hover:shadow-lg transition-all active-scale"
      >
        {{ isEdit ? '更新' : '添加' }}
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
