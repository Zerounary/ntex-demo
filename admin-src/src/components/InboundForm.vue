<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <div>
        <label class="block text-xs font-medium text-gray-600 mb-1.5">
          Tag <span class="text-red-400">*</span>
        </label>
        <div class="relative">
          <input
            v-model="form.tag"
            type="text"
            required
            :disabled="isEdit"
            placeholder="Enter inbound tag"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm pl-10"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-tag text-lg"></div>
          </div>
        </div>
      </div>

      <div>
        <label class="block text-xs font-medium text-gray-600 mb-1.5">
          Port <span class="text-red-400">*</span>
        </label>
        <div class="relative">
          <input
            v-model.number="form.port"
            type="number"
            min="1"
            max="65535"
            required
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm pl-10"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-connection-signal text-lg"></div>
          </div>
        </div>
      </div>

      <div>
        <BaseSelect
          v-model="form.protocol"
          :options="protocolOptions"
          label="Protocol"
          required
        >
          <template #icon>
            <div class="i-carbon-network-4 text-lg"></div>
          </template>
        </BaseSelect>
      </div>

      <div>
        <label class="block text-xs font-medium text-gray-600 mb-1.5">Listen</label>
        <div class="relative">
          <input
            v-model="form.listen"
            type="text"
            placeholder="0.0.0.0 (optional)"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm pl-10"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-location text-lg"></div>
          </div>
        </div>
      </div>
    </div>

    <div class="bg-white rounded-xl p-4 border border-gray-200 shadow-sm">
      <div class="flex items-center justify-between mb-3">
        <label class="flex items-center gap-2 text-sm font-semibold text-gray-700 cursor-pointer select-none group">
          <input
            v-model="showAdvanced"
            type="checkbox"
            class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
          />
          <span class="group-hover:text-primary-600 transition-colors">Advanced Mode (JSON)</span>
        </label>
      </div>

      <transition name="fade">
        <div v-if="showAdvanced" class="space-y-4">
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">Settings</label>
            <textarea
              v-model="settingsJson"
              rows="6"
              class="w-full px-4 py-3 bg-gray-900 text-gray-300 border border-gray-700 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all custom-scrollbar resize-y"
              placeholder="{}"
            ></textarea>
          </div>

          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">Stream Settings</label>
            <textarea
              v-model="streamSettingsJson"
              rows="6"
              class="w-full px-4 py-3 bg-gray-900 text-gray-300 border border-gray-700 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all custom-scrollbar resize-y"
              placeholder="null"
            ></textarea>
          </div>

          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">Sniffing</label>
            <textarea
              v-model="sniffingJson"
              rows="4"
              class="w-full px-4 py-3 bg-gray-900 text-gray-300 border border-gray-700 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all custom-scrollbar resize-y"
              placeholder="null"
            ></textarea>
          </div>

          <p v-if="jsonError" class="text-xs text-red-500 flex items-center gap-1">
            <div class="i-carbon-warning-filled"></div>
            {{ jsonError }}
          </p>
        </div>
      </transition>
    </div>

    <div class="flex justify-end gap-3 pt-4 border-t border-gray-100">
      <button type="button" @click="$emit('cancel')" class="btn-ghost">Cancel</button>
      <button
        type="submit"
        :disabled="!isFormValid"
        class="btn-primary shadow-lg shadow-primary-500/20 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none"
      >
        {{ isEdit ? 'Update Inbound' : 'Add Inbound' }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import BaseSelect from '@/components/BaseSelect.vue';
import type { InboundConfig } from '@/api/types';

interface Props {
  inbound?: InboundConfig | null;
  isEdit?: boolean;
}

interface Emits {
  (
    e: 'submit',
    inbound: {
      tag: string;
      protocol: string;
      port: number;
      listen?: string | null;
      settings: any;
      stream_settings?: any;
      sniffing?: any;
    }
  ): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  inbound: null,
  isEdit: false,
});

const emit = defineEmits<Emits>();

const form = ref({
  tag: '',
  protocol: 'vmess',
  port: 10086,
  listen: '',
});

const showAdvanced = ref(false);
const settingsJson = ref('{}');
const streamSettingsJson = ref('null');
const sniffingJson = ref('null');
const jsonError = ref('');

const protocolOptions = [
  { label: 'VMess', value: 'vmess' },
  { label: 'VLESS', value: 'vless' },
  { label: 'Trojan', value: 'trojan' },
  { label: 'Shadowsocks', value: 'shadowsocks' },
  { label: 'SOCKS', value: 'socks' },
  { label: 'HTTP', value: 'http' },
  { label: 'Dokodemo-Door', value: 'dokodemo-door' },
];

const isFormValid = computed(() => {
  if (!form.value.tag.trim()) return false;
  if (!form.value.port || form.value.port < 1 || form.value.port > 65535) return false;
  if (!form.value.protocol) return false;
  if (jsonError.value) return false;
  return true;
});

const validateJson = () => {
  if (!showAdvanced.value) {
    jsonError.value = '';
    return;
  }

  try {
    JSON.parse(settingsJson.value);
  } catch {
    jsonError.value = 'Settings JSON parse error';
    return;
  }

  try {
    const v = JSON.parse(streamSettingsJson.value);
    if (v !== null && typeof v !== 'object') {
      jsonError.value = 'Stream Settings must be an object or null';
      return;
    }
  } catch {
    jsonError.value = 'Stream Settings JSON parse error';
    return;
  }

  try {
    const v = JSON.parse(sniffingJson.value);
    if (v !== null && typeof v !== 'object') {
      jsonError.value = 'Sniffing must be an object or null';
      return;
    }
  } catch {
    jsonError.value = 'Sniffing JSON parse error';
    return;
  }

  jsonError.value = '';
};

watch([settingsJson, streamSettingsJson, sniffingJson, showAdvanced], validateJson);

watch(
  () => props.inbound,
  (inbound) => {
    if (inbound) {
      form.value = {
        tag: inbound.tag,
        protocol: inbound.protocol,
        port: inbound.port,
        listen: inbound.listen ?? '',
      };
      settingsJson.value = JSON.stringify(inbound.settings ?? {}, null, 2);
      streamSettingsJson.value = JSON.stringify(inbound.stream_settings ?? null, null, 2);
      sniffingJson.value = JSON.stringify(inbound.sniffing ?? null, null, 2);
    } else {
      form.value = {
        tag: '',
        protocol: 'vmess',
        port: 10086,
        listen: '',
      };
      settingsJson.value = '{}';
      streamSettingsJson.value = 'null';
      sniffingJson.value = 'null';
    }
    showAdvanced.value = false;
    jsonError.value = '';
  },
  { immediate: true }
);

const handleSubmit = () => {
  if (showAdvanced.value) {
    validateJson();
    if (jsonError.value) return;
  }

  const settings = showAdvanced.value ? JSON.parse(settingsJson.value) : {};
  const stream_settings = showAdvanced.value ? JSON.parse(streamSettingsJson.value) : null;
  const sniffing = showAdvanced.value ? JSON.parse(sniffingJson.value) : null;

  emit('submit', {
    tag: form.value.tag.trim(),
    protocol: form.value.protocol,
    port: Number(form.value.port),
    listen: form.value.listen.trim() ? form.value.listen.trim() : null,
    settings,
    stream_settings,
    sniffing,
  });
};
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.custom-scrollbar::-webkit-scrollbar {
  height: 4px;
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 4px;
}
</style>
