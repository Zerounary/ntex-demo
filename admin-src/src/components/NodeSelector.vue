<template>
  <div class="node-selector flex gap-2 items-center">
    <BaseSelect
      class="flex-1"
      :model-value="selectedNodeId"
      :options="nodeOptions"
      placeholder="Select Node"
      searchable
      @update:modelValue="handleChange"
    >
      <template #icon>
        <div class="i-carbon-data-base text-lg"></div>
      </template>
    </BaseSelect>

    <button
      type="button"
      class="btn-secondary flex items-center justify-center px-3 aspect-square shadow-sm hover:shadow-md"
      :disabled="loading"
      @click="refreshNodes"
      :title="loading ? 'Refreshing…' : 'Refresh node list'"
    >
      <div :class="[loading ? 'animate-spin text-primary-500' : 'text-gray-500']" class="i-carbon-renew text-lg"></div>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useNodeStore } from '@/stores/node';
import { storeToRefs } from 'pinia';
import BaseSelect from '@/components/BaseSelect.vue';

interface Props {
  modelValue?: number | null;
}

interface Emits {
  (e: 'update:modelValue', value: number | null): void;
  (e: 'change', nodeId: number | null): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const nodeStore = useNodeStore();
const { sortedNodes, loading } = storeToRefs(nodeStore);

const selectedNodeId = ref<number | null>(props.modelValue || null);

const nodeOptions = computed(() => {
  return sortedNodes.value.map(node => ({
    label: `Node ${node.node_id} - ${node.node_type} ${node.maintenance_mode ? '(Maintenance)' : ''}`,
    value: node.node_id,
    icon: node.maintenance_mode ? 'i-carbon-warning-filled text-yellow-500' : 'i-carbon-cloud-satellite'
  }));
});

watch(
  () => props.modelValue,
  (newValue) => {
    selectedNodeId.value = newValue || null;
  }
);

const handleChange = (value: any) => {
  const nodeId = value as number | null;
  selectedNodeId.value = nodeId;
  emit('update:modelValue', nodeId);
  emit('change', nodeId);
};

const refreshNodes = async () => {
  await nodeStore.fetchNodes();
};

// 初始化时加载节点列表
onMounted(() => {
  nodeStore.fetchNodes();
});
</script>

<style scoped></style>

