<template>
  <div class="grid grid-cols-12 gap-6">
    <div class="col-span-12 xl:col-span-5">
      <ChainManagement :selected-node-id="selectedNodeId" />
    </div>

    <div class="col-span-12 xl:col-span-7 space-y-4">
      <div class="card-base p-5">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
          <div>
            <h3 class="text-lg font-bold text-gray-900">Canvas</h3>
            <p class="text-xs text-gray-400 mt-1">Drag nodes in, click to select (for Quick Input)</p>
          </div>

          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="resetCanvas"
          >
            <div class="i-carbon-center-circle text-lg"></div>
            <span>Reset</span>
          </button>
        </div>

        <div class="mt-4 grid grid-cols-1 lg:grid-cols-12 gap-4">
          <div class="lg:col-span-4">
            <div class="rounded-2xl border border-gray-100 bg-white shadow-sm overflow-hidden">
              <div class="p-3 border-b border-gray-100 bg-gray-50/60">
                <div class="text-xs font-semibold text-gray-500 uppercase tracking-wider">Nodes</div>
                <div class="mt-2 relative">
                  <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                    <div class="i-carbon-search"></div>
                  </div>
                  <input
                    v-model="filter"
                    type="text"
                    placeholder="Filter"
                    class="w-full pl-9 pr-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/30 focus:border-primary-300 transition-all"
                  />
                </div>
              </div>

              <div class="max-h-[520px] overflow-auto custom-scrollbar">
                <div
                  v-for="n in filteredNodes"
                  :key="n.node_id"
                  class="p-3 border-b border-gray-50 hover:bg-gray-50/40 transition-colors cursor-grab active:cursor-grabbing"
                  draggable="true"
                  @dragstart="(e) => onDragStart(e, n.node_id)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <div class="font-semibold text-gray-900 truncate">{{ n.name || `Node ${n.node_id}` }}</div>
                      <div class="text-xs text-gray-400 mt-1">
                        <span class="font-mono">{{ n.node_id }}</span>
                        <span class="mx-1">·</span>
                        <span>{{ n.node_type }}</span>
                      </div>
                    </div>
                    <div class="shrink-0">
                      <span
                        class="px-2 py-0.5 rounded-full text-[10px] font-semibold border"
                        :class="(n.node_type || '').toLowerCase().includes('transparent') ? 'bg-green-50 text-green-700 border-green-100' : 'bg-indigo-50 text-indigo-600 border-indigo-100'"
                      >
                        {{ (n.node_type || '').toLowerCase().includes('transparent') ? 'TRANSPARENT' : 'AUTH' }}
                      </span>
                    </div>
                  </div>
                </div>

                <div v-if="filteredNodes.length === 0" class="p-6 text-center text-sm text-gray-400">
                  No nodes
                </div>
              </div>
            </div>
          </div>

          <div class="lg:col-span-8 h-[620px]">
            <GraphCanvas
              :nodes="canvasNodes"
              :edges="[]"
              :connect-mode="false"
              :pending-from-node-id="null"
              :selected-node-id="selectedCanvasNodeId"
              :selected-edge-id="null"
              @drop-node="handleDropNode"
              @node-click="handleNodeClick"
              @node-drag="handleNodeDrag"
              @background-click="clearSelection"
            />
          </div>
        </div>

        <div class="mt-4 text-xs text-gray-500">
          <span class="text-gray-400">Selected node (for Quick Input):</span>
          <span class="font-mono ml-1">{{ selectedNodeId || '-' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { v4 as uuidv4 } from 'uuid';
import type { NodeInfo } from '@/api/types';
import { useNodeStore } from '@/stores/node';
import GraphCanvas from '@/components/GraphCanvas.vue';
import ChainManagement from '@/components/ChainManagement.vue';
import type { FlowCanvasNode } from '@/stores/flow';

interface Props {
}

defineProps<Props>();

const nodeStore = useNodeStore();

const filter = ref('');

const canvasNodes = ref<FlowCanvasNode[]>([]);
const selectedCanvasNodeId = ref<string | null>(null);

const selectedNodeId = computed(() => {
  if (!selectedCanvasNodeId.value) return null;
  const n = canvasNodes.value.find((x) => x.id === selectedCanvasNodeId.value);
  return n?.node_id ?? null;
});

const filteredNodes = computed<NodeInfo[]>(() => {
  const kw = filter.value.trim().toLowerCase();
  if (!kw) return nodeStore.sortedNodes;
  return nodeStore.sortedNodes.filter((n) => {
    const name = (n.name || '').toLowerCase();
    const region = (n.region || '').toLowerCase();
    const type = (n.node_type || '').toLowerCase();
    return (
      name.includes(kw) ||
      region.includes(kw) ||
      type.includes(kw) ||
      String(n.node_id).includes(kw)
    );
  });
});

const onDragStart = (event: DragEvent, nodeId: number) => {
  event.dataTransfer?.setData('application/x-node-id', String(nodeId));
  event.dataTransfer?.setData('text/plain', String(nodeId));
  event.dataTransfer?.setDragImage(new Image(), 0, 0);
};

const handleDropNode = ({ nodeId, x, y }: { nodeId: number; x: number; y: number }) => {
  const node = nodeStore.sortedNodes.find((n) => n.node_id === nodeId);
  if (!node) return;

  const existing = canvasNodes.value.find((n) => n.node_id === nodeId);
  if (existing) {
    existing.x = x;
    existing.y = y;
    selectedCanvasNodeId.value = existing.id;
    return;
  }

  const canvasNode: FlowCanvasNode = {
    id: uuidv4(),
    node_id: node.node_id,
    label: node.name || `Node ${node.node_id}`,
    node_type: node.node_type,
    region: node.region,
    description: node.description,
    x,
    y,
  };

  canvasNodes.value.push(canvasNode);
  selectedCanvasNodeId.value = canvasNode.id;
};

const handleNodeClick = (nodeCanvasId: string) => {
  selectedCanvasNodeId.value = nodeCanvasId;
};

const handleNodeDrag = ({ id, x, y }: { id: string; x: number; y: number }) => {
  const n = canvasNodes.value.find((nn) => nn.id === id);
  if (!n) return;
  n.x = x;
  n.y = y;
};

const clearSelection = () => {
  selectedCanvasNodeId.value = null;
};

const resetCanvas = () => {
  if (!confirm('Clear canvas nodes?')) return;
  canvasNodes.value = [];
  selectedCanvasNodeId.value = null;
};

onMounted(() => {
  nodeStore.fetchNodes();
});

</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: #e2e8f0;
  border-radius: 20px;
}
</style>
