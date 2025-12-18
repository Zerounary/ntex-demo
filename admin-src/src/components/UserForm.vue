<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        UUID <span class="text-red-500">*</span>
      </label>
      <input
        v-model="form.uuid"
        type="text"
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="输入用户 UUID"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        限速 (Mbps)
      </label>
      <input
        v-model.number="form.st"
        type="number"
        min="0"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="默认: 1"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        设备限制
      </label>
      <input
        v-model.number="form.dt"
        type="number"
        min="0"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="默认: 0 (无限制)"
      />
    </div>

    <div class="flex justify-end space-x-3 pt-4">
      <button
        type="button"
        @click="$emit('cancel')"
        class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50"
      >
        取消
      </button>
      <button
        type="submit"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
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

