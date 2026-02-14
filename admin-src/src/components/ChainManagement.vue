<template>
  <div class="chain-management space-y-4">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 tracking-tight">Chains</h2>
        <p class="text-sm text-gray-500 mt-1">Maintain chain metadata and route table (like routing rules)</p>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        <button
          type="button"
          class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
          :disabled="loading"
          @click="load"
        >
          <div :class="loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
          <span>Reload</span>
        </button>
        <button
          type="button"
          class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
          :disabled="nodesLoading"
          @click="refreshNodes"
        >
          <div :class="nodesLoading ? 'animate-spin' : ''" class="i-carbon-data-base text-lg"></div>
          <span>Refresh Nodes</span>
        </button>

        <button
          type="button"
          class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
          :disabled="!activeChainId || activeChainId <= 0 || applying"
          @click="applyActiveChain"
        >
          <div :class="applying ? 'animate-spin' : ''" class="i-carbon-rocket text-lg"></div>
          <span>Apply</span>
        </button>

        <button
          type="button"
          class="btn-primary flex items-center gap-2 shadow-lg hover:shadow-xl hover:-translate-y-0.5"
          @click="createChain"
        >
          <div class="i-carbon-add text-lg"></div>
          <span>Add Chain</span>
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="loading-ring w-10 h-10 relative mx-auto mb-4"></div>
        <p class="text-gray-400 text-sm font-medium tracking-wide">LOADING...</p>
      </div>
    </div>

    <div
      v-else-if="error"
      class="p-4 bg-red-50/80 border border-red-100 text-red-600 rounded-xl flex items-center gap-3"
    >
      <div class="i-carbon-warning-alt text-lg"></div>
      <span class="font-medium">{{ error }}</span>
    </div>

    <div v-else class="space-y-6">
      <div
        v-if="applyResult"
        class="p-4 rounded-xl border border-gray-100 bg-white shadow-sm"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-semibold text-gray-800">Apply Result</div>
          <button
            type="button"
            class="btn-secondary flex items-center gap-2"
            @click="applyResult = null"
          >
            <div class="i-carbon-close text-lg"></div>
            <span>Close</span>
          </button>
        </div>

        <div class="mt-3 space-y-1 text-sm">
          <div
            v-for="(r, idx) in applyResult.applied_nodes"
            :key="idx"
            class="flex items-center justify-between gap-3 rounded-lg px-3 py-2 border"
            :class="r.status === 'ok' ? 'border-emerald-100 bg-emerald-50/40 text-emerald-700' : 'border-red-100 bg-red-50/40 text-red-700'"
          >
            <div class="font-mono">node_id={{ r.node_id }}</div>
            <div class="font-mono">tag={{ r.tag }}</div>
            <div class="text-xs">{{ r.status }}</div>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
        <div class="lg:col-span-4 space-y-4">
          <div class="card-base p-5">
            <div class="flex items-start justify-between gap-4">
              <div>
                <h3 class="text-lg font-bold text-gray-900">Chain List</h3>
                <p class="text-xs text-gray-400 mt-1">Select one to edit its route table</p>
              </div>
              <button
                type="button"
                class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
                :disabled="!dirty"
                @click="save"
              >
                <div class="i-carbon-save text-lg"></div>
                <span>Save</span>
              </button>
            </div>

            <div class="mt-4 space-y-2">
              <div
                v-for="c in chains"
                :key="c.id"
                class="rounded-2xl border transition-all"
                :class="c.id === activeChainId ? 'border-primary-200 bg-primary-50/30 shadow-sm' : 'border-gray-100 bg-white hover:bg-gray-50/50'"
              >
                <button
                  type="button"
                  class="w-full text-left p-3"
                  @click="selectChain(c.id)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0 flex-1">
                      <div class="font-bold text-gray-900 truncate">{{ c.name }}</div>
                      <div class="mt-2 flex items-center gap-2">
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold border bg-green-50 text-green-700 border-green-100">
                          TRANSPARENT
                        </span>
                        <span class="text-[10px] text-gray-400">{{ c.routes.length }} hops</span>
                        <span class="text-[10px] text-gray-300">ID: {{ c.id }}</span>
                      </div>
                    </div>
                    <div class="flex flex-col items-end gap-2 shrink-0">
                      <button
                        type="button"
                        class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        title="Delete Chain"
                        @click.stop="deleteChain(c.id)"
                      >
                        <div class="i-carbon-trash-can text-lg"></div>
                      </button>
                    </div>
                  </div>
                </button>
              </div>

              <div v-if="chains.length === 0" class="py-10 text-center text-sm text-gray-400">
                No chain configured
              </div>
            </div>

            <div v-if="dirty" class="mt-4 p-3 rounded-xl border border-amber-100 bg-amber-50/60 text-amber-700 text-xs flex items-center gap-2">
              <div class="i-carbon-warning-alt"></div>
              <span>You have unsaved changes</span>
            </div>
          </div>

          <div v-if="selectedNodeId" class="card-base p-5">
            <h3 class="text-lg font-bold text-gray-900">Quick Input</h3>
            <p class="text-xs text-gray-400 mt-1">Use selected nodes from graph to fill route table faster</p>

            <div class="mt-4 space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <button
                  type="button"
                  class="btn-secondary flex items-center justify-center gap-2 shadow-sm hover:shadow-md"
                  :disabled="!selectedNodeId"
                  @click="setPendingFromSelected"
                >
                  <div class="i-carbon-arrow-right text-lg"></div>
                  <span>Set From</span>
                </button>
                <button
                  type="button"
                  class="btn-secondary flex items-center justify-center gap-2 shadow-sm hover:shadow-md"
                  :disabled="!selectedNodeId || !pendingFromNodeId"
                  @click="appendHopFromPending"
                >
                  <div class="i-carbon-add-alt text-lg"></div>
                  <span>Add Hop</span>
                </button>
              </div>

              <div class="text-xs text-gray-500">
                <div>
                  <span class="text-gray-400">Graph selected node:</span>
                  <span class="font-mono ml-1">{{ selectedNodeId || '-' }}</span>
                </div>
                <div>
                  <span class="text-gray-400">Pending From:</span>
                  <span class="font-mono ml-1">{{ pendingFromNodeId || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="lg:col-span-8 space-y-4">
          <div class="card-base p-6" v-if="activeChain">
            <div class="flex items-start justify-between gap-4">
              <div>
                <h3 class="text-lg font-bold text-gray-900">Chain Meta</h3>
                <p class="text-xs text-gray-400 mt-1">Name / description</p>
              </div>
            </div>

            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Type</label>
                <BaseSelect
                  :model-value="activeChain.chainType || 'tcp'"
                  :options="chainTypeOptions"
                  placeholder="Select type"
                  @update:modelValue="(v: string) => updateChainMeta({ chainType: v })"
                >
                  <template #icon>
                    <div class="i-carbon-network-3"></div>
                  </template>
                </BaseSelect>
              </div>

              <div>
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Name</label>
                <input
                  v-model="activeChain.name"
                  type="text"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/20 focus:border-primary-400 transition-all"
                  @input="markDirty"
                />
              </div>

              <div class="md:col-span-2">
                <label class="block text-xs font-medium text-gray-600 mb-1.5">Description</label>
                <textarea
                  v-model="activeChain.description"
                  rows="2"
                  class="w-full px-3 py-2.5 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/20 focus:border-primary-400 transition-all resize-none"
                  @input="markDirty"
                ></textarea>
              </div>
            </div>
          </div>

          <div class="card-base p-6" v-if="activeChain">
            <div class="flex items-start justify-between gap-4">
              <div>
                <h3 class="text-lg font-bold text-gray-900">Route Table</h3>
                <p class="text-xs text-gray-400 mt-1">Each row = one hop (from -> to)</p>
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
                  @click="addHop"
                >
                  <div class="i-carbon-add text-lg"></div>
                  <span>Add Row</span>
                </button>
                <button
                  type="button"
                  class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
                  @click="sortHops"
                >
                  <div class="i-carbon-sort-ascending text-lg"></div>
                  <span>Sort</span>
                </button>
              </div>
            </div>

            <div class="mt-4 rounded-xl border border-gray-100 bg-white shadow-sm max-h-[70vh] overflow-auto">
              <table class="min-w-full text-sm">
                <thead class="bg-gray-50/60 sticky top-0 z-10">
                  <tr class="text-left">
                    <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">#</th>
                    <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">From</th>
                    <th class="px-2 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Swap</th>
                    <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">To</th>
                    <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Remark</th>
                    <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="(row, idx) in activeChain.routes"
                    :key="row.id"
                    class="border-t border-gray-100 hover:bg-gray-50/40 transition-colors"
                  >
                    <td class="px-4 py-3 font-mono text-gray-500 whitespace-nowrap">{{ idx + 1 }}</td>
                    <td class="px-4 py-3">
                      <BaseSelect
                        :model-value="row.fromNodeId"
                        :options="nodeOptions"
                        searchable
                        placeholder="Select node"
                        @update:modelValue="(v: number) => updateRow(row.id, { fromNodeId: v })"
                      >
                        <template #icon>
                          <div class="i-carbon-data-base text-lg"></div>
                        </template>
                      </BaseSelect>
                    </td>
                    <td class="px-2 py-3 text-center">
                      <button
                        type="button"
                        class="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-full transition-colors"
                        title="Swap from/to"
                        @click="swapRowEndpoints(row.id)"
                      >
                        <div class="i-carbon-swap-horizontal text-base"></div>
                      </button>
                    </td>
                    <td class="px-4 py-3">
                      <BaseSelect
                        :model-value="row.toNodeId"
                        :options="nodeOptions"
                        searchable
                        placeholder="Select node"
                        @update:modelValue="(v: number) => updateRow(row.id, { toNodeId: v })"
                      >
                        <template #icon>
                          <div class="i-carbon-data-base text-lg"></div>
                        </template>
                      </BaseSelect>
                    </td>
                    <td class="px-4 py-3">
                      <input
                        :value="row.remark || ''"
                        type="text"
                        class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/20 focus:border-primary-400 transition-all"
                        placeholder="optional"
                        @input="(e: Event) => updateRow(row.id, { remark: (e.target as HTMLInputElement).value })"
                      />
                    </td>
                    <td class="px-4 py-3">
                      <div class="flex items-center gap-1">
                        <button
                          type="button"
                          class="p-1.5 text-gray-400 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                          title="Move up"
                          :disabled="idx === 0"
                          @click="moveRow(row.id, -1)"
                        >
                          <div class="i-carbon-chevron-up"></div>
                        </button>
                        <button
                          type="button"
                          class="p-1.5 text-gray-400 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                          title="Move down"
                          :disabled="idx === activeChain.routes.length - 1"
                          @click="moveRow(row.id, 1)"
                        >
                          <div class="i-carbon-chevron-down"></div>
                        </button>
                        <button
                          type="button"
                          class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          title="Delete"
                          @click="removeRow(row.id)"
                        >
                          <div class="i-carbon-trash-can"></div>
                        </button>
                      </div>
                    </td>
                  </tr>

                  <tr v-if="activeChain.routes.length === 0" class="border-t border-gray-100">
                    <td class="px-4 py-8 text-center text-sm text-gray-400" colspan="5">
                      No hops yet
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div v-else class="card-base p-10 text-center text-sm text-gray-400">
            Select a chain to edit
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { v4 as uuidv4 } from 'uuid';
import { useNodeStore } from '@/stores/node';
import { useToastStore } from '@/stores/toast';
import BaseSelect from '@/components/BaseSelect.vue';
import * as adminApi from '@/api/admin';
import type { ApplyChainResult, ChainDefinition, ChainRouteEntry, NodeInfo } from '@/api/types';

interface Props {
  selectedNodeId?: number | null;
}

const props = defineProps<Props>();

const toast = useToastStore();
const nodeStore = useNodeStore();

const loading = ref(false);
const error = ref<string | null>(null);
const dirty = ref(false);

const applying = ref(false);
const applyResult = ref<ApplyChainResult | null>(null);

const chains = ref<ChainDefinition[]>([]);
const activeChainId = ref<number | null>(null);

const pendingFromNodeId = ref<number | null>(null);

const buildNodeLabel = (node: NodeInfo) => {
  const base = node.name?.trim() || `Node ${node.node_id}`;
  const meta = node.node_type ? ` · ${node.node_type}` : '';
  const maintenance = node.maintenance_mode ? ' (Maintenance)' : '';
  return `${base}${meta}${maintenance}`;
};

const nodeOptions = computed(() => {
  return nodeStore.sortedNodes.map((n) => ({
    label: buildNodeLabel(n),
    value: n.node_id,
    icon: n.maintenance_mode ? 'i-carbon-warning-filled text-yellow-500' : 'i-carbon-cloud-satellite',
  }));
});

const activeChain = computed(() => {
  if (!activeChainId.value) return null;
  return chains.value.find((c) => c.id === activeChainId.value) || null;
});

const selectedNodeId = computed(() => props.selectedNodeId || null);
const nodesLoading = computed(() => nodeStore.loading);

const getDefaultNodeId = () => nodeStore.sortedNodes[0]?.node_id || 0;

const getAlternateNodeId = (exclude?: number | null) => {
  const candidate = nodeStore.sortedNodes.find((n) => n.node_id !== exclude)?.node_id;
  if (candidate !== undefined) return candidate;
  return exclude ?? getDefaultNodeId();
};

const markDirty = () => {
  dirty.value = true;
};

const updateChainMeta = (patch: Partial<ChainDefinition>) => {
  if (!activeChain.value) return;
  Object.assign(activeChain.value, patch);
  markDirty();
};

const chainTypeOptions = computed(() => {
  return [
    { label: 'TCP', value: 'tcp', icon: 'i-carbon-link' },
    { label: 'UDP', value: 'udp', icon: 'i-carbon-link' },
  ];
});

const normalizeRoutesOrder = (c: ChainDefinition) => {
  c.routes
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
    .forEach((r, idx) => {
      r.order = idx + 1;
    });
};

const load = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nodeStore.fetchNodes();
    const data = await adminApi.getChains();
    chains.value = data || [];
    if (!activeChainId.value) {
      activeChainId.value = chains.value[0]?.id ?? null;
    }
    dirty.value = false;
  } catch (err: any) {
    error.value = err?.message || 'Failed to load chains';
  } finally {
    loading.value = false;
  }
};

