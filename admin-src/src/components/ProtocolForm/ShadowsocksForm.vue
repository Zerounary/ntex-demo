<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100/50">
      <h4 class="text-sm font-bold text-gray-700 mb-4 flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-blue-500 shadow-sm shadow-blue-500/50"></div>
        Shadowsocks Configuration
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
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-400/50 focus:border-blue-400 transition-all shadow-sm"
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
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-400/50 focus:border-blue-400 transition-all shadow-sm"
            />
          </div>
          
          <div>
            <BaseSelect
              v-model="config.method"
              :options="methodOptions"
              label="Encryption Method"
              required
            />
          </div>
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            Password <span class="text-red-400">*</span>
          </label>
          <div class="relative">
            <input
              v-model="config.password"
              :type="showPassword ? 'text' : 'password'"
              required
              placeholder="Enter password"
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-400/50 focus:border-blue-400 transition-all shadow-sm pr-10"
            />
            <button
              type="button"
              @click="showPassword = !showPassword"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-blue-600 transition-colors focus:outline-none"
            >
              <div :class="showPassword ? 'i-carbon-view-off' : 'i-carbon-view'" class="text-lg"></div>
            </button>
          </div>
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

const showPassword = ref(false);

const config = ref({
  address: '',
  port: 443,
  method: 'aes-256-gcm',
  password: '',
});

const methodOptions = [
  { label: 'AES-256-GCM', value: 'aes-256-gcm' },
  { label: 'AES-128-GCM', value: 'aes-128-gcm' },
  { label: 'ChaCha20-Poly1305', value: 'chacha20-poly1305' },
  { label: 'ChaCha20-IETF-Poly1305', value: 'chacha20-ietf-poly1305' },
  { label: 'XChaCha20-Poly1305', value: 'xchacha20-poly1305' },
  { label: '2022-blake3-aes-128-gcm', value: '2022-blake3-aes-128-gcm' },
  { label: '2022-blake3-aes-256-gcm', value: '2022-blake3-aes-256-gcm' },
  { label: '2022-blake3-chacha20-poly1305', value: '2022-blake3-chacha20-poly1305' },
];

watch(
  () => props.modelValue,
  (val) => {
    if (val && val.servers && val.servers[0]) {
      const server = val.servers[0];
      config.value = {
        address: server.address || '',
        port: server.port || 443,
        method: server.method || 'aes-256-gcm',
        password: server.password || '',
      };
    }
  },
  { immediate: true }
);

watch(
  config,
  (val) => {
    emit('update:modelValue', {
      servers: [
        {
          address: val.address,
          port: Number(val.port),
          method: val.method,
          password: val.password,
        },
      ],
    });
  },
  { deep: true }
);
</script>

<style scoped></style>

