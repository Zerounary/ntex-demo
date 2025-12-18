<template>
  <div class="routing-management space-y-6">
    <!-- 头部 -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
          路由配置管理
        </h2>
        <p class="text-sm text-gray-500 mt-1">配置流量路由规则</p>
      </div>
      <button
        @click="showAddRuleForm = true"
        class="px-6 py-2.5 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-medium shadow-lg hover:shadow-xl transition-all active-scale flex items-center gap-2"
      >
        <span>+</span>
        <span>添加规则</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="adminStore.routingLoading" class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-500 rounded-full animate-spin mx-auto mb-3"></div>
        <p class="text-gray-500 text-sm">加载中...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="adminStore.routingError"
      class="p-4 bg-red-50 border border-red-200 text-red-700 rounded-xl"
    >
      {{ adminStore.routingError }}
    </div>

    <!-- 路由配置 -->
    <div v-else-if="!adminStore.routing" class="text-center py-16 bg-gradient-to-br from-gray-50 to-gray-100 rounded-2xl border border-gray-200">
      <div class="w-20 h-20 bg-gradient-to-br from-indigo-100 to-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <span class="text-3xl">🛣️</span>
      </div>
      <p class="text-gray-500">暂无路由配置</p>
    </div>

    <div v-else class="space-y-6">
      <!-- Domain Strategy -->
      <div class="bg-white rounded-2xl p-6 border border-gray-100 shadow-sm">
        <label class="block text-sm font-semibold text-gray-700 mb-3">
          Domain Strategy
        </label>
        <select
          v-model="domainStrategy"
          @change="updateDomainStrategy"
          class="w-full px-4 py-2.5 bg-gray-50 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:border-transparent transition-all"
        >
          <option value="AsIs">AsIs</option>
          <option value="UseIP">UseIP</option>
          <option value="UseIPv4">UseIPv4</option>
          <option value="UseIPv6">UseIPv6</option>
        </select>
        <p class="mt-2 text-xs text-gray-500">域名解析策略</p>
      </div>

      <!-- 路由规则列表 -->
      <div>
        <h3 class="text-lg font-semibold text-gray-800 mb-4">路由规则</h3>
        <div v-if="adminStore.routing.rules.length === 0" class="text-center py-12 bg-gradient-to-br from-gray-50 to-gray-100 rounded-2xl border border-gray-200">
          <p class="text-gray-500">暂无路由规则</p>
        </div>
        <div v-else class="space-y-3">
          <transition-group name="list" tag="div">
            <div
              v-for="(rule, index) in adminStore.routing.rules"
              :key="index"
              class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm hover:shadow-md transition-all animate-slide-up"
            >
              <div class="flex justify-between items-start mb-3">
                <div class="flex-1">
                  <h4 class="font-semibold text-gray-800 mb-1">
                    规则 #{{ index + 1 }} - {{ rule.type }}
                  </h4>
                  <p v-if="rule.outbound_tag" class="text-sm text-gray-600">
                    出站: <span class="font-mono text-indigo-600">{{ rule.outbound_tag }}</span>
                  </p>
                </div>
                <div class="flex gap-2">
                  <button
                    @click="editRule(index, rule)"
                    class="px-3 py-1.5 text-xs font-medium bg-indigo-50 text-indigo-700 rounded-lg hover:bg-indigo-100 transition-all"
                  >
                    编辑
                  </button>
                  <button
                    @click="deleteRule(index)"
                    class="px-3 py-1.5 text-xs font-medium bg-red-50 text-red-700 rounded-lg hover:bg-red-100 transition-all"
                  >
                    删除
                  </button>
                </div>
              </div>
              <details class="cursor-pointer group">
                <summary class="text-xs text-gray-500 group-hover:text-gray-700 transition-colors">
                  查看详情
                </summary>
                <pre
                  class="mt-2 p-3 bg-gray-50 rounded-lg text-xs overflow-x-auto border border-gray-200"
                >{{ JSON.stringify(rule, null, 2) }}</pre>
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
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="closeForm"
      >
        <div class="bg-white rounded-2xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto animate-scale-in">
          <div class="p-6 border-b border-gray-200">
            <h3 class="text-xl font-bold text-gray-800">
              {{ editingRuleIndex !== null ? '编辑规则' : '添加规则' }}
            </h3>
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
import RoutingRuleForm from './RoutingRuleForm.vue';
import type { RoutingRule } from '@/api/types';

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const showAddRuleForm = ref(false);
const editingRuleIndex = ref<number | null>(null);
const domainStrategy = ref('AsIs');

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
  } catch (err: any) {
    alert(err.message || '更新 Domain Strategy 失败');
  }
};

const editRule = (index: number, rule: RoutingRule) => {
  editingRuleIndex.value = index;
  showAddRuleForm.value = false;
};

const deleteRule = async (index: number) => {
  if (!nodeStore.currentNodeId) return;
  if (!confirm('确定要删除这个规则吗？')) return;

  try {
    await adminStore.deleteRoutingRule(nodeStore.currentNodeId, index);
  } catch (err: any) {
    alert(err.message || '删除规则失败');
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
    } else {
      await adminStore.addRoutingRule(nodeStore.currentNodeId, rule);
    }
    closeForm();
  } catch (err: any) {
    alert(err.message || '操作失败');
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
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