const save = async () => {
  if (!activeChain.value) {
    toast.error('Please select a chain to save');
    return;
  }
  try {
    const clone: ChainDefinition = JSON.parse(JSON.stringify(activeChain.value));
    normalizeRoutesOrder(clone);
    clone.protocol = 'transparent';
    clone.routes = (clone.routes || []).map((r) => ({
      ...r,
      mode: 'transparent',
    }));
    clone.updatedAt = new Date().toISOString();

    const saved = await adminApi.upsertChain({
      id: clone.id > 0 ? clone.id : undefined,
      name: clone.name,
      protocol: clone.protocol,
      chainType: clone.chainType || 'tcp',
      routes: clone.routes,
      description: clone.description,
    });

    const oldId = activeChainId.value;
    if (oldId !== null) {
      const idx = chains.value.findIndex((c) => c.id === oldId);
      if (idx >= 0) {
        chains.value[idx] = saved;
      } else {
        chains.value.push(saved);
      }
    }
    activeChainId.value = saved.id;
    dirty.value = false;
    toast.success('Chains saved');
  } catch (err: any) {
    toast.error(err?.message || 'Failed to save chains');
  }
};

const selectChain = (id: number) => {
  activeChainId.value = id;
  pendingFromNodeId.value = null;
};

const refreshNodes = async () => {
  await nodeStore.fetchNodes();
  toast.success('Node list refreshed');
};

