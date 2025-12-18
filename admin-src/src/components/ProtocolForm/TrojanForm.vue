<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-orange-50 to-red-50 rounded-xl p-4 border border-orange-100">
      <h4 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-orange-500"></span>
        Trojan 配置
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-orange-400 focus:border-transparent transition-all"
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-orange-400 focus:border-transparent transition-all"
          />
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-orange-400 focus:border-transparent transition-all"
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
          password: val.password,
        },
      ],
    });
  },
  { deep: true }
);
</script>

<style scoped></style>

