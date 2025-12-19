<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-cyan-50 to-blue-50 rounded-xl p-4 border border-cyan-100/50">
      <h4 class="text-sm font-bold text-gray-700 mb-4 flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-cyan-500 shadow-sm shadow-cyan-500/50"></div>
        VLESS Configuration
      </h4>
      
      <div class="space-y-4">
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            Server Address <span class="text-red-400">*</span>
          </label>
          <input
            v-model="config.address"
            type="text"
            required
            placeholder="example.com"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:border-cyan-400 transition-all shadow-sm"
          />
        </div>
        
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              Port <span class="text-red-400">*</span>
            </label>
            <input
              v-model.number="config.port"
              type="number"
              required
              placeholder="443"
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:border-cyan-400 transition-all shadow-sm"
            />
          </div>
          
           <div>
            <BaseSelect
              v-model="config.encryption"
              :options="encryptionOptions"
              label="Encryption"
            />
          </div>
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            UUID <span class="text-red-400">*</span>
          </label>
          <input
            v-model="config.id"
            type="text"
            required
            placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm font-mono focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:border-cyan-400 transition-all shadow-sm"
          />
        </div>
        
        <div>
          <BaseSelect
            v-model="config.flow"
            :options="flowOptions"
            label="Flow (XTLS)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import BaseSelect from '@/components/BaseSelect.vue';

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

const encryptionOptions = [
  { label: 'none', value: 'none' },
  { label: 'zero', value: 'zero' },
];

const flowOptions = [
  { label: 'None', value: '' },
  { label: 'xtls-rprx-vision', value: 'xtls-rprx-vision' },
  { label: 'xtls-rprx-vision-udp443', value: 'xtls-rprx-vision-udp443' },
  { label: 'xtls-rprx-splice', value: 'xtls-rprx-splice' },
  { label: 'xtls-rprx-splice-udp443', value: 'xtls-rprx-splice-udp443' },
];

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