const applyActiveChain = async () => {
  if (!activeChainId.value) return;
  if (activeChainId.value <= 0) {
    toast.error('Please save the chain before applying');
    return;
  }
  applying.value = true;
  applyResult.value = null;
  try {
    const res = await adminApi.applyChain({
      chain_id: activeChainId.value,
      base_port: 40000,
    });
    applyResult.value = res;
    toast.success('Chain applied');
  } catch (err: any) {
    toast.error(err?.message || 'Apply failed');
  } finally {
    applying.value = false;
  }
};

const createChain = () => {
  const now = new Date().toISOString();
  const nextTmpId =
    Math.min(0, ...chains.value.map((c) => (typeof c.id === 'number' ? c.id : 0))) - 1;
  const c: ChainDefinition = {
    id: nextTmpId,
    name: `Chain ${chains.value.length + 1}`,
    protocol: 'transparent',
    chainType: 'tcp',
    routes: [],
    createdAt: now,
    updatedAt: now,
    description: '',
  };
  chains.value.push(c);
  activeChainId.value = c.id;
  dirty.value = true;
};

const deleteChain = async (id: number) => {
  const c = chains.value.find((x) => x.id === id);
  if (!c) return;
  if (!confirm(`Delete chain "${c.name}"?`)) return;
  try {
    if (id > 0) {
      await adminApi.deleteChain(id);
    }
    chains.value = chains.value.filter((x) => x.id !== id);
    if (activeChainId.value === id) activeChainId.value = chains.value[0]?.id ?? null;
    dirty.value = false;
    toast.success('Deleted');
  } catch (err: any) {
    toast.error(err?.message || 'Delete failed');
  }
};

