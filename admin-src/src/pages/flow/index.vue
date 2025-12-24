<template>
  <div class="flow-builder-page min-h-screen p-6 md:p-10 lg:p-14">
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-4xl font-bold text-gray-900 tracking-tight mb-3">
            Flow Builder
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium">Drag nodes onto canvas and build directed routing chains</p>
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="openImportExport = true"
          >
            <div class="i-carbon-document-import text-lg"></div>
            <span>Import/Export</span>
          </button>

          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="resetViewKey++"
          >
            <div class="i-carbon-center-circle text-lg"></div>
            <span>Reset View</span>
          </button>

          <button
            type="button"
            class="btn-primary flex items-center gap-2"
            :class="flowStore.connectMode ? 'shadow-lg shadow-indigo-500/30' : ''"
            @click="toggleConnectMode"
          >
            <div :class="flowStore.connectMode ? 'i-carbon-link text-lg' : 'i-carbon-link text-lg opacity-90'"></div>
            <span>{{ flowStore.connectMode ? 'CONNECTING' : 'CONNECT' }}</span>
          </button>
        </div>
      </div>

      <div class="mt-6 flex items-center gap-4">
        <button
          type="button"
          class="btn-ghost flex items-center gap-2"
          @click="toggleShowAllChains"
        >
          <div :class="flowStore.showAllChains ? 'i-carbon-view-filled text-lg' : 'i-carbon-view text-lg'"></div>
          <span>{{ flowStore.showAllChains ? 'Show All Chains' : 'Show Active Chain' }}</span>
        </button>

        <div class="text-xs text-gray-400">
          {{ flowStore.showAllChains ? 'Edges of all chains are visible' : 'Only active chain edges are visible' }}
        </div>
      </div>
    </div>

    <div class="grid grid-cols-12 gap-6">
      <div class="col-span-12 lg:col-span-4 space-y-6">
        <div class="card-base p-5">
          <div class="flex items-start justify-between gap-4">
            <div>
              <h2 class="text-lg font-bold text-gray-900">Chains</h2>
              <p class="text-xs text-gray-400 mt-1">One chain = one UUID</p>
            </div>
            <button
              type="button"
              class="btn-primary flex items-center gap-2"
              @click="createChain"
            >
              <div class="i-carbon-add text-lg"></div>
              <span>Create</span>
            </button>
          </div>

          <div class="mt-4 space-y-2">
            <div
              v-for="chain in flowStore.chains"
              :key="chain.id"
              class="rounded-2xl border transition-all"
              :class="chain.id === flowStore.activeChainId ? 'border-primary-200 bg-primary-50/30 shadow-sm' : 'border-gray-100 bg-white hover:bg-gray-50/50'"
            >
              <div class="p-3">
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <div class="w-2 h-2 rounded-full" :class="chain.id === flowStore.activeChainId ? 'bg-primary-500' : 'bg-gray-300'"></div>

                      <button
                        type="button"
                        class="min-w-0 text-left"
                        @click="flowStore.setActiveChain(chain.id)"
                      >
                        <div class="font-bold text-gray-900 truncate">
                          {{ chain.name }}
                        </div>
                      </button>

                      <span
                        class="px-2 py-0.5 rounded-full text-[10px] font-semibold border"
                        :class="chain.protocol === 'vmess' ? 'bg-indigo-50 text-indigo-600 border-indigo-100' : 'bg-green-50 text-green-700 border-green-100'"
                      >
                        {{ chain.protocol.toUpperCase() }}
                      </span>
                    </div>

                    <div class="mt-2">
                      <div class="text-[10px] text-gray-400 uppercase tracking-wider font-semibold">UUID</div>
                      <div class="mt-1 flex items-center gap-2">
                        <div class="text-xs font-mono text-gray-700 break-all">{{ chain.uuid }}</div>
                      </div>
                    </div>
                  </div>

                  <div class="flex flex-col items-end gap-2 shrink-0">
                    <button
                      type="button"
                      class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                      title="Copy UUID"
                      @click="copy(chain.uuid)"
                    >
                      <div class="i-carbon-copy text-lg"></div>
                    </button>

                    <button
                      type="button"
                      class="p-1.5 text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
                      title="Regenerate UUID"
                      @click="regenerateChainUuid(chain.id)"
                    >
                      <div class="i-carbon-renew text-lg"></div>
                    </button>

                    <button
                      type="button"
                      class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                      title="Delete Chain"
                      @click="deleteChain(chain.id)"
                    >
                      <div class="i-carbon-trash-can text-lg"></div>
                    </button>
                  </div>
                </div>

                <div class="mt-3 grid grid-cols-2 gap-2">
                  <BaseSelect
                    :model-value="chain.protocol"
                    :options="protocolOptions"
                    @update:modelValue="(v) => flowStore.setChainProtocol(chain.id, v)"
                  >
                    <template #icon>
                      <div class="i-carbon-security text-lg"></div>
                    </template>
                  </BaseSelect>

                  <button
                    type="button"
                    class="btn-ghost flex items-center justify-center gap-2"
                    @click="renameChain(chain.id)"
                  >
                    <div class="i-carbon-edit text-lg"></div>
                    <span>Rename</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="card-base p-5">
          <div class="flex items-start justify-between gap-4">
            <div>
              <h2 class="text-lg font-bold text-gray-900">Nodes</h2>
              <p class="text-xs text-gray-400 mt-1">Drag into canvas</p>
            </div>
            <button
              type="button"
              class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
              :disabled="nodeStore.loading"
              @click="refreshNodes"
            >
              <div :class="nodeStore.loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
              <span>Refresh</span>
            </button>
          </div>

          <div class="mt-4 grid grid-cols-1 gap-3">
            <div class="relative">
              <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                <div class="i-carbon-search"></div>
              </div>
              <input
                v-model="filterName"
                type="text"
                placeholder="Filter by name"
                class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
              />
            </div>

            <div class="relative">
              <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                <div class="i-carbon-earth"></div>
              </div>
              <input
                v-model="filterRegion"
                type="text"
                placeholder="Filter by region"
                class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
              />
            </div>

            <BaseSelect
              v-model="filterOnline"
              :options="onlineOptions"
              placeholder="All status"
            >
              <template #icon>
                <div class="i-carbon-signal-strength text-lg"></div>
              </template>
            </BaseSelect>
          </div>

          <div class="mt-4">
            <div v-if="nodeStore.loading && nodeStore.nodes.length === 0" class="flex flex-col items-center justify-center py-10">
              <div class="loading-ring w-10 h-10 relative mb-4"></div>
              <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
            </div>

            <div v-else class="space-y-2 max-h-[46vh] overflow-y-auto pr-1">
              <div
                v-for="node in filteredNodes"
                :key="node.node_id"
                class="rounded-2xl border border-gray-100 bg-white hover:bg-gray-50/50 transition-colors p-3 cursor-grab active:cursor-grabbing"
                draggable="true"
                @dragstart="(e) => onDragStartNode(e, node.node_id)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="font-bold text-gray-800 truncate">{{ node.name || `Node ${node.node_id}` }}</div>
                    <div class="text-xs text-gray-400 mt-0.5">#{{ node.node_id }} · {{ node.node_type }}</div>
                    <div v-if="node.region" class="mt-1">
                      <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-primary-50 text-primary-700 border border-primary-100">
                        {{ node.region }}
                      </span>
                    </div>
                  </div>
                  <div class="shrink-0">
                    <span
                      class="px-2.5 py-1 rounded-lg text-[10px] font-semibold"
                      :class="node.maintenance_mode ? 'bg-yellow-50 text-yellow-700' : node.is_online ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'"
                    >
                      {{ node.maintenance_mode ? 'MAINT' : node.is_online ? 'ONLINE' : 'OFFLINE' }}
                    </span>
                  </div>
                </div>
              </div>

              <div v-if="filteredNodes.length === 0" class="py-10 text-center text-sm text-gray-400">
                No nodes found
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="col-span-12 lg:col-span-8">
        <div class="card-base p-4 md:p-6" style="height: calc(100vh - 240px)">
          <div class="h-full relative">
            <GraphCanvas
              :key="resetViewKey"
              :nodes="flowStore.nodes"
              :edges="flowStore.edgesForDisplay"
              :connect-mode="flowStore.connectMode"
              :pending-from-node-id="flowStore.pendingFromNodeId"
              :selected-node-id="flowStore.selection?.type === 'node' ? flowStore.selection.id : null"
              :selected-edge-id="flowStore.selection?.type === 'edge' ? flowStore.selection.id : null"
              @drop-node="handleDropNode"
              @node-click="handleNodeClick"
              @edge-click="handleEdgeClick"
              @node-drag="({ id, x, y }) => flowStore.updateNodePosition(id, x, y)"
              @background-click="flowStore.clearSelection"
            />

            <transition name="slide-fade">
              <div
                v-if="selectedPanel"
                class="absolute top-4 right-4 w-80 bg-white/90 backdrop-blur-md border border-gray-200 rounded-2xl shadow-xl z-20 overflow-hidden animate-slide-up"
              >
                <div class="p-4 border-b border-gray-100 flex justify-between items-center bg-gray-50/50">
                  <h3 class="font-bold text-gray-800 flex items-center gap-2">
                    <div
                      class="w-2 h-2 rounded-full"
                      :class="selectedPanelDot"
                    ></div>
                    {{ selectedPanelTitle }}
                  </h3>
                  <button
                    type="button"
                    class="text-gray-400 hover:text-gray-600 transition-colors p-1 rounded-lg hover:bg-gray-100"
                    @click="flowStore.clearSelection"
                  >
                    <div class="i-carbon-close text-lg"></div>
                  </button>
                </div>

                <div class="p-4 max-h-[70vh] overflow-y-auto custom-scrollbar">
                  <template v-if="flowStore.selectedNode">
                    <div class="space-y-3">
                      <div class="space-y-1">
                        <div class="text-xs text-gray-400 uppercase tracking-wider font-semibold">Node</div>
                        <div class="text-sm font-bold text-gray-900">{{ flowStore.selectedNode.label }}</div>
                        <div class="text-xs text-gray-400">#{{ flowStore.selectedNode.node_id }} · {{ flowStore.selectedNode.node_type }}</div>
                      </div>

                      <div class="grid grid-cols-2 gap-2">
                        <button
                          type="button"
                          class="btn-secondary flex items-center justify-center gap-2"
                          @click="centerOnSelected"
                        >
                          <div class="i-carbon-center-circle text-lg"></div>
                          <span>Focus</span>
                        </button>
                        <button
                          type="button"
                          class="btn-secondary flex items-center justify-center gap-2"
                          @click="copy(String(flowStore.selectedNode.node_id))"
                        >
                          <div class="i-carbon-copy text-lg"></div>
                          <span>Copy ID</span>
                        </button>
                      </div>

                      <button
                        type="button"
                        class="w-full btn-secondary flex items-center justify-center gap-2 border border-red-100 text-red-600 hover:bg-red-50"
                        @click="removeSelectedNode"
                      >
                        <div class="i-carbon-trash-can text-lg"></div>
                        <span>Remove Node</span>
                      </button>
                    </div>
                  </template>

                  <template v-else-if="flowStore.selectedEdge">
                    <div class="space-y-3">
                      <div class="space-y-1">
                        <div class="text-xs text-gray-400 uppercase tracking-wider font-semibold">Chain</div>
                        <div class="text-sm font-bold text-gray-900">{{ chainOfSelectedEdge?.name || 'Unknown' }}</div>
                      </div>

                      <div v-if="chainOfSelectedEdge" class="space-y-2">
                        <div class="text-xs text-gray-400 uppercase tracking-wider font-semibold">UUID</div>
                        <div class="flex items-center gap-2">
                          <div class="text-xs font-mono text-gray-700 break-all">{{ chainOfSelectedEdge.uuid }}</div>
                          <button
                            type="button"
                            class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                            @click="copy(chainOfSelectedEdge.uuid)"
                            title="Copy UUID"
                          >
                            <div class="i-carbon-copy text-lg"></div>
                          </button>
                        </div>
                      </div>

                      <div class="grid grid-cols-2 gap-2">
                        <button
                          type="button"
                          class="btn-secondary flex items-center justify-center gap-2"
                          @click="gotoChain(chainOfSelectedEdge?.id)"
                        >
                          <div class="i-carbon-list text-lg"></div>
                          <span>Open Chain</span>
                        </button>
                        <button
                          type="button"
                          class="btn-secondary flex items-center justify-center gap-2"
                          @click="removeSelectedEdge"
                        >
                          <div class="i-carbon-trash-can text-lg"></div>
                          <span>Delete Edge</span>
                        </button>
                      </div>

                      <details class="cursor-pointer">
                        <summary class="text-xs font-medium text-gray-400 hover:text-primary-600 transition-colors flex items-center gap-1 select-none">
                          <div class="i-carbon-chevron-right group-open:rotate-90 transition-transform"></div>
                          View Raw
                        </summary>
                        <div class="mt-2 p-3 bg-gray-50/80 rounded-lg border border-gray-100">
                          <pre class="text-[10px] text-gray-600 font-mono overflow-x-auto whitespace-pre-wrap">{{ JSON.stringify(flowStore.selectedEdge, null, 2) }}</pre>
                        </div>
                      </details>
                    </div>
                  </template>
                </div>
              </div>
            </transition>
          </div>
        </div>
      </div>
    </div>

    <transition name="modal">
      <div
        v-if="openImportExport"
        class="fixed inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 transition-all"
      >
        <div class="bg-white rounded-2xl shadow-xl max-w-3xl w-full max-h-[90vh] overflow-y-auto animate-scale-in border border-gray-100">
          <div class="p-6 border-b border-gray-100 bg-gray-50/50 flex justify-between items-center">
            <h3 class="text-lg font-bold text-gray-900">Import / Export</h3>
            <button
              type="button"
              class="text-gray-400 hover:text-gray-600 transition-colors"
              @click="openImportExport = false"
            >
              <div class="i-carbon-close text-xl"></div>
            </button>
          </div>

          <div class="p-6 space-y-4">
            <div class="flex items-center gap-3">
              <button
                type="button"
                class="btn-primary flex items-center gap-2"
                @click="refreshExport"
              >
                <div class="i-carbon-document-export text-lg"></div>
                <span>Refresh Export</span>
              </button>

              <button
                type="button"
                class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
                @click="copy(exportText)"
              >
                <div class="i-carbon-copy text-lg"></div>
                <span>Copy JSON</span>
              </button>

              <button
                type="button"
                class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md border border-red-100 text-red-600 hover:bg-red-50"
                @click="clearAll"
              >
                <div class="i-carbon-trash-can text-lg"></div>
                <span>Clear All</span>
              </button>
            </div>

            <textarea
              v-model="exportText"
              class="w-full min-h-[240px] px-3 py-2.5 bg-white border border-gray-200 rounded-2xl text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
            ></textarea>

            <div class="flex items-center justify-end gap-3">
              <button
                type="button"
                class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
                @click="openImportExport = false"
              >
                <div class="i-carbon-close text-lg"></div>
                <span>Close</span>
              </button>
              <button
                type="button"
                class="btn-primary flex items-center gap-2"
                @click="importJson"
              >
                <div class="i-carbon-document-import text-lg"></div>
                <span>Import</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useNodeStore } from '@/stores/node';
