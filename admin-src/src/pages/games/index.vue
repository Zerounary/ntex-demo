<template>
  <div class="games-page min-h-screen p-4 md:p-6 lg:p-8">
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 tracking-tight mb-2">
            Game Config
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium text-sm">
            Search games and maintain acceleration bindings (TCP/UDP chains)
          </p>
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/')"
          >
            <div class="i-carbon-arrow-left text-lg"></div>
            <span>BACK TO NODES</span>
          </button>
        </div>
      </div>

      <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="relative md:col-span-2">
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
            <div class="i-carbon-search"></div>
          </div>
          <input
            v-model="keyword"
            type="text"
            placeholder="Search by name/region/status/process"
            class="w-full pl-9 pr-3 py-2.5 bg-white/70 border border-gray-200 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all shadow-sm"
            @keydown.enter="loadGames"
          />
        </div>
        <button
          class="btn-primary flex items-center justify-center gap-2.5 px-6 py-2.5"
          :disabled="loadingGames"
          @click="loadGames"
        >
          <div :class="loadingGames ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
          <span class="text-sm font-semibold tracking-wide">SEARCH</span>
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div class="bg-white/70 border border-gray-200 rounded-2xl shadow-sm overflow-hidden">
        <div class="p-4 border-b border-gray-100 flex items-center justify-between">
          <div class="text-sm font-semibold text-gray-700">Games</div>
          <div class="text-xs text-gray-400">{{ games.length }}</div>
        </div>

        <div v-if="gamesError" class="p-4 text-sm text-red-600">{{ gamesError }}</div>

        <div v-else class="max-h-[70vh] overflow-auto custom-scrollbar">
          <button
            v-for="g in games"
            :key="g.id"
            type="button"
            class="w-full text-left px-4 py-3 border-b border-gray-100 hover:bg-gray-50 transition-colors"
            :class="selectedGameId === g.id ? 'bg-primary-50' : ''"
            @click="selectGame(g.id)"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <div class="text-sm font-semibold text-gray-800 truncate">{{ g.name }}</div>
                <div class="text-xs text-gray-500 truncate">{{ g.region }} · {{ g.process_name }}</div>
              </div>
              <div class="text-xs text-gray-400">{{ g.status }}</div>
            </div>
          </button>

          <div v-if="!loadingGames && games.length === 0" class="p-6 text-sm text-gray-400">
            No games
          </div>
        </div>
      </div>

      <div class="lg:col-span-2 bg-white/70 border border-gray-200 rounded-2xl shadow-sm overflow-hidden">
        <div class="p-4 border-b border-gray-100 flex items-center justify-between">
          <div class="text-sm font-semibold text-gray-700">
            Bindings
            <span v-if="selectedGame" class="text-gray-400 font-normal">· {{ selectedGame.name }}</span>
          </div>
          <div class="flex items-center gap-2">
            <button
              class="btn-secondary flex items-center gap-2"
              type="button"
              :disabled="!selectedGameId || loadingBindings"
              @click="loadBindings"
            >
              <div :class="loadingBindings ? 'animate-spin' : ''" class="i-carbon-renew"></div>
              <span>REFRESH</span>
            </button>
          </div>
        </div>

        <div v-if="!selectedGameId" class="p-8 text-sm text-gray-400">
          Select a game to edit bindings.
        </div>

        <div v-else>
          <div v-if="bindingsError" class="p-4 text-sm text-red-600">{{ bindingsError }}</div>

          <div class="p-4 border-b border-gray-100">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-3 items-end">
              <BaseSelect
                v-model="newBindingType"
                :options="typeOptions"
                placeholder="Type"
                :disabled="savingNewBinding"
              >
                <template #icon>
                  <div class="i-carbon-category"></div>
                </template>
              </BaseSelect>

              <BaseSelect
                v-model="newBindingNodeId"
                :options="nodeOptions"
                placeholder="Select node"
                :disabled="loadingNodes || savingNewBinding || newBindingType !== 'node'"
                searchable
              >
                <template #icon>
                  <div class="i-carbon-network-3"></div>
                </template>
              </BaseSelect>

              <BaseSelect
                v-model="newBindingTcpChainId"
                :options="chainOptions"
                placeholder="TCP chain"
                :disabled="loadingChains || savingNewBinding"
                searchable
              >
                <template #icon>
                  <div class="i-carbon-link"></div>
                </template>
              </BaseSelect>

              <BaseSelect
                v-model="newBindingUdpChainId"
                :options="chainOptions"
                placeholder="UDP chain"
                :disabled="loadingChains || savingNewBinding"
                searchable
              >
                <template #icon>
                  <div class="i-carbon-link"></div>
                </template>
              </BaseSelect>
            </div>

            <div class="mt-3 flex items-center justify-end gap-2">
              <button
                class="btn-primary flex items-center gap-2"
                type="button"
                :disabled="(newBindingType === 'node' && !newBindingNodeId) || savingNewBinding"
                @click="addBinding"
              >
                <div :class="savingNewBinding ? 'animate-spin' : ''" class="i-carbon-add"></div>
                <span>ADD</span>
              </button>
            </div>
          </div>

          <div class="max-h-[60vh] overflow-auto custom-scrollbar space-y-4">
            <div
              v-for="b in bindings"
              :key="b.id"
              class="p-4 rounded-2xl border border-gray-500 shadow-lg transition-all hover:-translate-y-0.5 hover:shadow-2xl border-l-4 bg-gradient-to-br"
              :class="b.type === 'node'
                ? 'border-emerald-400 from-emerald-100 to-white'
                : 'border-sky-400 from-sky-100 to-white'"
            >
              <div class="flex flex-col gap-3">
                <div class="flex items-start justify-between gap-4">
                  <div class="min-w-0">
                    <div class="text-sm font-semibold text-gray-800 truncate">
                      {{ b.display_name || b.node_id || `binding-${b.id}` }}
                      <span class="text-xs text-gray-400 font-normal">(#{{ b.id }})</span>
                    </div>
                    <div class="text-xs text-gray-500">
                      {{ b.type }} · {{ b.node_id || '-' }} · {{ b.region }} · {{ b.mode }} · {{ b.ping }}ms · {{ b.status }}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      class="btn-secondary flex items-center gap-2"
                      type="button"
                      :disabled="savingId === b.id"
                      @click="saveBinding(b)"
                    >
                      <div :class="savingId === b.id ? 'animate-spin' : ''" class="i-carbon-save"></div>
                      <span>SAVE</span>
                    </button>
                    <button
                      class="btn-secondary flex items-center gap-2"
                      type="button"
                      :disabled="deletingId === b.id"
                      @click="removeBinding(b.id)"
                    >
                      <div :class="deletingId === b.id ? 'animate-spin' : ''" class="i-carbon-trash-can"></div>
                      <span>DELETE</span>
                    </button>
                  </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <BaseSelect
                    v-model="b.type"
                    :options="typeOptions"
                    placeholder="Type"
                    :disabled="savingId === b.id"
                    @change="() => saveBinding(b)"
                  />
                  <BaseSelect
                    v-model="b.node_id"
                    :options="nodeOptionsWithNone"
                    placeholder="Node (type=node)"
                    :disabled="savingId === b.id || b.type !== 'node'"
                    searchable
                    @change="() => saveBinding(b)"
                  />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <input
                    v-model="b.display_name"
                    type="text"
                    placeholder="Display name"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                  <input
                    v-model="b.region"
                    type="text"
                    placeholder="Region"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                  <input
                    v-model="b.mode"
                    type="text"
                    placeholder="Mode"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                  <input
                    v-model.number="b.ping"
                    type="number"
                    placeholder="Ping (ms)"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                  <input
                    v-model="b.status"
                    type="text"
                    placeholder="Status"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <BaseSelect
                    v-model="b.tcp_chain_id"
                    :options="chainOptions"
                    placeholder="TCP chain"
                    :disabled="savingId === b.id"
                    searchable
                    @change="() => saveBinding(b)"
                  />
                  <BaseSelect
                    v-model="b.udp_chain_id"
                    :options="chainOptions"
                    placeholder="UDP chain"
                    :disabled="savingId === b.id"
                    searchable
                    @change="() => saveBinding(b)"
                  />
                </div>

                <div>
                  <input
                    v-model="b.remark"
                    type="text"
                    placeholder="Remark"
                    class="w-full px-3 py-2 bg-white border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-400/40 focus:border-primary-300 transition-all"
                    :disabled="savingId === b.id"
                    @blur="() => saveBinding(b)"
                  />
                </div>
              </div>
            </div>

            <div v-if="!loadingBindings && bindings.length === 0" class="p-8 text-sm text-gray-400">
              No bindings
            </div>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import BaseSelect from '@/components/BaseSelect.vue';
import * as adminApi from '@/api/admin';
import { useToastStore } from '@/stores/toast';
import type { ChainDefinition, NodeInfo } from '@/api/types';

const router = useRouter();
const toast = useToastStore();

const keyword = ref('');

const games = ref<adminApi.GameVO[]>([]);
const loadingGames = ref(false);
const gamesError = ref<string | null>(null);

const selectedGameId = ref<string | null>(null);

const nodes = ref<NodeInfo[]>([]);
const loadingNodes = ref(false);

const chains = ref<ChainDefinition[]>([]);
const loadingChains = ref(false);

const bindings = ref<adminApi.GameBindingVO[]>([]);
const loadingBindings = ref(false);
const bindingsError = ref<string | null>(null);

const savingId = ref<number | null>(null);
const deletingId = ref<number | null>(null);

const newBindingType = ref<'node' | 'chain'>('node');
const newBindingNodeId = ref<string | null>(null);
const newBindingTcpChainId = ref<number | null>(null);
const newBindingUdpChainId = ref<number | null>(null);
const savingNewBinding = ref(false);

const selectedGame = computed(() =>
  games.value.find((g: adminApi.GameVO) => g.id === selectedGameId.value) || null
);

const nodeOptions = computed(() => {
  return nodes.value.map((n: NodeInfo) => ({
    label: `${n.node_id} · ${n.name || '-'} · ${n.region || '-'} · ${n.is_online ? 'online' : 'offline'}`,
    value: String(n.node_id),
  }));
});

const nodeOptionsWithNone = computed(() => {
  const opts: Array<{ label: string; value: string | null }> = [{ label: 'None', value: null }];
  for (const n of nodes.value) {
    opts.push({
      label: `${n.node_id} · ${n.name || '-'} · ${n.region || '-'} · ${n.is_online ? 'online' : 'offline'}`,
      value: String(n.node_id),
    });
  }
  return opts;
});

const typeOptions = computed(() => [
  { label: 'node', value: 'node' },
  { label: 'chain', value: 'chain' },
]);

const chainOptions = computed(() => {
  const opts: Array<{ label: string; value: number | null }> = [{ label: 'None', value: null }];
  for (const c of chains.value) {
    opts.push({ label: `${c.id} · ${c.name}`, value: c.id });
  }
  return opts;
});

const loadGames = async () => {
  loadingGames.value = true;
  gamesError.value = null;
  try {
    games.value = await adminApi.listGames({ keyword: keyword.value.trim() || undefined });
  } catch (e: any) {
    gamesError.value = e?.message || 'Failed to load games';
  } finally {
    loadingGames.value = false;
  }
};

const loadNodes = async () => {
  loadingNodes.value = true;
  try {
    nodes.value = await adminApi.listNodes();
  } catch (e: any) {
    toast.error(e?.message || 'Failed to load accelerator nodes');
  } finally {
    loadingNodes.value = false;
  }
};

const loadChains = async () => {
  loadingChains.value = true;
  try {
    chains.value = await adminApi.getChains();
  } catch (e: any) {
    toast.error(e?.message || 'Failed to load chains');
  } finally {
    loadingChains.value = false;
  }
};

const loadBindings = async () => {
  if (!selectedGameId.value) return;
  loadingBindings.value = true;
  bindingsError.value = null;
  try {
    bindings.value = await adminApi.listGameBindings(selectedGameId.value);
  } catch (e: any) {
    bindingsError.value = e?.message || 'Failed to load bindings';
  } finally {
    loadingBindings.value = false;
  }
};

const selectGame = async (gameId: string) => {
  selectedGameId.value = gameId;
  newBindingType.value = 'node';
  newBindingNodeId.value = null;
  newBindingTcpChainId.value = null;
  newBindingUdpChainId.value = null;
  await loadBindings();
};

const addBinding = async () => {
  if (!selectedGameId.value) return;
  if (newBindingType.value === 'node' && !newBindingNodeId.value) return;
  savingNewBinding.value = true;
  try {
    const saved = await adminApi.upsertGameBinding(selectedGameId.value, {
      type: newBindingType.value,
      node_id: newBindingType.value === 'node' ? newBindingNodeId.value : null,
      tcp_chain_id: newBindingTcpChainId.value,
      udp_chain_id: newBindingUdpChainId.value,
    });
    const idx = bindings.value.findIndex((b: adminApi.GameBindingVO) => b.id === saved.id);
    if (idx >= 0) bindings.value[idx] = saved;
    else bindings.value.unshift(saved);

    toast.success('Saved');
    newBindingNodeId.value = null;
    newBindingTcpChainId.value = null;
    newBindingUdpChainId.value = null;
  } catch (e: any) {
    toast.error(e?.message || 'Save failed');
  } finally {
    savingNewBinding.value = false;
  }
};

const saveBinding = async (b: adminApi.GameBindingVO) => {
  if (!selectedGameId.value) return;
  savingId.value = b.id;
  try {
    const saved = await adminApi.upsertGameBinding(selectedGameId.value, {
      id: b.id,
      type: b.type,
      node_id: b.type === 'node' ? (b.node_id ?? null) : null,
      tcp_chain_id: b.tcp_chain_id ?? null,
      udp_chain_id: b.udp_chain_id ?? null,
      display_name: b.display_name ?? null,
      region: b.region ?? null,
      mode: b.mode ?? null,
      ping: b.ping ?? null,
      status: b.status ?? null,
      remark: b.remark ?? null,
    });
    const idx = bindings.value.findIndex((x: adminApi.GameBindingVO) => x.id === saved.id);
    if (idx >= 0) bindings.value[idx] = { ...bindings.value[idx], ...saved };
    toast.success('Saved');
  } catch (e: any) {
    toast.error(e?.message || 'Save failed');
  } finally {
    savingId.value = null;
  }
};

const removeBinding = async (id: number) => {
  if (!selectedGameId.value) return;
  deletingId.value = id;
  try {
    await adminApi.deleteGameBinding(selectedGameId.value, id);
    bindings.value = bindings.value.filter((b: adminApi.GameBindingVO) => b.id !== id);
    toast.success('Deleted');
  } catch (e: any) {
    toast.error(e?.message || 'Delete failed');
  } finally {
    deletingId.value = null;
  }
};

onMounted(async () => {
  await Promise.all([loadGames(), loadNodes(), loadChains()]);
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
  background: rgba(156, 163, 175, 0.5);
  border-radius: 999px;
}
</style>
