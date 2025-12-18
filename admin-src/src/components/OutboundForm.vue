<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- Tag 和协议选择 -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          Tag <span class="text-red-400">*</span>
        </label>
        <input
          v-model="form.tag"
          type="text"
          required
          :disabled="isEdit"
          placeholder="输入代理标签"
          class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all shadow-sm hover:shadow-md"
        />
      </div>
      
      <div>
        <label class="block text-sm font-semibold text-gray-700 mb-2">
          协议类型 <span class="text-red-400">*</span>
        </label>
        <select
          v-model="form.protocol"
          class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all shadow-sm hover:shadow-md"
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
      </div>
    </div>

    <!-- 协议配置表单 -->
    <transition name="fade" mode="out-in">
      <component
        :is="protocolComponent"
        v-if="protocolComponent"
        :key="form.protocol"
        v-model="settings"
        class="animate-slide-up"
      />
    </transition>

    <!-- 传输设置 -->
    <StreamSettingsForm
      v-model="streamSettings"
    />

    <!-- JSON 编辑器（高级模式） -->
    <div class="bg-white rounded-xl p-4 border border-gray-200 shadow-sm">
      <div class="flex items-center justify-between mb-3">
        <label class="flex items-center gap-2 text-sm font-semibold text-gray-700">
          <input
            v-model="showAdvanced"
            type="checkbox"
            class="w-4 h-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
          />
          <span>高级模式 (JSON 编辑)</span>
        </label>
      </div>
      
      <transition name="fade">
        <div v-if="showAdvanced" class="animate-fade-in">
          <textarea
            v-model="settingsJson"
            rows="8"
            class="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
            placeholder='{"servers": [...]}'
          ></textarea>
          <p v-if="settingsError" class="mt-2 text-xs text-red-500 flex items-center gap-1">
            <span>⚠</span>
            {{ settingsError }}
          </p>
        </div>
      </transition>
    </div>

    <!-- 操作按钮 -->
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
        :disabled="!!settingsError || !isFormValid"
        class="px-6 py-2.5 text-sm font-medium text-white bg-gradient-to-r from-primary to-primary-dark rounded-xl hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed transition-all active-scale"
      >
        {{ isEdit ? '更新配置' : '添加代理' }}
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
      settingsError.value = '无效的 JSON 格式';
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
      // 忽略错误，使用当前 settings
    }
  }
});

const handleSubmit = () => {
  let finalSettings = settings.value;
  
  if (showAdvanced.value) {
    try {
      finalSettings = JSON.parse(settingsJson.value);
    } catch (e) {
      settingsError.value = 'JSON 解析失败';
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
</style>