import { useFlowStore } from '@/stores/flow';
import { useToastStore } from '@/stores/toast';
import BaseSelect from '@/components/BaseSelect.vue';
import GraphCanvas from '@/components/GraphCanvas.vue';

const nodeStore = useNodeStore();
const flowStore = useFlowStore();
const toastStore = useToastStore();

const resetViewKey = ref(0);

const filterName = ref('');
const filterRegion = ref('');
const filterOnline = ref<'all' | 'online' | 'offline'>('all');

const onlineOptions = [
  { label: 'All status', value: 'all' },
  { label: 'Online', value: 'online' },
  { label: 'Offline', value: 'offline' },
];

const protocolOptions = [
  { label: 'VMess (UUID auth)', value: 'vmess' },
  { label: 'Transparent', value: 'transparent' },
];

const onlineParam = computed(() => {
  if (filterOnline.value === 'online') return true;
  if (filterOnline.value === 'offline') return false;
  return undefined;
});

const filteredNodes = computed(() => {
  const nameQ = filterName.value.trim().toLowerCase();
  const regionQ = filterRegion.value.trim().toLowerCase();

  return nodeStore.sortedNodes.filter((n) => {
    const name = (n.name || '').toLowerCase();
    const region = (n.region || '').toLowerCase();

    if (nameQ && !name.includes(nameQ)) return false;
    if (regionQ && !region.includes(regionQ)) return false;
    return true;
  });
});

