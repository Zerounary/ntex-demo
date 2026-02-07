<template>
  <div class="games-manage-page min-h-screen p-4 md:p-6 lg:p-8">
    <div class="mb-8 animate-slide-up">
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 tracking-tight mb-2">
            Games
            <span class="text-primary-400">.</span>
          </h1>
          <p class="text-gray-500 font-medium text-sm">CRUD for accelerator_games</p>
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            @click="router.push('/games')"
          >
            <div class="i-carbon-arrow-left text-lg"></div>
            <span>BACK</span>
          </button>
          <button
            type="button"
            class="btn-primary flex items-center gap-2.5 px-6 py-2.5"
            :disabled="loading"
            @click="openCreate"
          >
            <div class="i-carbon-add text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">NEW GAME</span>
          </button>
          <button
            type="button"
            class="btn-secondary flex items-center gap-2 shadow-sm hover:shadow-md"
            :disabled="loading"
            @click="load"
          >
            <div :class="loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
            <span class="text-sm font-semibold tracking-wide">REFRESH</span>
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
            @keydown.enter="load"
          />
        </div>
        <button
          class="btn-primary flex items-center justify-center gap-2.5 px-6 py-2.5"
          :disabled="loading"
          @click="load"
        >
          <div :class="loading ? 'animate-spin' : ''" class="i-carbon-renew text-lg"></div>
          <span class="text-sm font-semibold tracking-wide">SEARCH</span>
        </button>
      </div>
    </div>

    <div class="bg-white/70 border border-gray-200 rounded-2xl shadow-sm overflow-hidden">
      <div class="p-4 border-b border-gray-100 flex items-center justify-between">
        <div class="text-sm font-semibold text-gray-700">Games</div>
        <div class="text-xs text-gray-400">{{ games.length }}</div>
      </div>

      <div v-if="error" class="p-4 text-sm text-red-600">{{ error }}</div>

      <div v-else class="overflow-auto custom-scrollbar">
        <table class="min-w-full text-sm">
          <thead class="bg-gray-50/70">
            <tr class="text-left text-gray-600">
              <th class="px-4 py-3 font-semibold">ID</th>
              <th class="px-4 py-3 font-semibold">Name</th>
              <th class="px-4 py-3 font-semibold">Region</th>
              <th class="px-4 py-3 font-semibold">Status</th>
              <th class="px-4 py-3 font-semibold">Process</th>
              <th class="px-4 py-3 font-semibold w-[220px]">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="g in games"
              :key="g.id"
              class="border-t border-gray-100 hover:bg-gray-50/60"
            >
              <td class="px-4 py-3 font-mono text-xs text-gray-600">{{ g.id }}</td>
              <td class="px-4 py-3 text-gray-800 font-semibold">{{ g.name }}</td>
              <td class="px-4 py-3 text-gray-600">{{ g.region }}</td>
              <td class="px-4 py-3 text-gray-600">{{ g.status }}</td>
              <td class="px-4 py-3 text-gray-600 truncate max-w-[360px]">{{ g.process_name }}</td>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="btn-secondary flex items-center gap-2"
                    :disabled="loadingDetailId === g.id"
                    @click="openEdit(g.id)"
                  >
                    <div
                      :class="loadingDetailId === g.id ? 'animate-spin' : ''"
                      class="i-carbon-edit"
                    ></div>
                    <span>EDIT</span>
                  </button>
                  <button
                    type="button"
                    class="btn-secondary flex items-center gap-2"
                    :disabled="deletingId === g.id"
                    @click="askDelete(g.id)"
                  >
                    <div
                      :class="deletingId === g.id ? 'animate-spin' : ''"
                      class="i-carbon-trash-can"
                    ></div>
                    <span>DELETE</span>
                  </button>
                </div>
              </td>
            </tr>

            <tr v-if="!loading && games.length === 0">
              <td colspan="6" class="px-4 py-8 text-center text-gray-400">No games</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <transition name="fade">
      <div
        v-if="showEditor"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click.self="closeEditor"
      >
        <div class="bg-white rounded-3xl p-8 w-full max-w-3xl shadow-2xl animate-slide-up">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-2xl font-bold text-gray-900">{{ editorMode === 'create' ? 'Create Game' : 'Edit Game' }}</h2>
              <p class="text-sm text-gray-500">Fields map to accelerator_games</p>
            </div>
            <button
              @click="closeEditor"
              class="text-gray-400 hover:text-gray-600 transition-colors"
              type="button"
            >
              <div class="i-carbon-close text-2xl"></div>
            </button>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">ID</label>
              <input
                v-model="form.id"
                type="text"
                :disabled="editorMode === 'edit'"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all font-mono"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Name</label>
              <input
                v-model="form.name"
                type="text"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Region</label>
              <input
                v-model="form.region"
                type="text"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Status</label>
              <input
                v-model="form.status"
                type="text"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Icon</label>
              <input
                v-model="form.icon"
                type="text"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">Ping</label>
              <input
                v-model.number="form.ping"
                type="number"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all"
              />
            </div>
          </div>

          <div class="mt-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">Process Name</label>
            <textarea
              v-model="form.process_name"
              rows="2"
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all resize-none font-mono text-xs"
              placeholder="chrome.exe,xxx.exe"
            />
          </div>

          <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">routing_rules (JSON string)</label>
              <textarea
                v-model="form.routing_rules"
                rows="8"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all resize-none font-mono text-xs"
                placeholder="[]"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">sniff_domains_excluded (JSON string)</label>
              <textarea
                v-model="form.sniff_domains_excluded"
                rows="8"
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent transition-all resize-none font-mono text-xs"
                placeholder="[]"
              />
            </div>
          </div>

          <div class="flex gap-3 mt-8">
            <button type="button" class="flex-1 btn-secondary py-3" @click="closeEditor">
              Cancel
            </button>
            <button
              type="button"
              class="flex-1 btn-success py-3 flex items-center justify-center gap-2"
              :disabled="saving"
              @click="save"
            >
              <div
                v-if="saving"
                class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"
              ></div>
              <span>{{ saving ? 'Saving...' : 'Save' }}</span>
            </button>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div
        v-if="showDeleteConfirm"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click.self="cancelDelete"
      >
        <div class="bg-white rounded-3xl p-8 w-full max-w-md shadow-2xl animate-slide-up">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-xl font-bold text-gray-900">Delete game</h2>
            <button
              @click="cancelDelete"
              class="text-gray-400 hover:text-gray-600 transition-colors"
              type="button"
            >
              <div class="i-carbon-close text-2xl"></div>
            </button>
          </div>
          <p class="text-sm text-gray-600">Are you sure to delete <span class="font-mono">{{ pendingDeleteId }}</span> ?</p>
          <div class="flex gap-3 mt-8">
            <button type="button" class="flex-1 btn-secondary py-3" @click="cancelDelete">Cancel</button>
            <button
              type="button"
              class="flex-1 btn-danger py-3 flex items-center justify-center gap-2"
              :disabled="!pendingDeleteId || deletingId === pendingDeleteId"
              @click="confirmDelete"
            >
              <div
                v-if="pendingDeleteId && deletingId === pendingDeleteId"
                class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"
              ></div>
              <span>{{ pendingDeleteId && deletingId === pendingDeleteId ? 'Deleting...' : 'Delete' }}</span>
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import * as adminApi from '@/api/admin';
import { useToastStore } from '@/stores/toast';