const addHop = () => {
  if (!activeChain.value) return;
  const validRoutes = activeChain.value.routes.filter((r) => r.fromNodeId !== r.toNodeId);
  const previous = validRoutes[validRoutes.length - 1];
  const fromNodeId = previous ? previous.toNodeId : getDefaultNodeId();
  const row: ChainRouteEntry = {
    id: uuidv4(),
    order: activeChain.value.routes.length + 1,
    fromNodeId,
    toNodeId: getAlternateNodeId(fromNodeId),
    mode: 'transparent',
    remark: '',
  };
  activeChain.value.routes.push(row);
  markDirty();
};

const removeRow = (rowId: string) => {
  if (!activeChain.value) return;
  activeChain.value.routes = activeChain.value.routes.filter((r) => r.id !== rowId);
  markDirty();
};

const moveRow = (rowId: string, delta: number) => {
  if (!activeChain.value) return;
  const idx = activeChain.value.routes.findIndex((r) => r.id === rowId);
  if (idx === -1) return;
  const next = idx + delta;
  if (next < 0 || next >= activeChain.value.routes.length) return;
  const arr = [...activeChain.value.routes];
  const tmp = arr[idx];
  arr[idx] = arr[next];
  arr[next] = tmp;
  activeChain.value.routes = arr;
  markDirty();
};

