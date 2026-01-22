<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- Tag 和协议选择 -->
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
            placeholder="Enter proxy tag"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm pl-10"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-tag text-lg"></div>
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
    </div>

    <!-- 协议配置表单 -->
    <div class="bg-gray-50/50 p-5 rounded-2xl border border-gray-100 transition-all duration-300">
      <transition name="fade" mode="out-in">
        <component
          :is="protocolComponent"
          v-if="protocolComponent"
          :key="form.protocol"
          v-model="settings"
          class="animate-slide-up"
        />
      </transition>
    </div>

    <!-- 传输设置 -->
    <StreamSettingsForm
      v-model="streamSettings"
    />

    <!-- JSON 编辑器（高级模式） -->
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
        <div v-if="showAdvanced" class="animate-fade-in">
          <textarea
            v-model="settingsJson"
            rows="8"
            class="w-full px-4 py-3 bg-gray-900 text-gray-300 border border-gray-700 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all custom-scrollbar resize-y"
            placeholder='{"servers": [...]}'
          ></textarea>
          <p v-if="settingsError" class="mt-2 text-xs text-red-500 flex items-center gap-1">
            <div class="i-carbon-warning-filled"></div>
            {{ settingsError }}
          </p>
        </div>
      </transition>
    </div>

    <!-- 操作按钮 -->
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
        :disabled="!!settingsError || !isFormValid"
        class="btn-primary shadow-lg shadow-primary-500/20 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none"
      >
        {{ isEdit ? 'Update Proxy' : 'Add Proxy' }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch, markRaw } from 'vue';
import type { OutboundConfig } from '@/api/types';
import BaseSelect from '@/components/BaseSelect.vue';
import ShadowsocksForm from './ProtocolForm/ShadowsocksForm.vue';
import VMessForm from './ProtocolForm/VMessForm.vue';
import VLESSForm from './ProtocolForm/VLESSForm.vue';
import TrojanForm from './ProtocolForm/TrojanForm.vue';
import StreamSettingsForm from './ProtocolForm/StreamSettingsForm.vue';

interface Props {
  outbound?: OutboundConfig | null;
  isEdit?: boolean;
}

interface Emits {
  (e: 'submit', outbound: { tag: string; protocol: string; settings: any; stream_settings?: any }): void;
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

const settings = ref<any>({});
const streamSettings = ref<any>(null);
const isAdvanced = ref(false);
const showAdvanced = computed({
  get: () => isAdvanced.value,
  set: (val: boolean) => toggleAdvancedMode(val),
});
const settingsJson = ref('{}');
const settingsError = ref('');

const cloneData = <T>(data: T): T => {
  if (data === null || data === undefined) {
    return data;
  }
  return JSON.parse(JSON.stringify(data)) as T;
};

const toggleAdvancedMode = (enabled: boolean) => {
  if (enabled) {
    settingsJson.value = JSON.stringify(settings.value, null, 2);
    settingsError.value = '';
    isAdvanced.value = true;
    return;
  }

  try {
    settings.value = JSON.parse(settingsJson.value);
    settingsError.value = '';
    isAdvanced.value = false;
  } catch (err) {
    settingsError.value = 'Invalid JSON format';
    isAdvanced.value = true;
    console.error('Failed to parse settings JSON', err);
  }
};

const protocolOptions = [
  { label: 'Shadowsocks', value: 'shadowsocks' },
  { label: 'VMess', value: 'vmess' },
  { label: 'VLESS', value: 'vless' },
  { label: 'Trojan', value: 'trojan' },
  { label: 'SOCKS', value: 'socks' },
  { label: 'HTTP', value: 'http' },
  { label: 'Freedom', value: 'freedom' },
  { label: 'Blackhole', value: 'blackhole' },
];

const protocolComponents: Record<string, any> = {
  shadowsocks: markRaw(ShadowsocksForm),
  vmess: markRaw(VMessForm),
  vless: markRaw(VLESSForm),
  trojan: markRaw(TrojanForm),
};

const protocolComponent = computed(() => {
  return protocolComponents[form.value.protocol] || null;
});

const isFormValid = computed(() => {
  return form.value.tag.trim() !== '' && !settingsError.value;
});

watch(
  () => props.outbound,
  (outbound) => {
    if (outbound) {
      form.value = {
        tag: outbound.tag,
        protocol: outbound.protocol,
      };
      const clonedSettings = outbound.settings ? cloneData(outbound.settings) : {};
      const clonedStreamSettings = outbound.stream_settings ? cloneData(outbound.stream_settings) : null;
      settings.value = clonedSettings;
      streamSettings.value = clonedStreamSettings;
      settingsJson.value = JSON.stringify(clonedSettings, null, 2);
    } else {
      form.value = {
        tag: '',
        protocol: 'shadowsocks',
      };
      settings.value = {};
      streamSettings.value = null;
      settingsJson.value = '{}';
    }
    settingsError.value = '';
    isAdvanced.value = false;
  },
  { immediate: true }
);

const handleSubmit = () => {
  let finalSettings = settings.value;

  if (showAdvanced.value) {
    try {
      finalSettings = JSON.parse(settingsJson.value);
      settingsError.value = '';
    } catch (e) {
      settingsError.value = 'Invalid JSON format';
      return;
    }
  }

  emit('submit', {
    tag: form.value.tag,
    protocol: form.value.protocol,
    settings: finalSettings,
    stream_settings: streamSettings.value,
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
