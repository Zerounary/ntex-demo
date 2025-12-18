<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">
        规则类型 <span class="text-red-500">*</span>
      </label>
      <select
        v-model="form.type"
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="field">field</option>
        <option value="chinaip">chinaip</option>
        <option value="chinasites">chinasites</option>
      </select>
    </div>

    <div v-if="form.type === 'field'">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        出站标签
      </label>
      <input
        v-model="form.outbound_tag"
        type="text"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="outbound_tag"
      />
    </div>

    <div v-if="form.type === 'field'">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        域名 (每行一个)
      </label>
      <textarea
        v-model="domainText"
        rows="4"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="example.com&#10;*.example.com"
      ></textarea>
    </div>

    <div v-if="form.type === 'field'">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        IP (每行一个)
      </label>
      <textarea
        v-model="ipText"
        rows="4"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="192.168.1.1&#10;10.0.0.0/8"
      ></textarea>
    </div>

    <div v-if="form.type === 'field'">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        端口
      </label>
      <input
        v-model="form.port"
        type="text"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="80,443"
      />
    </div>

    <div v-if="form.type === 'field'">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        网络类型
      </label>
      <select
        v-model="form.network"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="">全部</option>
        <option value="tcp">TCP</option>
        <option value="udp">UDP</option>
        <option value="tcp,udp">TCP+UDP</option>
      </select>
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
import { ref, watch, computed } from 'vue';
import type { RoutingRule } from '@/api/types';

interface Props {
  rule?: RoutingRule | null;
  isEdit?: boolean;
}

interface Emits {
  (e: 'submit', rule: RoutingRule): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  rule: null,
  isEdit: false,
});

const emit = defineEmits<Emits>();

const form = ref<RoutingRule>({
  type: 'field',
  outbound_tag: undefined,
  domain: undefined,
  ip: undefined,
  port: undefined,
  network: undefined,
  source: undefined,
  protocol: undefined,
});

const domainText = ref('');
const ipText = ref('');

watch(
  () => props.rule,
  (rule) => {
    if (rule) {
      form.value = { ...rule };
      domainText.value = rule.domain?.join('\n') || '';
      ipText.value = rule.ip?.join('\n') || '';
    } else {
      form.value = {
        type: 'field',
        outbound_tag: undefined,
        domain: undefined,
        ip: undefined,
        port: undefined,
        network: undefined,
        source: undefined,
        protocol: undefined,
      };
      domainText.value = '';
      ipText.value = '';
    }
  },
  { immediate: true }
);

const handleSubmit = () => {
  const rule: RoutingRule = {
    type: form.value.type,
    outbound_tag: form.value.outbound_tag || undefined,
    port: form.value.port || undefined,
    network: form.value.network || undefined,
    source: form.value.source || undefined,
    protocol: form.value.protocol || undefined,
  };

  if (domainText.value.trim()) {
    rule.domain = domainText.value
      .split('\n')
      .map((d) => d.trim())
      .filter((d) => d);
  }

  if (ipText.value.trim()) {
    rule.ip = ipText.value
      .split('\n')
      .map((i) => i.trim())
      .filter((i) => i);
  }

  emit('submit', rule);
};
</script>

<style scoped></style>