const openImportExport = ref(false);
const exportText = ref('');

const refreshExport = () => {
  exportText.value = flowStore.exportJson();
  toastStore.success('Export refreshed');
};

const importJson = () => {
  try {
    flowStore.importJson(exportText.value);
    toastStore.success('Imported successfully');
    openImportExport.value = false;
  } catch (err: any) {
    toastStore.error(err?.message || 'Import failed');
  }
};

const clearAll = () => {
  if (!confirm('This will clear all nodes/edges/chains. Continue?')) return;
  flowStore.clearAll('CONFIRM');
  toastStore.success('Cleared');
  exportText.value = flowStore.exportJson();
};

const refreshNodes = () => {
  nodeStore.fetchNodes(onlineParam.value);
};

watch(
  () => filterOnline.value,
  () => {
    nodeStore.fetchNodes(onlineParam.value);
  }
);

onMounted(() => {
  flowStore.ensureInitialized();
  nodeStore.fetchNodes(onlineParam.value);
  exportText.value = flowStore.exportJson();
});

const createChain = () => {
  const name = prompt('Chain name:', `Chain ${flowStore.chains.length + 1}`);
  if (name === null) return;
  flowStore.createChain(name);
  toastStore.success('Chain created');
};

const renameChain = (chainId: string) => {
  const c = flowStore.chains.find((x) => x.id === chainId);
  if (!c) return;
  const name = prompt('Rename chain:', c.name);
  if (name === null) return;
  flowStore.renameChain(chainId, name);
  toastStore.success('Renamed');
};

