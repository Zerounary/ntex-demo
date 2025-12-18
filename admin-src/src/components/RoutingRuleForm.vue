<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- 规则类型 -->
    <div class="bg-gray-50/50 rounded-2xl p-5 border border-gray-100">
      <label class="block text-sm font-bold text-gray-700 mb-3">
        Rule Type <span class="text-red-400">*</span>
      </label>
      <div class="grid grid-cols-3 gap-3">
        <button
          v-for="type in ruleTypes"
          :key="type.value"
          type="button"
          @click="form.type = type.value"
          :class="[
            'px-4 py-3 rounded-xl text-sm font-medium transition-all active:scale-95',
            form.type === type.value
              ? 'bg-primary-600 text-white shadow-lg shadow-primary-500/20'
              : 'bg-white text-gray-600 border border-gray-200 hover:border-primary-300 hover:text-primary-600'
          ]"
        >
          {{ type.label }}
        </button>
      </div>
    </div>

    <!-- Field 规则配置 -->
    <transition name="fade">
      <div v-if="form.type === 'field'" class="space-y-5 animate-fade-in">
        <div class="card-base p-5">
          <h4 class="text-sm font-bold text-gray-800 mb-4 flex items-center gap-2 pb-3 border-b border-gray-100">
            <div class="w-1.5 h-1.5 rounded-full bg-primary-500"></div>
            Matching Conditions
          </h4>
          
          <div class="space-y-5">
            <div>
              <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                Outbound Tag
              </label>
              <div class="relative">
                <input
                  v-model="form.outbound_tag"
                  type="text"
                  placeholder="e.g., proxy-us"
                  class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10"
                />
                <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                  <div class="i-carbon-tag text-lg"></div>
                </div>
              </div>
            </div>

            <div>
              <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                Domains
              </label>
              <div class="space-y-2">
                <div
                  v-for="(domain, index) in domainList"
                  :key="index"
                  class="flex items-center gap-2"
                >
                  <div class="relative flex-1">
                    <input
                      v-model="domainList[index]"
                      type="text"
                      placeholder="example.com or *.example.com"
                      class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10"
                    />
                    <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                      <div class="i-carbon-wikis text-lg"></div>
                    </div>
                  </div>
                  <button
                    v-if="domainList.length > 1"
                    type="button"
                    @click="domainList.splice(index, 1)"
                    class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  >
                    <div class="i-carbon-close text-lg"></div>
                  </button>
                </div>
                <button
                  type="button"
                  @click="domainList.push('')"
                  class="text-xs font-medium text-primary-600 hover:text-primary-700 flex items-center gap-1 px-2 py-1 rounded hover:bg-primary-50 transition-colors w-fit"
                >
                  <div class="i-carbon-add"></div>
                  <span>Add Domain</span>
                </button>
              </div>
            </div>

            <div>
              <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                IPs / CIDRs
              </label>
              <div class="space-y-2">
                <div
                  v-for="(ip, index) in ipList"
                  :key="index"
                  class="flex items-center gap-2"
                >
                  <div class="relative flex-1">
                    <input
                      v-model="ipList[index]"
                      type="text"
                      placeholder="192.168.1.1 or 10.0.0.0/8"
                      class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10 font-mono"
                    />
                    <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                      <div class="i-carbon-network-1 text-lg"></div>
                    </div>
                  </div>
                  <button
                    v-if="ipList.length > 1"
                    type="button"
                    @click="ipList.splice(index, 1)"
                    class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  >
                    <div class="i-carbon-close text-lg"></div>
                  </button>
                </div>
                <button
                  type="button"
                  @click="ipList.push('')"
                  class="text-xs font-medium text-primary-600 hover:text-primary-700 flex items-center gap-1 px-2 py-1 rounded hover:bg-primary-50 transition-colors w-fit"
                >
                  <div class="i-carbon-add"></div>
                  <span>Add IP</span>
                </button>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-5">
              <div>
                <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                  Port
                </label>
                <div class="relative">
                  <input
                    v-model="form.port"
                    type="text"
                    placeholder="80,443 or 1000-2000"
                    class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all pl-10"
                  />
                  <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                    <div class="i-carbon-port-input text-lg"></div>
                  </div>
                </div>
              </div>

              <div>
                <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                  Network
                </label>
                <div class="relative">
                  <select
                    v-model="form.network"
                    class="w-full px-4 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/50 focus:border-primary-400 transition-all appearance-none pl-10"
                  >
                    <option value="">All Networks</option>
                    <option value="tcp">TCP</option>
                    <option value="udp">UDP</option>
                    <option value="tcp,udp">TCP + UDP</option>
                  </select>
                  <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
                    <div class="i-carbon-network-overlay text-lg"></div>
                  </div>
                  <div class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
                    <div class="i-carbon-chevron-down"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <!-- 规则预览 -->
    <div class="bg-gray-900 rounded-xl p-4 border border-gray-800 shadow-inner">
      <h4 class="text-xs font-bold text-gray-400 mb-2 uppercase tracking-wider flex items-center gap-2">
        <div class="i-carbon-code"></div>
        Preview
      </h4>
      <pre class="text-xs text-gray-300 font-mono overflow-x-auto custom-scrollbar">{{ JSON.stringify(previewRule, null, 2) }}</pre>
    </div>

    <!-- 操作按钮 -->
    <div class="flex justify-end gap-3 pt-4 border-t border-gray-100">
      <button
        type="button"
        @click="$emit('cancel')"
        class="btn-ghost"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="btn-primary shadow-lg shadow-primary-500/20"
      >
        {{ isEdit ? 'Update Rule' : 'Add Rule' }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import type { RoutingRule } from '@/api/types';

interface Props {
  rule?: RoutingRule | null;
  isEdit?: boolean;
}

interface Emits {
  (e: 'submit', rule: RoutingRule): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  rule: null,
  isEdit: false,
});

