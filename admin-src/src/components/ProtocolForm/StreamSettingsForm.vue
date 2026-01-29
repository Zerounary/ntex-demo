<template>
  <div class="space-y-4">
    <div class="bg-gradient-to-br from-gray-50 to-slate-50 rounded-xl p-5 border border-gray-200/60 shadow-sm">
      <div class="flex items-center justify-between mb-4">
        <h4 class="text-sm font-bold text-gray-700 flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-gray-500 shadow-sm shadow-gray-500/50"></div>
          Stream Settings
        </h4>
        <label class="flex items-center gap-2 text-xs text-gray-600 cursor-pointer select-none group">
          <input
            v-model="enabled"
            type="checkbox"
            class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
          />
          <span class="group-hover:text-primary-600 transition-colors">Enable Stream Settings</span>
        </label>
      </div>
      
      <div v-if="enabled" class="space-y-4 animate-fade-in">
        <div>
          <BaseSelect
            v-model="config.network"
            :options="networkOptions"
            label="Network"
          />
        </div>
        
        <!-- TLS 设置 -->
        <div class="bg-white/50 rounded-lg p-3 border border-gray-200/50">
          <label class="flex items-center gap-2 text-xs font-semibold text-gray-700 mb-2 cursor-pointer select-none group w-fit">
            <input
              v-model="config.security"
              type="checkbox"
              value="tls"
              :true-value="'tls'"
              :false-value="''"
              class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
            />
            <span class="group-hover:text-primary-600 transition-colors">Enable TLS</span>
          </label>
        
          <div v-if="config.security === 'tls'" class="space-y-3 pl-6 animate-fade-in">
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">
                Server Name (SNI)
              </label>
              <input
                v-model="config.tlsSettings.serverName"
                type="text"
                placeholder="example.com"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>
            
            <div>
              <label class="flex items-center gap-2 text-xs text-gray-600 cursor-pointer select-none group w-fit">
                <input
                  v-model="config.tlsSettings.allowInsecure"
                  type="checkbox"
                  class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
                />
                <span class="group-hover:text-primary-600 transition-colors">Allow Insecure</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Reality 设置 -->
        <div class="bg-white/50 rounded-lg p-3 border border-gray-200/50">
          <label class="flex items-center gap-2 text-xs font-semibold text-gray-700 mb-2 cursor-pointer select-none group w-fit">
            <input
              v-model="config.security"
              type="checkbox"
              value="reality"
              :true-value="'reality'"
              :false-value="''"
              class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
            />
            <span class="group-hover:text-primary-600 transition-colors">Enable Reality</span>
          </label>

          <div v-if="config.security === 'reality'" class="space-y-3 pl-6 animate-fade-in">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Server Name (SNI)</label>
                <input
                  v-model="config.realitySettings.serverName"
                  type="text"
                  placeholder="www.cloudflare.com"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>

              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Public Key (base64url)</label>
                <input
                  v-model="config.realitySettings.publicKey"
                  type="text"
                  placeholder="..."
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>

              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Short ID (hex)</label>
                <input
                  v-model="config.realitySettings.shortId"
                  type="text"
                  placeholder="a66cafe7"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>

              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Fingerprint</label>
                <input
                  v-model="config.realitySettings.fingerprint"
                  type="text"
                  placeholder="chrome"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">SpiderX</label>
              <input
                v-model="config.realitySettings.spiderX"
                type="text"
                placeholder="/"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Dest (server)</label>
                <input
                  v-model="config.realitySettings.dest"
                  type="text"
                  placeholder="www.cloudflare.com:443"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>
              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Xver (server)</label>
                <input
                  v-model.number="config.realitySettings.xver"
                  type="number"
                  min="0"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
                />
              </div>
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">Server Names (server, comma separated)</label>
              <input
                v-model="config.realitySettings.serverNamesText"
                type="text"
                placeholder="www.cloudflare.com"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">Private Key (server)</label>
              <input
                v-model="config.realitySettings.privateKey"
                type="text"
                placeholder="..."
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">Short IDs (server, comma separated)</label>
              <input
                v-model="config.realitySettings.shortIdsText"
                type="text"
                placeholder="a66cafe7"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>

            <div>
              <label class="flex items-center gap-2 text-xs text-gray-600 cursor-pointer select-none group w-fit">
                <input
                  v-model="config.realitySettings.show"
                  type="checkbox"
                  class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-2 focus:ring-primary-400/50 transition-colors"
                />
                <span class="group-hover:text-primary-600 transition-colors">Show (server)</span>
              </label>
            </div>
          </div>
        </div>
        
        <!-- WebSocket 设置 -->
        <div v-if="config.network === 'ws'" class="bg-white/50 rounded-lg p-3 border border-gray-200/50 animate-fade-in">
           <h5 class="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">WebSocket Settings</h5>
           <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">
                Path
              </label>
              <input
                v-model="config.wsSettings.path"
                type="text"
                placeholder="/path"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>
            
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1.5">
                Host Header
              </label>
              <input
                v-model="config.wsSettings.headers.Host"
                type="text"
                placeholder="example.com"
                class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
              />
            </div>
          </div>
        </div>
        
        <!-- gRPC 设置 -->
        <div v-if="config.network === 'grpc'" class="bg-white/50 rounded-lg p-3 border border-gray-200/50 animate-fade-in">
          <h5 class="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">gRPC Settings</h5>
          <div>
            <label class="block text-xs font-medium text-gray-600 mb-1.5">
              Service Name
            </label>
            <input
              v-model="config.grpcSettings.serviceName"
              type="text"
              placeholder="GunService"
              class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-400/30 focus:border-gray-400 transition-all shadow-sm"
            />
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
  realitySettings: {
    show: false,
    dest: '',
    xver: 0,
    serverNamesText: '',
    privateKey: '',
    shortIdsText: '',
    serverName: '',
    publicKey: '',
    shortId: '',
    fingerprint: 'chrome',
    spiderX: '/',
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

const networkOptions = [
  { label: 'TCP', value: 'tcp' },
  { label: 'mKCP', value: 'kcp' },
  { label: 'WebSocket', value: 'ws' },
  { label: 'HTTP/2', value: 'http' },
  { label: 'QUIC', value: 'quic' },
  { label: 'gRPC', value: 'grpc' },
];

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
        realitySettings: {
          show: val.realitySettings?.show || false,
          dest: val.realitySettings?.dest || '',
          xver: typeof val.realitySettings?.xver === 'number' ? val.realitySettings.xver : 0,
          serverNamesText: Array.isArray(val.realitySettings?.serverNames)
            ? val.realitySettings.serverNames.join(', ')
            : (val.realitySettings?.serverNamesText || ''),
          privateKey: val.realitySettings?.privateKey || '',
          shortIdsText: Array.isArray(val.realitySettings?.shortIds)
            ? val.realitySettings.shortIds.join(', ')
            : (val.realitySettings?.shortIdsText || ''),
          serverName: val.realitySettings?.serverName || '',
          publicKey: val.realitySettings?.publicKey || '',
          shortId: val.realitySettings?.shortId || '',
          fingerprint: val.realitySettings?.fingerprint || 'chrome',
          spiderX: val.realitySettings?.spiderX || '/',
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

      if (configVal.security === 'reality') {
        const parseList = (value: string) =>
          value
            .split(',')
            .map((s) => s.trim())
            .filter((s) => s.length > 0);

        result.security = 'reality';
        result.realitySettings = {
          show: configVal.realitySettings.show,
          dest: configVal.realitySettings.dest,
          xver: Number(configVal.realitySettings.xver) || 0,
          serverNames: parseList(configVal.realitySettings.serverNamesText),
          privateKey: configVal.realitySettings.privateKey,
          shortIds: parseList(configVal.realitySettings.shortIdsText),
          serverName: configVal.realitySettings.serverName,
          publicKey: configVal.realitySettings.publicKey,
          shortId: configVal.realitySettings.shortId,
          fingerprint: configVal.realitySettings.fingerprint,
          spiderX: configVal.realitySettings.spiderX,
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