const router = useRouter();
const toast = useToastStore();

const keyword = ref('');
const games = ref<adminApi.GameVO[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const showEditor = ref(false);
const editorMode = ref<'create' | 'edit'>('create');
const saving = ref(false);
const loadingDetailId = ref<string | null>(null);

const showDeleteConfirm = ref(false);
const pendingDeleteId = ref<string | null>(null);
const deletingId = ref<string | null>(null);

const blankForm = (): adminApi.GameDetailVO => ({
  id: '',
  name: '',
  icon: '',
  status: 'idle',
  ping: 0,
  process_name: '',
  region: '',
  routing_rules: '[]',
  sniff_domains_excluded: '[]',
});

const form = ref<adminApi.GameDetailVO>(blankForm());

const load = async () => {
  loading.value = true;
  error.value = null;
  try {
    games.value = await adminApi.listGames({ keyword: keyword.value.trim() || undefined });
  } catch (e: any) {
    error.value = e?.message || 'Failed to load games';
  } finally {
    loading.value = false;
  }
};

const openCreate = () => {
  editorMode.value = 'create';
  form.value = blankForm();
  showEditor.value = true;
};

const openEdit = async (gameId: string) => {
  loadingDetailId.value = gameId;
  try {
    const detail = await adminApi.getGame(gameId);
    editorMode.value = 'edit';
    form.value = { ...detail };
    showEditor.value = true;
  } catch (e: any) {
    toast.error(e?.message || 'Failed to load game detail');
  } finally {
    loadingDetailId.value = null;
  }
};

const closeEditor = () => {
  showEditor.value = false;
  saving.value = false;
};

const save = async () => {
  const id = form.value.id.trim();
  const name = form.value.name.trim();

  if (!id) {
    toast.warning('ID is required');
    return;
  }
  if (!name) {
    toast.warning('Name is required');
    return;
  }

  saving.value = true;
  try {
    if (editorMode.value === 'create') {
      await adminApi.createGame({ ...form.value, id, name });
      toast.success('Created');
    } else {
      await adminApi.updateGame(id, { ...form.value, id, name });
      toast.success('Saved');
    }
    showEditor.value = false;
    await load();
  } catch (e: any) {
    toast.error(e?.message || 'Save failed');
  } finally {
    saving.value = false;
  }
};

const askDelete = (gameId: string) => {
  pendingDeleteId.value = gameId;
  showDeleteConfirm.value = true;
};

const cancelDelete = () => {
  showDeleteConfirm.value = false;
  pendingDeleteId.value = null;
};

const confirmDelete = async () => {
  if (!pendingDeleteId.value) return;
  const gameId = pendingDeleteId.value;
  deletingId.value = gameId;
  try {
    await adminApi.deleteGame(gameId);
    toast.success('Deleted');
    showDeleteConfirm.value = false;
    pendingDeleteId.value = null;
    await load();
  } catch (e: any) {
    toast.error(e?.message || 'Delete failed');
  } finally {
    deletingId.value = null;
  }
};

onMounted(load);
</script>
