import { defineStore } from 'pinia';
import { storage } from '@/utils';
import { v4 as uuidv4 } from 'uuid';
import type { NodeInfo } from '@/api/types';

export type ChainProtocol = 'transparent' | 'vmess';

export interface FlowChain {
  id: string;
  name: string;
  uuid: string;
  protocol: ChainProtocol;
  createdAt: string;
  updatedAt: string;
}

export interface FlowCanvasNode {
  id: string;
  node_id: number;
  label: string;
  node_type: string;
  region?: string | null;
  description?: string | null;
  x: number;
  y: number;
}

export interface FlowEdge {
  id: string;
  chain_id: string;
  from: string;
  to: string;
}

export type FlowSelection =
  | { type: 'node'; id: string }
  | { type: 'edge'; id: string }
  | null;

function nowIso() {
  return new Date().toISOString();
}

export const useFlowStore = defineStore('flow', {
  state: () => ({
    chains: [] as FlowChain[],
    activeChainId: null as string | null,

    nodes: [] as FlowCanvasNode[],
    edges: [] as FlowEdge[],

    showAllChains: true,
    connectMode: false,
    pendingFromNodeId: null as string | null,

    selection: null as FlowSelection,
  }),

  getters: {
    activeChain(state) {
      if (!state.activeChainId) return null;
      return state.chains.find((c) => c.id === state.activeChainId) || null;
    },

    edgesForDisplay(state) {
      if (state.showAllChains) return state.edges;
      if (!state.activeChainId) return [];
      return state.edges.filter((e) => e.chain_id === state.activeChainId);
    },

    selectedNode(state) {
      if (!state.selection || state.selection.type !== 'node') return null;
      return state.nodes.find((n) => n.id === state.selection!.id) || null;
    },

    selectedEdge(state) {
      if (!state.selection || state.selection.type !== 'edge') return null;
      return state.edges.find((e) => e.id === state.selection!.id) || null;
    },
  },

  actions: {
    ensureInitialized() {
      if (this.chains.length > 0) {
        if (!this.activeChainId) this.activeChainId = this.chains[0].id;
        return;
      }
      const id = uuidv4();
      const ts = nowIso();
      this.chains = [
        {
          id,
          name: 'Chain 1',
          uuid: uuidv4(),
          protocol: 'vmess',
          createdAt: ts,
          updatedAt: ts,
        },
      ];
      this.activeChainId = id;
    },

    setActiveChain(chainId: string) {
      this.activeChainId = chainId;
      this.pendingFromNodeId = null;
      this.selection = null;
    },

    toggleShowAllChains() {
      this.showAllChains = !this.showAllChains;
      this.pendingFromNodeId = null;
      this.selection = null;
    },

    setConnectMode(on: boolean) {
      this.connectMode = on;
      if (!on) this.pendingFromNodeId = null;
    },

    createChain(name?: string) {
      const id = uuidv4();
      const ts = nowIso();
      const chain: FlowChain = {
        id,
        name: name?.trim() || `Chain ${this.chains.length + 1}`,
        uuid: uuidv4(),
        protocol: 'vmess',
        createdAt: ts,
        updatedAt: ts,
      };
      this.chains.push(chain);
      this.setActiveChain(id);
      return chain;
    },

    renameChain(chainId: string, name: string) {
      const c = this.chains.find((x) => x.id === chainId);
      if (!c) return;
      c.name = name.trim() || c.name;
      c.updatedAt = nowIso();
    },

    setChainProtocol(chainId: string, protocol: ChainProtocol) {
      const c = this.chains.find((x) => x.id === chainId);
      if (!c) return;
      c.protocol = protocol;
      c.updatedAt = nowIso();
    },

    regenerateChainUuid(chainId: string) {
      const c = this.chains.find((x) => x.id === chainId);
      if (!c) return null;
      c.uuid = uuidv4();
      c.updatedAt = nowIso();
      return c.uuid;
    },

    deleteChain(chainId: string) {
      const idx = this.chains.findIndex((c) => c.id === chainId);
      if (idx === -1) return;

      this.edges = this.edges.filter((e) => e.chain_id !== chainId);
      this.chains.splice(idx, 1);

      if (this.activeChainId === chainId) {
        this.activeChainId = this.chains[0]?.id || null;
      }
      this.pendingFromNodeId = null;
      this.selection = null;

      if (this.chains.length === 0) this.ensureInitialized();
    },

    clearAll(confirmToken?: string) {
      if (confirmToken !== 'CONFIRM') return;
      this.nodes = [];
      this.edges = [];
      this.chains = [];
      this.activeChainId = null;
      this.showAllChains = true;
      this.connectMode = false;
      this.pendingFromNodeId = null;
      this.selection = null;
      this.ensureInitialized();
    },

    addNodeFromNodeInfo(node: NodeInfo, x: number, y: number) {
      const existing = this.nodes.find((n) => n.node_id === node.node_id);
      if (existing) {
        existing.x = x;
        existing.y = y;
        return existing;
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
      this.nodes.push(canvasNode);
      return canvasNode;
    },

    updateNodePosition(nodeId: string, x: number, y: number) {
      const n = this.nodes.find((nn) => nn.id === nodeId);
      if (!n) return;
      n.x = x;
      n.y = y;
    },

    removeNode(nodeId: string) {
      this.edges = this.edges.filter((e) => e.from !== nodeId && e.to !== nodeId);
      this.nodes = this.nodes.filter((n) => n.id !== nodeId);
      if (this.pendingFromNodeId === nodeId) this.pendingFromNodeId = null;
      if (this.selection?.type === 'node' && this.selection.id === nodeId) this.selection = null;
    },

    selectNode(nodeId: string | null) {
      if (!nodeId) {
        this.selection = null;
        return;
      }
      this.selection = { type: 'node', id: nodeId };
    },

    selectEdge(edgeId: string | null) {
      if (!edgeId) {
        this.selection = null;
        return;
      }
      this.selection = { type: 'edge', id: edgeId };
    },

    clearSelection() {
      this.selection = null;
      this.pendingFromNodeId = null;
    },

    beginOrCompleteConnect(nodeId: string) {
      if (!this.connectMode) {
        this.selectNode(nodeId);
        return;
      }
      if (!this.activeChainId) return;

      if (!this.pendingFromNodeId) {
        this.pendingFromNodeId = nodeId;
        return;
      }

      const from = this.pendingFromNodeId;
      const to = nodeId;
      this.pendingFromNodeId = null;
      if (from === to) return;

      const exists = this.edges.some(
        (e) => e.chain_id === this.activeChainId && e.from === from && e.to === to
      );
      if (exists) return;

      const edge: FlowEdge = {
        id: uuidv4(),
        chain_id: this.activeChainId,
        from,
        to,
      };
      this.edges.push(edge);
      this.selection = { type: 'edge', id: edge.id };
    },

    removeEdge(edgeId: string) {
      this.edges = this.edges.filter((e) => e.id !== edgeId);
      if (this.selection?.type === 'edge' && this.selection.id === edgeId) this.selection = null;
    },

    exportJson() {
      return JSON.stringify(
        {
          version: 1,
          chains: this.chains,
          activeChainId: this.activeChainId,
          nodes: this.nodes,
          edges: this.edges,
        },
        null,
        2
      );
    },

    importJson(raw: string) {
      const parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== 'object') throw new Error('Invalid JSON');
      if (!Array.isArray(parsed.chains) || !Array.isArray(parsed.nodes) || !Array.isArray(parsed.edges)) {
        throw new Error('Missing fields');
      }

      this.chains = parsed.chains;
      this.nodes = parsed.nodes;
      this.edges = parsed.edges;
      this.activeChainId = parsed.activeChainId || this.chains[0]?.id || null;
      this.pendingFromNodeId = null;
      this.selection = null;
      this.ensureInitialized();
    },
  },

  persist: {
    enabled: true,
    strategies: [
      {
        key: 'flow',
        storage,
      },
    ],
  },
});
