<template>
  <form @submit.prevent="handleSubmit" class="space-y-5">
    <!-- 规则类型 -->
    <div class="bg-gradient-to-br from-indigo-50 to-purple-50 rounded-xl p-5 border border-indigo-100 shadow-sm">
      <label class="block text-sm font-semibold text-gray-700 mb-3">
        规则类型 <span class="text-red-400">*</span>
      </label>
      <div class="grid grid-cols-3 gap-3">
        <button
          v-for="type in ruleTypes"
          :key="type.value"
          type="button"
          @click="form.type = type.value"
          :class="[
            'px-4 py-3 rounded-xl text-sm font-medium transition-all active-scale',
            form.type === type.value
              ? 'bg-gradient-to-r from-indigo-500 to-purple-500 text-white shadow-lg'
              : 'bg-white text-gray-700 border border-gray-200 hover:border-indigo-300 hover:shadow-md'
          ]"
        >
          {{ type.label }}
        </button>
      </div>
    </div>

    <!-- Field 规则配置 -->
    <transition name="fade">
      <div v-if="form.type === 'field'" class="space-y-4 animate-fade-in">
        <div class="bg-white rounded-xl p-5 border border-gray-200 shadow-sm">
          <h4 class="text-sm font-semibold text-gray-700 mb-4 flex items-center gap-2">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-500"></span>
            匹配条件
          </h4>
          
          <div class="space-y-4">
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-2">
                出站标签
              </label>
              <input
                v-model="form.outbound_tag"
                type="text"
                placeholder="outbound_tag"
                class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
              />
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-2">
                域名匹配
              </label>
              <div class="space-y-2">
                <div
                  v-for="(domain, index) in domainList"
                  :key="index"
                  class="flex items-center gap-2"
                >
                  <input
                    v-model="domainList[index]"
                    type="text"
                    placeholder="example.com 或 *.example.com"
                    class="flex-1 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
                  />
                  <button
                    v-if="domainList.length > 1"
                    type="button"
                    @click="domainList.splice(index, 1)"
                    class="px-3 py-2 text-red-500 hover:bg-red-50 rounded-lg transition-all"
                  >
                    ✕
                  </button>
                </div>
                <button
                  type="button"
                  @click="domainList.push('')"
                  class="text-xs text-indigo-600 hover:text-indigo-700 flex items-center gap-1"
                >
                  <span>+</span>
                  <span>添加域名</span>
                </button>
              </div>
            </div>

            <div>
              <label class="block text-xs font-medium text-gray-600 mb-2">
                IP 匹配
              </label>
              <div class="space-y-2">
                <div
                  v-for="(ip, index) in ipList"
                  :key="index"
                  class="flex items-center gap-2"
                >
                  <input
                    v-model="ipList[index]"
                    type="text"
                    placeholder="192.168.1.1 或 10.0.0.0/8"
                    class="flex-1 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
                  />
                  <button
                    v-if="ipList.length > 1"
                    type="button"
                    @click="ipList.splice(index, 1)"
                    class="px-3 py-2 text-red-500 hover:bg-red-50 rounded-lg transition-all"
                  >
                    ✕
                  </button>
                </div>
                <button
                  type="button"
                  @click="ipList.push('')"
                  class="text-xs text-indigo-600 hover:text-indigo-700 flex items-center gap-1"
                >
                  <span>+</span>
                  <span>添加 IP</span>
                </button>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-medium text-gray-600 mb-2">
                  端口
                </label>
                <input
                  v-model="form.port"
                  type="text"
                  placeholder="80,443 或 1000-2000"
                  class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
                />
              </div>

              <div>
                <label class="block text-xs font-medium text-gray-600 mb-2">
                  网络类型
                </label>
                <select
                  v-model="form.network"
                  class="w-full px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
                >
                  <option value="">全部</option>
                  <option value="tcp">TCP</option>
                  <option value="udp">UDP</option>
                  <option value="tcp,udp">TCP+UDP</option>
                </select>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <!-- 规则预览 -->
    <div class="bg-gray-50 rounded-xl p-4 border border-gray-200">
      <h4 class="text-xs font-semibold text-gray-600 mb-2">规则预览</h4>
      <pre class="text-xs text-gray-700 font-mono bg-white p-3 rounded-lg overflow-x-auto">{{ JSON.stringify(previewRule, null, 2) }}</pre>
    </div>

    <!-- 操作按钮 -->
    <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
      <button
        type="button"
        @click="$emit('cancel')"
        class="px-6 py-2.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-xl hover:bg-gray-50 hover:shadow-md transition-all active-scale"
      >
        取消
      </button>
      <button
        type="submit"
        class="px-6 py-2.5 text-sm font-medium text-white bg-gradient-to-r from-indigo-500 to-purple-500 rounded-xl hover:shadow-lg transition-all active-scale"
      >
        {{ isEdit ? '更新规则' : '添加规则' }}
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
</style>