const deleteChain = (chainId: string) => {
  const c = flowStore.chains.find((x) => x.id === chainId);
  if (!c) return;
  if (!confirm(`Delete chain "${c.name}" and all its edges?`)) return;
  flowStore.deleteChain(chainId);
  toastStore.success('Deleted');
};

const regenerateChainUuid = (chainId: string) => {
  const c = flowStore.chains.find((x) => x.id === chainId);
  if (!c) return;
  if (!confirm(`Regenerate UUID for "${c.name}"? Existing clients will fail until updated.`)) return;
  const uuid = flowStore.regenerateChainUuid(chainId);
  if (uuid) {
    toastStore.success('UUID regenerated');
  }
};

const toggleConnectMode = () => {
  flowStore.setConnectMode(!flowStore.connectMode);
  toastStore.info(flowStore.connectMode ? 'Connect mode on' : 'Connect mode off');
};

const toggleShowAllChains = () => {
  flowStore.toggleShowAllChains();
};

const onDragStartNode = (event: DragEvent, nodeId: number) => {
  event.dataTransfer?.setData('application/x-node-id', String(nodeId));
  event.dataTransfer?.setData('text/plain', String(nodeId));
  event.dataTransfer?.setDragImage?.(new Image(), 0, 0);
};

const handleDropNode = ({ nodeId, x, y }: { nodeId: number; x: number; y: number }) => {
  const node = nodeStore.nodes.find((n) => n.node_id === nodeId);
  if (!node) {
    toastStore.error('Node not found');
    return;
  }
  flowStore.addNodeFromNodeInfo(node, x, y);
  toastStore.success('Node added');
};

