<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100">
      <h4 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-blue-500"></span>
        Shadowsocks 配置
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all"
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            加密方法 <span class="text-red-400">*</span>
          </label>
          <select
            v-model="config.method"
            required
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all"
          >
            <option value="aes-256-gcm">AES-256-GCM</option>
            <option value="aes-128-gcm">AES-128-GCM</option>
            <option value="chacha20-poly1305">ChaCha20-Poly1305</option>
            <option value="chacha20-ietf-poly1305">ChaCha20-IETF-Poly1305</option>
            <option value="xchacha20-poly1305">XChaCha20-Poly1305</option>
            <option value="2022-blake3-aes-128-gcm">2022-blake3-aes-128-gcm</option>
            <option value="2022-blake3-aes-256-gcm">2022-blake3-aes-256-gcm</option>
            <option value="2022-blake3-chacha20-poly1305">2022-blake3-chacha20-poly1305</option>
          </select>
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            密码 <span class="text-red-400">*</span>
          </label>
          <input
            v-model="config.password"
            type="password"
            required
            placeholder="输入密码"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all"
          />
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
  method: 'aes-256-gcm',
  password: '',
});

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

