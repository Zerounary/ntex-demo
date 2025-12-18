<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-gray-50 to-slate-50 rounded-xl p-4 border border-gray-200">
      <div class="flex items-center justify-between mb-3">
        <h4 class="text-sm font-semibold text-gray-700 flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-gray-400"></span>
          传输设置 (StreamSettings)
        </h4>
        <label class="flex items-center gap-2 text-xs text-gray-600 cursor-pointer">
          <input
            v-model="enabled"
            type="checkbox"
            class="w-4 h-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
          />
          <span>启用传输设置</span>
        </label>
      </div>
      
      <div v-if="enabled" class="space-y-3 animate-fade-in">
        <div>
          <label class="block text-xs font-medium text-gray-600 mb-1.5">
            网络类型
          </label>
          <select
            v-model="config.network"
            class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent transition-all"
          >
            <option value="tcp">TCP</option>
            <option value="kcp">mKCP</option>
            <option value="ws">WebSocket</option>
            <option value="http">HTTP/2</option>
            <option value="quic">QUIC</option>
            <option value="grpc">gRPC</option>
          </select>
        </div>
        
        <!-- TLS 设置 -->
        <div>
          <label class="flex items-center gap-2 text-xs text-gray-600 mb-1.5">
            <input
              v-model="config.security"
              type="checkbox"
              value="tls"
              class="w-4 h-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
            />
            <span>启用 TLS</span>
          </label>
        </div>
        
        <div v-if="config.security === 'tls'" class="ml-6 space-y-2 pl-4 border-l-2 border-gray-200">
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              服务器名称 (SNI)
            </label>
            <input
              v-model="config.tlsSettings.serverName"
              type="text"
              placeholder="example.com"
              class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent transition-all"
            />
          </div>
          
          <div>
            <label class="flex items-center gap-2 text-xs text-gray-600 mb-1.5">
              <input
                v-model="config.tlsSettings.allowInsecure"
                type="checkbox"
                class="w-4 h-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
              />
              <span>允许不安全连接</span>
            </label>
          </div>
        </div>
        
        <!-- WebSocket 设置 -->
        <div v-if="config.network === 'ws'" class="ml-6 space-y-2 pl-4 border-l-2 border-gray-200">
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              路径 (Path)
            </label>
            <input
              v-model="config.wsSettings.path"
              type="text"
              placeholder="/path"
              class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent transition-all"
            />
          </div>
          
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              Host 头
            </label>
            <input
              v-model="config.wsSettings.headers.Host"
              type="text"
              placeholder="example.com"
              class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent transition-all"
            />
          </div>
        </div>
        
        <!-- gRPC 设置 -->
        <div v-if="config.network === 'grpc'" class="ml-6 space-y-2 pl-4 border-l-2 border-gray-200">
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              服务名称
            </label>
            <input
              v-model="config.grpcSettings.serviceName"
              type="text"
              placeholder="GunService"
              class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent transition-all"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';

interface Props {
  modelValue?: any;
}

interface Emits {
  (e: 'update:modelValue', value: any): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const enabled = ref(false);

const config = ref({
  network: 'tcp',
  security: '',
  tlsSettings: {
    serverName: '',
    allowInsecure: false,
  },
  wsSettings: {
    path: '/',
    headers: {
      Host: '',
    },
  },
  grpcSettings: {
    serviceName: '',
  },
});

watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      enabled.value = true;
      config.value = {
        network: val.network || 'tcp',
        security: val.security || '',
        tlsSettings: {
          serverName: val.tlsSettings?.serverName || '',
          allowInsecure: val.tlsSettings?.allowInsecure || false,
        },
        wsSettings: {
          path: val.wsSettings?.path || '/',
          headers: {
            Host: val.wsSettings?.headers?.Host || '',
          },
        },
        grpcSettings: {
          serviceName: val.grpcSettings?.serviceName || '',
        },
      };
    } else {
      enabled.value = false;
    }
  },
  { immediate: true }
);

watch(
  [enabled, config],
  ([enabledVal, configVal]) => {
    if (enabledVal) {
      const result: any = {
        network: configVal.network,
      };
      
      if (configVal.security === 'tls') {
        result.security = 'tls';
        result.tlsSettings = {
          serverName: configVal.tlsSettings.serverName,
          allowInsecure: configVal.tlsSettings.allowInsecure,
        };
      }
      
      if (configVal.network === 'ws') {
        result.wsSettings = {
          path: configVal.wsSettings.path,
          headers: {
            Host: configVal.wsSettings.headers.Host,
          },
        };
      }
      
      if (configVal.network === 'grpc') {
        result.grpcSettings = {
          serviceName: configVal.grpcSettings.serviceName,
        };
      }
      
      emit('update:modelValue', result);
    } else {
      emit('update:modelValue', null);
    }
  },
  { deep: true }
);
</script>

<style scoped></style>

