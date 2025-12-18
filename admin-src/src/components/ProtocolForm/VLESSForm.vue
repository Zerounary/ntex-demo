<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-cyan-50 to-blue-50 rounded-xl p-4 border border-cyan-100">
      <h4 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-cyan-500"></span>
        VLESS 配置
      </h4>
      
      <div class="space-y-3">
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            服务器地址 <span class="text-red-400">*</span>
          </label>
          <input
            v-model="config.address"
            type="text"
            required
            placeholder="example.com"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            端口 <span class="text-red-400">*</span>
          </label>
          <input
            v-model.number="config.port"
            type="number"
            required
            placeholder="443"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            用户 ID (UUID) <span class="text-red-400">*</span>
          </label>
          <input
            v-model="config.id"
            type="text"
            required
            placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            加密方式
          </label>
          <select
            v-model="config.encryption"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
          >
            <option value="none">none</option>
            <option value="zero">zero</option>
          </select>
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            Flow (XTLS)
          </label>
          <select
            v-model="config.flow"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
          >
            <option value="">无</option>
            <option value="xtls-rprx-vision">xtls-rprx-vision</option>
            <option value="xtls-rprx-vision-udp443">xtls-rprx-vision-udp443</option>
            <option value="xtls-rprx-splice">xtls-rprx-splice</option>
            <option value="xtls-rprx-splice-udp443">xtls-rprx-splice-udp443</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  modelValue: any;
}

interface Emits {
  (e: 'update:modelValue', value: any): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const config = ref({
  address: '',
  port: 443,
  id: '',
  encryption: 'none',
  flow: '',
});

watch(
  () => props.modelValue,
  (val) => {
    if (val && val.vnext && val.vnext[0]) {
      const vnext = val.vnext[0];
      const user = vnext.users?.[0] || {};
      config.value = {
        address: vnext.address || '',
        port: vnext.port || 443,
        id: user.id || '',
        encryption: user.encryption || 'none',
        flow: user.flow || '',
      };
    }
  },
  { immediate: true }
);

watch(
  config,
  (val) => {
    const user: any = {
      id: val.id,
      encryption: val.encryption,
    };
    if (val.flow) {
      user.flow = val.flow;
    }
    
    emit('update:modelValue', {
      vnext: [
        {
          address: val.address,
          port: Number(val.port),
          users: [user],
        },
      ],
    });
  },
  { deep: true }
);
</script>

<style scoped></style>

