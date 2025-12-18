<template>
  <div class="node-selector">
    <select
      v-model="selectedNodeId"
      class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      @change="handleChange"
    >
      <option value="">请选择节点</option>
      <option
        v-for="node in sortedNodes"
        :key="node.node_id"
        :value="node.node_id"
      >
        节点 {{ node.node_id }} - {{ node.node_type }}
        <span v-if="node.maintenance_mode">(维护中)</span>
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useNodeStore } from '@/stores/node';
import { storeToRefs } from 'pinia';

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
const { sortedNodes } = storeToRefs(nodeStore);

const selectedNodeId = ref<number | null>(props.modelValue || null);

watch(
  () => props.modelValue,
  (newValue) => {
    selectedNodeId.value = newValue || null;
  }
);

const handleChange = () => {
  emit('update:modelValue', selectedNodeId.value);
  emit('change', selectedNodeId.value);
};

// 初始化时加载节点列表
nodeStore.fetchNodes();
</script>

<style scoped></style>

