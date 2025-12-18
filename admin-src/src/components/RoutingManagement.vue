<template>
  <div class="routing-management">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl font-bold text-gray-800">路由配置管理</h2>
      <button
        @click="showAddRuleForm = true"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
      >
        添加规则
      </button>
    </div>

    <div v-if="adminStore.routingLoading" class="text-center py-8 text-gray-500">
      加载中...
    </div>

    <div v-else-if="adminStore.routingError" class="text-center py-8 text-red-500">
      {{ adminStore.routingError }}
    </div>

    <div v-else-if="!adminStore.routing" class="text-center py-8 text-gray-500">
      暂无路由配置
    </div>

    <div v-else class="space-y-4">
      <!-- Domain Strategy -->
      <div class="bg-white border border-gray-200 rounded-lg p-4">
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Domain Strategy
        </label>
        <select
          v-model="domainStrategy"
          @change="updateDomainStrategy"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="AsIs">AsIs</option>
          <option value="UseIP">UseIP</option>
          <option value="UseIPv4">UseIPv4</option>
          <option value="UseIPv6">UseIPv6</option>
        </select>
      </div>

      <!-- Routing Rules -->
      <div>
        <h3 class="text-lg font-semibold text-gray-800 mb-3">路由规则</h3>
        <div v-if="adminStore.routing.rules.length === 0" class="text-center py-8 text-gray-500">
          暂无路由规则
        </div>
        <div v-else class="space-y-3">
          <div
            v-for="(rule, index) in adminStore.routing.rules"
            :key="index"
            class="bg-white border border-gray-200 rounded-lg p-4"
          >
            <div class="flex justify-between items-start mb-2">
              <div>
                <h4 class="font-semibold text-gray-800">
                  规则 #{{ index + 1 }} - {{ rule.type }}
                </h4>
                <p v-if="rule.outbound_tag" class="text-sm text-gray-600">
                  出站: {{ rule.outbound_tag }}
                </p>
              </div>
              <div class="flex space-x-2">
                <button
                  @click="editRule(index, rule)"
                  class="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
                >
                  编辑
                </button>
                <button
                  @click="deleteRule(index)"
                  class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
                >
                  删除
                </button>
              </div>
            </div>
            <details class="cursor-pointer mt-2">
              <summary class="text-sm text-gray-600">查看详情</summary>
              <pre
                class="mt-2 p-3 bg-gray-50 rounded text-xs overflow-x-auto"
              >{{ JSON.stringify(rule, null, 2) }}</pre>
            </details>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加/编辑规则表单对话框 -->
    <div
      v-if="showAddRuleForm || editingRuleIndex !== null"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="closeForm"
    >
      <div class="bg-white rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <h3 class="text-xl font-bold mb-4">
          {{ editingRuleIndex !== null ? '编辑规则' : '添加规则' }}
        </h3>
        <RoutingRuleForm
          :rule="editingRule"
          :is-edit="editingRuleIndex !== null"
          @submit="handleSubmit"
          @cancel="closeForm"
        />
      </div>
    </div>
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

<style scoped></style>

