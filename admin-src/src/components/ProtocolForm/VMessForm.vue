<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-purple-50 to-pink-50 rounded-xl p-4 border border-purple-100">
      <h4 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-purple-500"></span>
        VMess 配置
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent transition-all"
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent transition-all"
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
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent transition-all"
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
            placeholder="0 (推荐)"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent transition-all"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            加密方式
          </label>
          <select
            v-model="config.security"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent transition-all"
          >
            <option value="auto">auto</option>
            <option value="aes-128-gcm">aes-128-gcm</option>
            <option value="chacha20-poly1305">chacha20-poly1305</option>
            <option value="none">none</option>
            <option value="zero">zero</option>
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
  alterId: 0,
  security: 'auto',
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

