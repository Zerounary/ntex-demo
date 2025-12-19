<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-purple-50 to-pink-50 rounded-xl p-4 border border-purple-100/50">
      <h4 class="text-sm font-bold text-gray-700 mb-4 flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-purple-500 shadow-sm shadow-purple-500/50"></div>
        VMess Configuration
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
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-purple-400/50 focus:border-purple-400 transition-all shadow-sm"
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
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-purple-400/50 focus:border-purple-400 transition-all shadow-sm"
            />
          </div>
          
          <div>
             <BaseSelect
              v-model="config.security"
              :options="securityOptions"
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
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm font-mono focus:outline-none focus:ring-2 focus:ring-purple-400/50 focus:border-purple-400 transition-all shadow-sm"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            AlterId
          </label>
          <input
            v-model.number="config.alterId"
            type="number"
            min="0"
            placeholder="0 (Recommended)"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-purple-400/50 focus:border-purple-400 transition-all shadow-sm"
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
  alterId: 0,
  security: 'auto',
});

const securityOptions = [
  { label: 'auto', value: 'auto' },
  { label: 'aes-128-gcm', value: 'aes-128-gcm' },
  { label: 'chacha20-poly1305', value: 'chacha20-poly1305' },
  { label: 'none', value: 'none' },
  { label: 'zero', value: 'zero' },
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
        alterId: user.alterId || 0,
        security: user.security || 'auto',
      };
    }
  },
  { immediate: true }
);

watch(
  config,
  (val) => {
    emit('update:modelValue', {
      vnext: [
        {
          address: val.address,
          port: Number(val.port),
          users: [
            {
              id: val.id,
              alterId: Number(val.alterId) || 0,
              security: val.security,
            },
          ],
        },
      ],
    });
  },
  { deep: true }
);
</script>

<style scoped></style>

