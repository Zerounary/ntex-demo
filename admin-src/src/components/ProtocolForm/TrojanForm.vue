<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-orange-50 to-red-50 rounded-xl p-4 border border-orange-100/50">
      <h4 class="text-sm font-bold text-gray-700 mb-4 flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-orange-500 shadow-sm shadow-orange-500/50"></div>
        Trojan Configuration
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
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-orange-400/50 focus:border-orange-400 transition-all shadow-sm"
          />
        </div>
        
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            Port <span class="text-red-400">*</span>
          </label>
          <input
            v-model.number="config.port"
            type="number"
            required
            placeholder="443"
            class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-orange-400/50 focus:border-orange-400 transition-all shadow-sm"
          />
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
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-orange-400/50 focus:border-orange-400 transition-all shadow-sm pr-10"
            />
            <button
              type="button"
              @click="showPassword = !showPassword"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-orange-600 transition-colors focus:outline-none"
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

