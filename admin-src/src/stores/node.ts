import { defineStore } from 'pinia';
import type { NodeInfo } from '@/api/types';
import * as adminApi from '@/api/admin';

export const useNodeStore = defineStore('node', {
  state: () => ({
    nodes: [] as NodeInfo[],
    currentNodeId: null as number | null,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    currentNode: (state) => {
      if (!state.currentNodeId) return null;
      return state.nodes.find((n) => n.node_id === state.currentNodeId) || null;
    },

    sortedNodes: (state) => {
      return [...state.nodes].sort((a, b) => a.sort - b.sort);
    },
  },

  actions: {
    async fetchNodes(isOnline?: boolean) {
      this.loading = true;
      this.error = null;
      try {
        this.nodes = await adminApi.listNodes({ is_online: isOnline });
      } catch (err: any) {
        this.error = err.message || '获取节点列表失败';
        console.error('Failed to fetch nodes:', err);
      } finally {
        this.loading = false;
      }
    },

    setCurrentNode(nodeId: number | null) {
      this.currentNodeId = nodeId;
    },

    updateNode(nodeId: number, updates: Partial<NodeInfo>) {
      const index = this.nodes.findIndex((n) => n.node_id === nodeId);
      if (index !== -1) {
        this.nodes[index] = { ...this.nodes[index], ...updates };
      }
    },
  },
});

