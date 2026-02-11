<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg font-bold text-gray-900">Firewall Rules</h3>
        <p class="text-xs text-gray-500">Manage Windows Firewall port rules on this node (requires admin privileges on node).</p>
      </div>
      <button type="button" class="btn-secondary" :disabled="loading" @click="reload">
        <span v-if="loading">Loading...</span>
        <span v-else>Refresh</span>
      </button>
    </div>

    <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5 space-y-4">
      <div class="grid grid-cols-1 md:grid-cols-6 gap-4">
        <div class="md:col-span-2">
          <label class="block text-xs font-semibold text-gray-500 mb-1">Rule Name</label>
          <input v-model="form.name" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm" placeholder="e.g. api-10086" />
        </div>
        <div class="md:col-span-2">
          <label class="block text-xs font-semibold text-gray-500 mb-1">Display Name</label>
          <input v-model="form.display_name" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm" placeholder="optional" />
        </div>
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">Protocol</label>
          <select v-model="form.proto" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm">
            <option value="tcp">TCP</option>
            <option value="udp">UDP</option>
          </select>
        </div>
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">Port</label>
          <input v-model.number="form.port" type="number" min="1" max="65535" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm" />
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-6 gap-4">
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">Direction</label>
          <select v-model="form.dir" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm">
            <option value="in">Inbound</option>
            <option value="out">Outbound</option>
          </select>
        </div>
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">Action</label>
          <select v-model="form.action" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm">
            <option value="allow">Allow</option>
            <option value="block">Block</option>
          </select>
        </div>
        <div class="md:col-span-4">
          <label class="block text-xs font-semibold text-gray-500 mb-1">Remote Addresses</label>
          <input v-model="remoteInput" class="w-full px-3 py-2 rounded-lg border border-gray-200 bg-gray-50 text-sm" placeholder="comma separated, e.g. 1.2.3.4,10.0.0.0/8" />
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 pt-2">
        <button type="button" class="btn-primary" :disabled="saving" @click="upsert">
          <span v-if="saving">Saving...</span>
          <span v-else>Upsert Rule</span>
        </button>
      </div>
    </div>

    <div class="overflow-x-auto rounded-xl border border-gray-100 bg-white shadow-sm">
      <table class="min-w-full text-sm">
        <thead class="bg-gray-50/60">
          <tr class="text-left">
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Display Name</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Enabled</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Direction</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Action</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Protocol</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Local Port</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Remote</th>
            <th class="px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Ops</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in rules" :key="r.display_name" class="border-t border-gray-100 hover:bg-gray-50/40 transition-colors">
            <td class="px-4 py-3 font-mono text-gray-800 whitespace-nowrap">{{ r.display_name }}</td>
            <td class="px-4 py-3 text-gray-600">{{ r.enabled }}</td>
            <td class="px-4 py-3 text-gray-600">{{ r.direction }}</td>
            <td class="px-4 py-3 text-gray-600">{{ r.action }}</td>
            <td class="px-4 py-3 text-gray-600">{{ r.protocol }}</td>
            <td class="px-4 py-3 font-mono text-gray-700">{{ r.local_port }}</td>
            <td class="px-4 py-3 font-mono text-gray-500 max-w-[20rem] truncate" :title="r.remote_address">{{ r.remote_address }}</td>
            <td class="px-4 py-3">
              <button
                type="button"
                class="inline-flex items-center rounded-md px-2.5 py-1.5 text-xs font-semibold text-red-700 hover:text-red-800 hover:bg-red-50 disabled:opacity-60 disabled:cursor-not-allowed"
                :disabled="isDeleting(r.display_name)"
                @click="removeRule(r.display_name)"
              >
                <span v-if="isDeleting(r.display_name)">Deleting...</span>
                <span v-else>Delete</span>
              </button>
            </td>
          </tr>
          <tr v-if="!rules.length">
            <td class="px-4 py-6 text-center text-gray-400" colspan="8">No rules</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useNodeStore } from '@/stores/node';
import { useToastStore } from '@/stores/toast';
import * as adminApi from '@/api/admin';
import type { FirewallRule } from '@/api/types';

const nodeStore = useNodeStore();
const toast = useToastStore();

const nodeId = computed(() => nodeStore.currentNodeId);

const loading = ref(false);
const saving = ref(false);
const rules = ref<FirewallRule[]>([]);
const deleting = ref<Set<string>>(new Set());

const remoteInput = ref('');
const form = ref({
  name: '',
  display_name: '',
  proto: 'tcp',
  port: 0,
  action: 'allow',
  dir: 'in',
});

const reload = async () => {
  const id = nodeId.value;
  if (!id) return;
  if (loading.value) return;
  loading.value = true;
  try {
    const resp = await adminApi.firewallListRules(id, {});
    rules.value = resp.rules || [];
  } catch (err: any) {
    toast.error(err?.message || 'Failed to load firewall rules');
  } finally {
    loading.value = false;
  }
};

const upsert = async () => {
  const id = nodeId.value;
  if (!id) return;
  if (saving.value) return;

  saving.value = true;
  try {
    const remote = remoteInput.value
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    await adminApi.firewallUpsertPortRule(id, {
      name: form.value.name,
      display_name: form.value.display_name || null,
      proto: form.value.proto,
      port: form.value.port,
      action: form.value.action,
      dir: form.value.dir,
      remote,
    });

    toast.success('Firewall rule updated');
    await reload();
  } catch (err: any) {
    toast.error(err?.message || 'Failed to upsert firewall rule');
  } finally {
    saving.value = false;
  }
};

const isDeleting = (displayName: string) => {
  return deleting.value.has(displayName);
};

const removeRule = async (displayName: string) => {
  const id = nodeId.value;
  if (!id) return;
  if (deleting.value.has(displayName)) return;
  deleting.value.add(displayName);
  try {
    await adminApi.firewallDeleteRule(id, { display_name: displayName });
    toast.success('Firewall rule deleted');
    await reload();
  } catch (err: any) {
    toast.error(err?.message || 'Failed to delete firewall rule');
  } finally {
    deleting.value.delete(displayName);
  }
};

onMounted(() => {
  reload();
});
</script>