const handleNodeClick = (id: string) => {
  if (flowStore.connectMode) {
    flowStore.beginOrCompleteConnect(id);
    return;
  }
  flowStore.selectNode(id);
};

const handleEdgeClick = (id: string) => {
  flowStore.selectEdge(id);
};

const removeSelectedNode = () => {
  if (!flowStore.selectedNode) return;
  if (!confirm('Remove node and its related edges?')) return;
  flowStore.removeNode(flowStore.selectedNode.id);
  toastStore.success('Node removed');
};

const removeSelectedEdge = () => {
  if (!flowStore.selectedEdge) return;
  flowStore.removeEdge(flowStore.selectedEdge.id);
  toastStore.success('Edge removed');
};

const copy = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    toastStore.success('Copied');
  } catch {
    toastStore.error('Copy failed');
  }
};

const chainOfSelectedEdge = computed(() => {
  if (!flowStore.selectedEdge) return null;
  return flowStore.chains.find((c) => c.id === flowStore.selectedEdge!.chain_id) || null;
});

const gotoChain = (id?: string) => {
  if (!id) return;
  flowStore.setActiveChain(id);
  toastStore.info('Switched to chain');
};

const selectedPanel = computed(() => {
  return !!flowStore.selectedNode || !!flowStore.selectedEdge;
});

const selectedPanelTitle = computed(() => {
  if (flowStore.selectedNode) return 'Node Details';
  if (flowStore.selectedEdge) return 'Edge Details';
  return '';
});

const selectedPanelDot = computed(() => {
  if (flowStore.selectedNode) return 'bg-blue-500';
  if (flowStore.selectedEdge) return 'bg-indigo-500';
  return 'bg-gray-300';
});

const centerOnSelected = () => {
  resetViewKey.value++;
};
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

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s var(--ease-smooth);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