const emit = defineEmits<Emits>();

const ruleTypes = [
  { value: 'field', label: 'Field' },
  { value: 'chinaip', label: 'ChinaIP' },
  { value: 'chinasites', label: 'ChinaSites' },
];

const form = ref<RoutingRule>({
  type: 'field',
  outbound_tag: undefined,
  domain: undefined,
  ip: undefined,
  port: undefined,
  network: undefined,
  source: undefined,
  protocol: undefined,
});

const domainList = ref<string[]>(['']);
const ipList = ref<string[]>(['']);

const previewRule = computed(() => {
  const rule: RoutingRule = {
    type: form.value.type,
  };
  
  if (form.value.type === 'field') {
    if (form.value.outbound_tag) rule.outbound_tag = form.value.outbound_tag;
    if (domainList.value.filter(d => d.trim()).length > 0) {
      rule.domain = domainList.value.filter(d => d.trim());
    }
    if (ipList.value.filter(i => i.trim()).length > 0) {
      rule.ip = ipList.value.filter(i => i.trim());
    }
    if (form.value.port) rule.port = form.value.port;
    if (form.value.network) rule.network = form.value.network;
  } else {
    rule.outbound_tag = form.value.outbound_tag;
  }
  
  return rule;
});

watch(
  () => props.rule,
  (rule) => {
    if (rule) {
      form.value = { ...rule };
      domainList.value = rule.domain && rule.domain.length > 0 ? [...rule.domain, ''] : [''];
      ipList.value = rule.ip && rule.ip.length > 0 ? [...rule.ip, ''] : [''];
    } else {
      form.value = {
        type: 'field',
        outbound_tag: undefined,
        domain: undefined,
        ip: undefined,
        port: undefined,
        network: undefined,
        source: undefined,
        protocol: undefined,
      };
      domainList.value = [''];
      ipList.value = [''];
    }
  },
  { immediate: true }
);

const handleSubmit = () => {
  const rule: RoutingRule = {
    type: form.value.type,
  };
  
  if (form.value.type === 'field') {
    if (form.value.outbound_tag) rule.outbound_tag = form.value.outbound_tag;
    
    const domains = domainList.value.filter(d => d.trim());
    if (domains.length > 0) rule.domain = domains;
    
    const ips = ipList.value.filter(i => i.trim());
    if (ips.length > 0) rule.ip = ips;
    
    if (form.value.port) rule.port = form.value.port;
    if (form.value.network) rule.network = form.value.network;
  } else {
    rule.outbound_tag = form.value.outbound_tag;
  }
  
  emit('submit', rule);
};
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.custom-scrollbar::-webkit-scrollbar {
  height: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 4px;
}
</style>
