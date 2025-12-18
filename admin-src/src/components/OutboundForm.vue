<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- Tag 和协议选择 -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          Tag <span class="text-red-400">*</span>
        </label>
        <div class="relative">
          <input
            v-model="form.tag"
            type="text"
            required
            :disabled="isEdit"
            placeholder="Enter proxy tag"
            class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm pl-10"
          />
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-tag text-lg"></div>
          </div>
        </div>
      </div>
      
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          Protocol <span class="text-red-400">*</span>
        </label>
        <div class="relative">
          <select
            v-model="form.protocol"
            class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all shadow-sm appearance-none pl-10"
          >
            <option value="shadowsocks">Shadowsocks</option>
            <option value="vmess">VMess</option>
            <option value="vless">VLESS</option>
            <option value="trojan">Trojan</option>
            <option value="socks">SOCKS</option>
            <option value="http">HTTP</option>
            <option value="freedom">Freedom</option>
            <option value="blackhole">Blackhole</option>
          </select>
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
            <div class="i-carbon-network-4 text-lg"></div>
          </div>
          <div class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
            <div class="i-carbon-chevron-down"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 协议配置表单 -->
    <div class="bg-gray-50/50 p-5 rounded-2xl border border-gray-100">
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
        <label class="flex items-center gap-2 text-sm font-semibold text-gray-700 cursor-pointer select-none">
          <input
            v-model="showAdvanced"
            type="checkbox"
            class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
          />
          <span>Advanced Mode (JSON)</span>
        </label>
      </div>
      
      <transition name="fade">
        <div v-if="showAdvanced" class="animate-fade-in">
          <textarea
            v-model="settingsJson"
            rows="8"
            class="w-full px-4 py-3 bg-gray-900 text-gray-300 border border-gray-700 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all custom-scrollbar"
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
const showAdvanced = ref(false);
const settingsJson = ref('{}');
const settingsError = ref('');

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
      settings.value = outbound.settings || {};
      streamSettings.value = outbound.stream_settings || null;
      settingsJson.value = JSON.stringify(outbound.settings, null, 2);
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
    showAdvanced.value = false;
  },
  { immediate: true }
);

watch(settingsJson, (value) => {
  if (showAdvanced.value) {
    try {
      JSON.parse(value);
      settingsError.value = '';
    } catch (e) {
      settingsError.value = 'Invalid JSON format';
    }
  }
});

watch(showAdvanced, (val) => {
  if (val) {
    settingsJson.value = JSON.stringify(settings.value, null, 2);
  } else {
    try {
      settings.value = JSON.parse(settingsJson.value);
      settingsError.value = '';
    } catch (e) {
      // Ignore error, use current settings
    }
  }
});

const handleSubmit = () => {
  let finalSettings = settings.value;
  
  if (showAdvanced.value) {
    try {
      finalSettings = JSON.parse(settingsJson.value);
    } catch (e) {
      settingsError.value = 'JSON parse error';
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
