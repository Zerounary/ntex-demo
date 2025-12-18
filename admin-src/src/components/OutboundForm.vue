<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        Tag <span class="text-red-500">*</span>
      </label>
      <input
        v-model="form.tag"
        type="text"
        required
        :disabled="isEdit"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
        placeholder="输入代理标签"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        协议
      </label>
      <select
        v-model="form.protocol"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="shadowsocks">Shadowsocks</option>
        <option value="vmess">VMess</option>
        <option value="vless">VLESS</option>
        <option value="trojan">Trojan</option>
        <option value="socks">SOCKS</option>
        <option value="http">HTTP</option>
      </select>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        配置 (JSON)
      </label>
      <textarea
        v-model="settingsJson"
        rows="8"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
        placeholder='{"address": "example.com", "port": 443, ...}'
      ></textarea>
      <p v-if="settingsError" class="mt-1 text-sm text-red-500">
        {{ settingsError }}
      </p>
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
        :disabled="!!settingsError"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-400"
      >
        {{ isEdit ? '更新' : '添加' }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { OutboundConfig } from '@/api/types';

interface Props {
  outbound?: OutboundConfig | null;
  isEdit?: boolean;
}

interface Emits {
  (e: 'submit', outbound: { tag: string; protocol: string; settings: any }): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  outbound: null,
  isEdit: false,
});

const emit = defineEmits<Emits>();

const form = ref({
  tag: '',
  protocol: 'shadowsocks',
});

const settingsJson = ref('{}');
const settingsError = ref('');

watch(
  () => props.outbound,
  (outbound) => {
    if (outbound) {
      form.value = {
        tag: outbound.tag,
        protocol: outbound.protocol,
      };
      settingsJson.value = JSON.stringify(outbound.settings, null, 2);
    } else {
      form.value = {
        tag: '',
        protocol: 'shadowsocks',
      };
      settingsJson.value = '{}';
    }
    settingsError.value = '';
  },
  { immediate: true }
);

watch(settingsJson, (value) => {
  try {
    JSON.parse(value);
    settingsError.value = '';
  } catch (e) {
    settingsError.value = '无效的 JSON 格式';
  }
});

const handleSubmit = () => {
  try {
    const settings = JSON.parse(settingsJson.value);
    emit('submit', {
      tag: form.value.tag,
      protocol: form.value.protocol,
      settings,
    });
  } catch (e) {
    settingsError.value = 'JSON 解析失败';
  }
};
</script>

<style scoped></style>

