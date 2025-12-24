<template>
  <div
    class="graph-canvas relative w-full h-full bg-slate-50/50 border border-gray-200 rounded-2xl overflow-hidden shadow-inner"
    @dragover.prevent
    @dragenter.prevent="onDragEnter"
    @dragleave.prevent="onDragLeave"
    @drop="handleDrop"
  >
    <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
      <div class="flex flex-col bg-white rounded-xl shadow-lg border border-gray-100 overflow-hidden">
        <button
          type="button"
          @click="zoomIn"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors border-b border-gray-100"
          title="Zoom In"
        >
          <div class="i-carbon-zoom-in text-xl"></div>
        </button>
        <button
          type="button"
          @click="zoomOut"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors border-b border-gray-100"
          title="Zoom Out"
        >
          <div class="i-carbon-zoom-out text-xl"></div>
        </button>
        <button
          type="button"
          @click="resetView"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors"
          title="Reset View"
        >
          <div class="i-carbon-center-circle text-xl"></div>
        </button>
      </div>

      <div
        v-if="connectMode"
        class="px-3 py-2 bg-white/90 backdrop-blur-md rounded-xl border border-gray-200 shadow-lg text-xs text-gray-600"
      >
        <div class="flex items-center gap-2">
          <div class="i-carbon-direction-right-02 text-lg text-primary-600"></div>
          <div class="min-w-0">
            <div class="font-semibold text-gray-800">Connect Mode</div>
            <div class="text-[11px] text-gray-500">
              {{ pendingFromNodeId ? 'Select target node' : 'Select source node' }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <svg
      ref="svgRef"
      class="w-full h-full cursor-grab active:cursor-grabbing"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
      @wheel="handleWheel"
      @click="handleBackgroundClick"
    >
      <defs>
        <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
          <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#e2e8f0" stroke-width="0.5" />
        </pattern>

        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="28" refY="3.5" orient="auto">
          <polygon points="0 0, 10 3.5, 0 7" fill="#cbd5e1" />
        </marker>

        <filter id="glow-primary" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="4" result="coloredBlur" />
          <feMerge>
            <feMergeNode in="coloredBlur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <rect width="100%" height="100%" fill="url(#grid)" />

      <g :transform="`translate(${panX}, ${panY}) scale(${zoom})`">
        <g class="edges">
          <line
            v-for="edge in edgeLines"
            :key="edge.id"
            :x1="edge.x1"
            :y1="edge.y1"
            :x2="edge.x2"
            :y2="edge.y2"
            :stroke="edge.stroke"
            :stroke-width="edge.strokeWidth"
            stroke-linecap="round"
            marker-end="url(#arrowhead)"
            class="transition-all duration-200"
            @click.stop="emit('edge-click', edge.id)"
          />
        </g>

        <g class="nodes">
          <g
            v-for="node in nodes"
            :key="node.id"
            :transform="`translate(${node.x}, ${node.y})`"
            class="cursor-move transition-all duration-200"
            @mousedown.stop="startNodeDrag(node.id, $event)"
            @click.stop="emit('node-click', node.id)"
          >
            <circle
              :r="nodeRadius"
              :fill="nodeFill(node)"
              :stroke="nodeStroke(node.id)"
              :stroke-width="nodeStrokeWidth(node.id)"
              filter="url(#glow-primary)"
              class="transition-all duration-200"
            />
            <foreignObject :x="-nodeRadius" :y="-nodeRadius" :width="nodeRadius * 2" :height="nodeRadius * 2">
              <div class="w-full h-full flex flex-col items-center justify-center text-white">
                <div class="i-carbon-cloud-satellite text-2xl mb-1"></div>
                <span class="text-[10px] font-bold opacity-90 truncate max-w-[70px]">{{ node.label }}</span>
              </div>
            </foreignObject>

            <circle
              v-if="connectMode && pendingFromNodeId === node.id"
              :r="nodeRadius + 10"
              fill="transparent"
              stroke="#6366f1"
              stroke-width="3"
              stroke-dasharray="6 4"
              class="opacity-70"
            />
          </g>
        </g>
      </g>
    </svg>

    <div
      v-if="draggingOver"
      class="absolute inset-0 z-20 bg-primary-50/40 border-2 border-dashed border-primary-300 rounded-2xl flex items-center justify-center"
    >
      <div class="bg-white/90 backdrop-blur-md px-4 py-2 rounded-xl shadow-lg border border-gray-100 text-sm font-semibold text-primary-700">
        Drop to add node
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { FlowCanvasNode, FlowEdge } from '@/stores/flow';

interface Props {
  nodes: FlowCanvasNode[];
  edges: FlowEdge[];
  connectMode: boolean;
  pendingFromNodeId: string | null;
  selectedNodeId: string | null;
  selectedEdgeId: string | null;
}

interface Emits {
  (e: 'drop-node', payload: { nodeId: number; x: number; y: number }): void;
  (e: 'node-click', nodeId: string): void;
  (e: 'edge-click', edgeId: string): void;
  (e: 'node-drag', payload: { id: string; x: number; y: number }): void;
  (e: 'background-click'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const svgRef = ref<SVGElement | null>(null);

const zoom = ref(1);
const panX = ref(0);
const panY = ref(0);

const isPanning = ref(false);
const panStart = ref({ x: 0, y: 0 });

const draggingNodeId = ref<string | null>(null);

const draggingOver = ref(false);

const nodeRadius = 42;

const colorForChain = (chainId: string) => {
  let h = 0;
  for (let i = 0; i < chainId.length; i++) {
    h = (h * 31 + chainId.charCodeAt(i)) >>> 0;
  }
  const hue = h % 360;
  return `hsl(${hue} 70% 55%)`;
};

const nodeMap = computed(() => {
  const m = new Map<string, FlowCanvasNode>();
  for (const n of props.nodes) m.set(n.id, n);
  return m;
});

const edgeLines = computed(() => {
  const lines: Array<{
    id: string;
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    stroke: string;
    strokeWidth: number;
  }> = [];

  for (const e of props.edges) {
    const from = nodeMap.value.get(e.from);
    const to = nodeMap.value.get(e.to);
    if (!from || !to) continue;

    const angle = Math.atan2(to.y - from.y, to.x - from.x);
    const x1 = from.x + Math.cos(angle) * (nodeRadius + 6);
    const y1 = from.y + Math.sin(angle) * (nodeRadius + 6);
    const x2 = to.x - Math.cos(angle) * (nodeRadius + 10);
    const y2 = to.y - Math.sin(angle) * (nodeRadius + 10);

    const selected = props.selectedEdgeId === e.id;
    const chainStroke = colorForChain(e.chain_id);

    lines.push({
      id: e.id,
      x1,
      y1,
      x2,
      y2,
      stroke: selected ? '#6366f1' : chainStroke,
      strokeWidth: selected ? 4 : 2.5,
    });
  }

  return lines;
});

const nodeFill = (node: FlowCanvasNode) => {
  const t = (node.node_type || '').toLowerCase();
  if (t.includes('transparent')) return '#10b981';
  if (t.includes('vmess') || t.includes('vless') || t.includes('auth')) return '#6366f1';
  return '#64748b';
};

const nodeStroke = (id: string) => {
  if (props.selectedNodeId === id) return '#818cf8';
  return 'white';
};

const nodeStrokeWidth = (id: string) => {
  if (props.selectedNodeId === id) return 5;
  return 4;
};

const zoomIn = () => {
  zoom.value = Math.min(zoom.value * 1.2, 3);
};

const zoomOut = () => {
  zoom.value = Math.max(zoom.value / 1.2, 0.5);
};

const resetView = () => {
  zoom.value = 1;
  panX.value = 0;
  panY.value = 0;
};

const handleMouseDown = (event: MouseEvent) => {
  if (event.button !== 0) return;
  isPanning.value = true;
  panStart.value = { x: event.clientX - panX.value, y: event.clientY - panY.value };
};

const handleMouseMove = (event: MouseEvent) => {
  if (draggingNodeId.value) {
    const rect = svgRef.value?.getBoundingClientRect();
    if (!rect) return;

    const x = (event.clientX - rect.left - panX.value) / zoom.value;
    const y = (event.clientY - rect.top - panY.value) / zoom.value;

    emit('node-drag', { id: draggingNodeId.value, x, y });
    return;
  }

  if (!isPanning.value) return;
  panX.value = event.clientX - panStart.value.x;
  panY.value = event.clientY - panStart.value.y;
};

const handleMouseUp = () => {
  isPanning.value = false;
  draggingNodeId.value = null;
};

const handleWheel = (event: WheelEvent) => {
  event.preventDefault();
  const delta = event.deltaY > 0 ? 0.9 : 1.1;
  zoom.value = Math.max(0.5, Math.min(3, zoom.value * delta));
};

const startNodeDrag = (id: string, event: MouseEvent) => {
  event.stopPropagation();
  draggingNodeId.value = id;
  isPanning.value = false;
};

const handleBackgroundClick = () => {
  emit('background-click');
};

const handleDrop = (event: DragEvent) => {
  draggingOver.value = false;

  const raw = event.dataTransfer?.getData('application/x-node-id') || event.dataTransfer?.getData('text/plain');
  if (!raw) return;

  const nodeId = parseInt(raw, 10);
  if (Number.isNaN(nodeId)) return;

  const rect = svgRef.value?.getBoundingClientRect();
  if (!rect) return;

  const x = (event.clientX - rect.left - panX.value) / zoom.value;
  const y = (event.clientY - rect.top - panY.value) / zoom.value;

  emit('drop-node', { nodeId, x, y });
};

watch(
  () => props.nodes.length,
  () => {
    if (!svgRef.value) return;
    if (props.nodes.length > 0) return;
    resetView();
  }
);

const onDragEnter = () => {
  draggingOver.value = true;
};

const onDragLeave = () => {
  draggingOver.value = false;
};
</script>

<style scoped>
</style>