const sortHops = () => {
  if (!activeChain.value) return;
  normalizeRoutesOrder(activeChain.value);
  markDirty();
};

const updateRow = (rowId: string, patch: Partial<ChainRouteEntry>) => {
  if (!activeChain.value) return;
  const row = activeChain.value.routes.find((r) => r.id === rowId);
  if (!row) return;
  Object.assign(row, patch);
  markDirty();
};

const swapRowEndpoints = (rowId: string) => {
  if (!activeChain.value) return;
  const row = activeChain.value.routes.find((r) => r.id === rowId);
  if (!row) return;
  const { fromNodeId, toNodeId } = row;
  row.fromNodeId = toNodeId;
  row.toNodeId = fromNodeId;
  markDirty();
};

const setPendingFromSelected = () => {
  if (!selectedNodeId.value) return;
  pendingFromNodeId.value = selectedNodeId.value;
  toast.info('From node set');
};

const appendHopFromPending = () => {
  if (!activeChain.value) return;
  if (!pendingFromNodeId.value) return;
  if (!selectedNodeId.value) return;
  const row: ChainRouteEntry = {
    id: uuidv4(),
    order: activeChain.value.routes.length + 1,
    fromNodeId: pendingFromNodeId.value,
    toNodeId: selectedNodeId.value,
    mode: 'transparent',
    remark: '',
  };
  activeChain.value.routes.push(row);
  pendingFromNodeId.value = selectedNodeId.value;
  markDirty();
  toast.success('Hop added');
};

onMounted(() => {
  load();
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
