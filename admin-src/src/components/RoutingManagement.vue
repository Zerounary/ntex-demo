<template>
  <div class="routing-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">
          Routing Rules
        </h2>
        <p class="text-sm text-gray-500 mt-1">Configure traffic routing rules</p>
      </div>
      <button
        @click="showAddRuleForm = true"
        class="btn-primary flex items-center gap-2 shadow-lg hover:shadow-xl hover:-translate-y-0.5"
      >
        <div class="i-carbon-add text-lg"></div>
        <span>Add Rule</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.routingLoading" class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="loading-ring w-10 h-10 relative mx-auto mb-4"></div>
        <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.routingError"
      class="p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-xl flex items-center gap-3"
    >
      <div class="i-carbon-warning-alt text-lg"></div>
      <span class="font-medium">{{ adminStore.routingError }}</span>
    </div>

    <!-- 路由配置 -->
    <div v-else-if="!adminStore.routing" class="flex flex-col items-center justify-center py-20 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed">
      <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 text-gray-400">
        <div class="i-carbon-direction-fork text-3xl"></div>
      </div>
      <h3 class="text-lg font-bold text-gray-700 mb-1">No Routing Configured</h3>
      <p class="text-gray-400 text-sm">Initialize routing configuration to proceed</p>
    </div>

    <div v-else class="space-y-6">
      <!-- Domain Strategy -->
      <div class="card-base p-6">
        <BaseSelect
          v-model="domainStrategy"
          :options="domainStrategyOptions"
          label="Domain Strategy"
          @update:modelValue="updateDomainStrategy"
        >
          <template #icon>
            <div class="i-carbon-settings text-lg"></div>
          </template>
        </BaseSelect>
        <p class="mt-2 text-xs text-gray-400">Controls how domains are resolved in routing decisions</p>
      </div>

      <!-- 路由规则列表 -->
      <div>
        <h3 class="text-lg font-bold text-gray-900 mb-4 flex items-center gap-2">
          <div class="w-1 h-6 bg-primary-500 rounded-full"></div>
          Rules
          <span class="text-sm font-medium text-gray-400 bg-gray-100 px-2 py-0.5 rounded-full">{{ adminStore.routing.rules.length }}</span>
        </h3>
        
        <div v-if="adminStore.routing.rules.length === 0" class="text-center py-12 bg-gray-50/50 rounded-2xl border border-gray-100 border-dashed">
          <p class="text-gray-400">No routing rules defined</p>
        </div>
        
        <div v-else class="space-y-3">
          <transition-group name="list" tag="div">
            <div
              v-for="(rule, index) in adminStore.routing.rules"
              :key="index"
              class="card-base p-5 hover:shadow-md hover:-translate-x-[-4px] transition-all animate-slide-up group border-l-4 border-l-transparent hover:border-l-primary-500"
            >
              <div class="flex justify-between items-start mb-3">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-1">
                    <span class="text-xs font-mono text-gray-400">#{{ index + 1 }}</span>
                    <h4 class="font-bold text-gray-800">
                      {{ rule.type }}
                    </h4>
                    <span v-if="rule.outbound_tag" class="px-2 py-0.5 bg-indigo-50 text-indigo-600 rounded text-[10px] font-mono font-medium border border-indigo-100">
                      ➜ {{ rule.outbound_tag }}
                    </span>
                  </div>
                </div>
                <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    @click="editRule(index, rule)"
                    class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <div class="i-carbon-edit text-lg"></div>
                  </button>
                  <button
                    @click="deleteRule(index)"
                    class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <div class="i-carbon-trash-can text-lg"></div>
                  </button>
                </div>
              </div>
              
              <details class="cursor-pointer group/details">
                <summary class="text-xs font-medium text-gray-400 hover:text-primary-600 transition-colors flex items-center gap-1 select-none">
                  <div class="i-carbon-chevron-right group-open/details:rotate-90 transition-transform"></div>
                  View Details
                </summary>
                <div class="mt-2 p-3 bg-gray-50/80 rounded-lg border border-gray-100">
                  <pre class="text-[10px] text-gray-600 font-mono overflow-x-auto whitespace-pre-wrap">{{ JSON.stringify(rule, null, 2) }}</pre>
                </div>
              </details>
            </div>
          </transition-group>
        </div>
      </div>
    </div>

    <!-- 添加/编辑规则表单对话框 -->
    <transition name="modal">
      <div
        v-if="showAddRuleForm || editingRuleIndex !== null"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <h3 class="text-lg font-bold text-gray-900">
              {{ editingRuleIndex !== null ? 'Edit Rule' : 'Add Rule' }}
            </h3>
            <button @click="closeForm" class="text-gray-400 hover:text-gray-600 transition-colors">
              <div class="i-carbon-close text-xl"></div>
            </button>
          </div>
          <div class="p-6">
            <RoutingRuleForm
              :rule="editingRule"
              :is-edit="editingRuleIndex !== null"
              @submit="handleSubmit"
              @cancel="closeForm"
            />
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';
import { useToastStore } from '@/stores/toast';
import RoutingRuleForm from './RoutingRuleForm.vue';
import BaseSelect from '@/components/BaseSelect.vue';
import type { RoutingRule } from '@/api/types';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();
const toastStore = useToastStore();

const showAddRuleForm = ref(false);
const editingRuleIndex = ref<number | null>(null);
const domainStrategy = ref('AsIs');

const domainStrategyOptions = [
  { label: 'AsIs', value: 'AsIs' },
  { label: 'UseIP', value: 'UseIP' },
  { label: 'UseIPv4', value: 'UseIPv4' },
  { label: 'UseIPv6', value: 'UseIPv6' },
];

const editingRule = computed(() => {
  if (editingRuleIndex.value === null || !adminStore.routing) return null;
  return adminStore.routing.rules[editingRuleIndex.value];
});

onMounted(() => {
  if (nodeStore.currentNodeId) {
    adminStore.fetchRouting(nodeStore.currentNodeId);
  }
});

watch(
  () => adminStore.routing,
  (routing) => {
    if (routing) {
      domainStrategy.value = routing.domainStrategy;
    }
  },
  { immediate: true }
);

const updateDomainStrategy = async () => {
  if (!nodeStore.currentNodeId) return;
  try {
    await adminStore.updateRouting(nodeStore.currentNodeId, {
      domain_strategy: domainStrategy.value,
    });
    toastStore.success('Domain Strategy updated');
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to update Domain Strategy');
  }
};

const editRule = (index: number, rule: RoutingRule) => {
  editingRuleIndex.value = index;
  showAddRuleForm.value = false;
};

const deleteRule = async (index: number) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('Are you sure you want to delete this rule?')) return;

  try {
    await adminStore.deleteRoutingRule(nodeStore.currentNodeId, index);
    toastStore.success('Rule deleted successfully');
  } catch (err: any) {
    toastStore.error(err.message || 'Failed to delete rule');
  }
};

const handleSubmit = async (rule: RoutingRule) => {
  if (!nodeStore.currentNodeId) return;

  try {
    if (editingRuleIndex.value !== null) {
      await adminStore.updateRoutingRule(
        nodeStore.currentNodeId,
        editingRuleIndex.value,
        rule
      );
      toastStore.success('Rule updated successfully');
    } else {
      await adminStore.addRoutingRule(nodeStore.currentNodeId, rule);
      toastStore.success('Rule added successfully');
    }
    closeForm();
  } catch (err: any) {
    toastStore.error(err.message || 'Operation failed');
  }
};

const closeForm = () => {
  showAddRuleForm.value = false;
  editingRuleIndex.value = null;
};
</script>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.3s var(--ease-smooth);
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s var(--ease-smooth);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
