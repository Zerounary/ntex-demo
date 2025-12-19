<template>
  <div class="topology-canvas relative w-full h-full bg-slate-50/50 border border-gray-200 rounded-2xl overflow-hidden shadow-inner">
    <!-- Controls -->
    <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
      <div class="flex flex-col bg-white rounded-xl shadow-lg border border-gray-100 overflow-hidden">
        <button
          @click="zoomIn"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors border-b border-gray-100"
          title="Zoom In"
        >
          <div class="i-carbon-zoom-in text-xl"></div>
        </button>
        <button
          @click="zoomOut"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors border-b border-gray-100"
          title="Zoom Out"
        >
          <div class="i-carbon-zoom-out text-xl"></div>
        </button>
        <button
          @click="resetView"
          class="p-2 hover:bg-gray-50 text-gray-600 transition-colors"
          title="Reset View"
        >
          <div class="i-carbon-center-circle text-xl"></div>
        </button>
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
    >
      <!-- Background Grid -->
      <defs>
        <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
          <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#e2e8f0" stroke-width="0.5"/>
        </pattern>
        <marker
          id="arrowhead"
          markerWidth="10"
          markerHeight="7"
          refX="28"
          refY="3.5"
          orient="auto"
        >
          <polygon points="0 0, 10 3.5, 0 7" fill="#cbd5e1" />
        </marker>
        
        <!-- Glow Filters -->
        <filter id="glow-primary" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
          <feMerge>
            <feMergeNode in="coloredBlur"/>
            <feMergeNode in="SourceGraphic"/>
          </feMerge>
        </filter>
      </defs>
      
      <rect width="100%" height="100%" fill="url(#grid)" />

      <g :transform="`translate(${panX}, ${panY}) scale(${zoom})`">
        <!-- Connections -->
        <g class="connections">
          <line
            v-for="(conn, index) in connections"
            :key="`conn-${index}`"
            :x1="conn.x1"
            :y1="conn.y1"
            :x2="conn.x2"
            :y2="conn.y2"
            stroke="#cbd5e1"
            stroke-width="2"
            stroke-linecap="round"
            marker-end="url(#arrowhead)"
            class="transition-all duration-300"
          />
        </g>

        <!-- Outbounds (Rectangles) -->
        <g class="outbounds">
          <g
            v-for="outbound in outbounds"
            :key="`outbound-${outbound.id}`"
            :transform="`translate(${outbound.x}, ${outbound.y})`"
            class="cursor-pointer transition-all duration-300"
            :class="{ 'opacity-100': !selectedItem || selectedItem.data === outbound.data, 'opacity-40': selectedItem && selectedItem.data !== outbound.data }"
            @click.stop="selectOutbound(outbound)"
          >
            <rect
              :x="-outbound.width / 2"
              :y="-outbound.height / 2"
              :width="outbound.width"
              :height="outbound.height"
              :fill="outbound.color"
              rx="12"
              class="transition-all duration-300 shadow-sm"
              :stroke="selectedOutbound?.id === outbound.id ? '#f59e0b' : 'transparent'"
              :stroke-width="selectedOutbound?.id === outbound.id ? 3 : 0"
              filter="url(#glow-primary)"
            />
            <foreignObject
              :x="-outbound.width / 2"
              :y="-outbound.height / 2"
              :width="outbound.width"
              :height="outbound.height"
            >
              <div class="w-full h-full flex items-center justify-center">
                <span class="text-xs font-bold text-white truncate px-2">{{ outbound.label }}</span>
              </div>
            </foreignObject>
          </g>
        </g>

        <!-- Nodes (Center) -->
        <g class="nodes">
          <g
            v-for="node in nodes"
            :key="`node-${node.id}`"
            :transform="`translate(${node.x}, ${node.y})`"
            class="cursor-move transition-all duration-300"
            @mousedown.stop="startDrag(node, $event)"
            @click.stop="selectNode(node)"
          >
            <circle
              :r="node.radius"
              :fill="node.color"
              class="transition-all duration-300"
              :stroke="selectedNode?.id === node.id ? '#818cf8' : 'white'"
              :stroke-width="selectedNode?.id === node.id ? 4 : 4"
              filter="url(#glow-primary)"
            />
            <foreignObject
              :x="-node.radius"
              :y="-node.radius"
              :width="node.radius * 2"
              :height="node.radius * 2"
            >
              <div class="w-full h-full flex flex-col items-center justify-center text-white">
                <div class="i-carbon-cloud-satellite text-2xl mb-1"></div>
                <span class="text-[10px] font-bold opacity-90">{{ node.label }}</span>
              </div>
            </foreignObject>
          </g>
        </g>

        <!-- Users (Circles) -->
        <g class="users">
          <g
            v-for="user in users"
            :key="`user-${user.id}`"
            :transform="`translate(${user.x}, ${user.y})`"
            class="cursor-pointer transition-all duration-300"
            :class="{ 'opacity-100': !selectedItem || selectedItem.data === user.data, 'opacity-40': selectedItem && selectedItem.data !== user.data }"
            @click.stop="selectUser(user)"
          >
            <circle
              :r="user.radius"
              :fill="user.color"
              class="transition-all duration-300"
              :stroke="selectedUser?.id === user.id ? '#34d399' : 'white'"
              :stroke-width="selectedUser?.id === user.id ? 3 : 2"
              filter="url(#glow-primary)"
            />
            <foreignObject
              :x="-user.radius"
              :y="-user.radius"
              :width="user.radius * 2"
              :height="user.radius * 2"
            >
              <div class="w-full h-full flex items-center justify-center">
                <div class="i-carbon-user text-white text-sm"></div>
              </div>
            </foreignObject>
          </g>
        </g>
      </g>
    </svg>

    <!-- Detail Panel -->
    <transition name="slide-fade">
      <div
        v-if="selectedItem"
        class="absolute top-4 right-4 w-72 bg-white/90 backdrop-blur-md border border-gray-200 rounded-2xl shadow-xl z-20 overflow-hidden animate-slide-up"
      >
        <div class="p-4 border-b border-gray-100 flex justify-between items-center bg-gray-50/50">
          <h3 class="font-bold text-gray-800 flex items-center gap-2">
            <div 
              class="w-2 h-2 rounded-full"
              :class="{
                'bg-blue-500': selectedItem.type === 'node',
                'bg-green-500': selectedItem.type === 'user',
                'bg-orange-500': selectedItem.type === 'outbound'
              }"
            ></div>
            {{ selectedItem.type.charAt(0).toUpperCase() + selectedItem.type.slice(1) }} Details
          </h3>
          <button
            @click="selectedItem = null"
            class="text-gray-400 hover:text-gray-600 transition-colors p-1 rounded-lg hover:bg-gray-100"
          >
            <div class="i-carbon-close text-lg"></div>
          </button>
        </div>
        <div class="p-4 max-h-80 overflow-y-auto custom-scrollbar">
          <div class="space-y-3">
            <template v-if="selectedItem.type === 'user'">
              <div class="space-y-1">
                <span class="text-xs text-gray-400 uppercase tracking-wider font-semibold">UUID</span>
                <p class="text-sm font-mono text-gray-700 bg-gray-50 p-2 rounded-lg break-all border border-gray-100">
                  {{ selectedItem.data.uuid }}
                </p>
              </div>
              <div class="grid grid-cols-2 gap-3">
                <div class="bg-gray-50 p-2 rounded-lg border border-gray-100">
                  <span class="text-xs text-gray-400 block mb-1">Speed Limit</span>
                  <span class="text-sm font-bold text-blue-600">{{ selectedItem.data.st || '∞' }} Mbps</span>
                </div>
                <div class="bg-gray-50 p-2 rounded-lg border border-gray-100">
                  <span class="text-xs text-gray-400 block mb-1">Devices</span>
                  <span class="text-sm font-bold text-purple-600">{{ selectedItem.data.dt || '∞' }}</span>
                </div>
              </div>
            </template>

            <template v-else-if="selectedItem.type === 'outbound'">
              <div class="space-y-1">
                <span class="text-xs text-gray-400 uppercase tracking-wider font-semibold">Tag</span>
                <p class="text-sm font-bold text-gray-800">{{ selectedItem.data.tag }}</p>
              </div>
              <div class="space-y-1">
                <span class="text-xs text-gray-400 uppercase tracking-wider font-semibold">Protocol</span>
                <span class="px-2 py-1 rounded-md bg-orange-50 text-orange-600 text-xs font-bold border border-orange-100">
                  {{ selectedItem.data.protocol }}
                </span>
              </div>
              <div class="space-y-1">
                <span class="text-xs text-gray-400 uppercase tracking-wider font-semibold">Config</span>
                <pre class="text-[10px] text-gray-600 font-mono bg-gray-50 p-2 rounded-lg overflow-x-auto border border-gray-100">{{ JSON.stringify(selectedItem.data.settings, null, 2) }}</pre>
              </div>
            </template>

            <template v-else>
              <pre class="text-xs text-gray-600 font-mono bg-gray-50 p-2 rounded-lg overflow-x-auto border border-gray-100">{{ JSON.stringify(selectedItem.data, null, 2) }}</pre>
            </template>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useAdminStore } from '@/stores/admin';
import { useNodeStore } from '@/stores/node';
import type { User, OutboundConfig } from '@/api/types';

interface CanvasNode {
  id: string;
  x: number;
  y: number;
  radius: number;
  color: string;
  label: string;
  data: any;
}

interface CanvasUser {
  id: string;
  x: number;
  y: number;
  radius: number;
  color: string;
  data: User;
}

interface CanvasOutbound {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  label: string;
  data: OutboundConfig;
}

interface Connection {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

const adminStore = useAdminStore();
const nodeStore = useNodeStore();

const svgRef = ref<SVGElement | null>(null);
const zoom = ref(1);
const panX = ref(0);
const panY = ref(0);
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const draggingNode = ref<CanvasNode | null>(null);

const selectedNode = ref<CanvasNode | null>(null);
const selectedUser = ref<CanvasUser | null>(null);
const selectedOutbound = ref<CanvasOutbound | null>(null);

const selectedItem = computed({
  get: () => {
    if (selectedNode.value) return { type: 'node', data: selectedNode.value.data };
    if (selectedUser.value) return { type: 'user', data: selectedUser.value.data };
    if (selectedOutbound.value) return { type: 'outbound', data: selectedOutbound.value.data };
    return null;
  },
  set: (val) => {
    if (!val) {
      selectedNode.value = null;
      selectedUser.value = null;
      selectedOutbound.value = null;
    }
  }
});

const nodes = ref<CanvasNode[]>([]);
const users = ref<CanvasUser[]>([]);
const outbounds = ref<CanvasOutbound[]>([]);
const connections = ref<Connection[]>([]);

const layoutNodes = () => {
  if (!nodeStore.currentNodeId) return;

  const rect = svgRef.value?.getBoundingClientRect();
  const centerX = rect ? rect.width / 2 : 400;
  const centerY = rect ? rect.height / 2 : 300;
  
  // Center Node
  const nodeRadius = 45;
  nodes.value = [
    {
      id: `node-${nodeStore.currentNodeId}`,
      x: centerX,
      y: centerY,
      radius: nodeRadius,
      color: '#6366f1', // Primary-500
      label: `Node ${nodeStore.currentNodeId}`,
      data: nodeStore.currentNode,
    },
  ];

  // Users (Orbiting)
  const userCount = adminStore.users.length;
  const userRadius = 18;
  const userDistance = 160;
  users.value = adminStore.users.map((user, index) => {
    const angle = (index / userCount) * Math.PI * 2 - Math.PI / 2;
    return {
      id: `user-${user.id}`,
      x: centerX + Math.cos(angle) * userDistance,
      y: centerY + Math.sin(angle) * userDistance,
      radius: userRadius,
      color: '#10b981', // Emerald-500
      data: user,
    };
  });

  // Outbounds (Bottom Row)
  const outboundWidth = 100;
  const outboundHeight = 44;
  const outboundSpacing = 120;
  const outboundStartY = centerY + 180;
  const totalOutboundWidth = (adminStore.outbounds.length - 1) * outboundSpacing;
  const outboundStartX = centerX - totalOutboundWidth / 2;
  
  outbounds.value = adminStore.outbounds.map((outbound, index) => ({
    id: `outbound-${outbound.tag}`,
    x: outboundStartX + index * outboundSpacing,
    y: outboundStartY,
    width: outboundWidth,
    height: outboundHeight,
    color: '#f59e0b', // Amber-500
    label: outbound.tag,
    data: outbound,
  }));

  updateConnections();
};

const updateConnections = () => {
  connections.value = [];

  if (nodes.value.length === 0) return;

  const node = nodes.value[0];

  // User to Node
  users.value.forEach((user) => {
    // Calculate intersection point on node circle to stop line at edge
    const angle = Math.atan2(user.y - node.y, user.x - node.x);
    const nodeEdgeX = node.x + Math.cos(angle) * (node.radius + 5);
    const nodeEdgeY = node.y + Math.sin(angle) * (node.radius + 5);
    
    // Calculate intersection point on user circle
    const userEdgeX = user.x - Math.cos(angle) * (user.radius + 5);
    const userEdgeY = user.y - Math.sin(angle) * (user.radius + 5);

    connections.value.push({
      x1: userEdgeX,
      y1: userEdgeY,
      x2: nodeEdgeX,
      y2: nodeEdgeY,
    });
  });

  // Node to Outbound
  outbounds.value.forEach((outbound) => {
    const angle = Math.atan2(outbound.y - node.y, outbound.x - node.x);
    const nodeEdgeX = node.x + Math.cos(angle) * (node.radius + 5);
    const nodeEdgeY = node.y + Math.sin(angle) * (node.radius + 5);
    
    connections.value.push({
      x1: nodeEdgeX,
      y1: nodeEdgeY,
      x2: outbound.x,
      y2: outbound.y - outbound.height / 2 - 5,
    });
  });
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
  if (event.button === 0) {
    isDragging.value = true;
    dragStart.value = { x: event.clientX - panX.value, y: event.clientY - panY.value };
  }
};

const handleMouseMove = (event: MouseEvent) => {
  if (isDragging.value && !draggingNode.value) {
    panX.value = event.clientX - dragStart.value.x;
    panY.value = event.clientY - dragStart.value.y;
  } else if (draggingNode.value) {
    const rect = svgRef.value?.getBoundingClientRect();
    if (rect) {
      draggingNode.value.x = (event.clientX - rect.left - panX.value) / zoom.value;
      draggingNode.value.y = (event.clientY - rect.top - panY.value) / zoom.value;
      updateConnections();
    }
  }
};

const handleMouseUp = () => {
  isDragging.value = false;
  draggingNode.value = null;
};

const handleWheel = (event: WheelEvent) => {
  event.preventDefault();
  const delta = event.deltaY > 0 ? 0.9 : 1.1;
  zoom.value = Math.max(0.5, Math.min(3, zoom.value * delta));
};

const startDrag = (node: CanvasNode, event: MouseEvent) => {
  event.stopPropagation();
  draggingNode.value = node;
  isDragging.value = true;
};

const selectNode = (node: CanvasNode) => {
  selectedNode.value = node;
  selectedUser.value = null;
  selectedOutbound.value = null;
};

const selectUser = (user: CanvasUser) => {
  selectedUser.value = user;
  selectedNode.value = null;
  selectedOutbound.value = null;
};

const selectOutbound = (outbound: CanvasOutbound) => {
  selectedOutbound.value = outbound;
  selectedNode.value = null;
  selectedUser.value = null;
};

watch(
  () => nodeStore.currentNodeId,
  () => {
    if (nodeStore.currentNodeId) {
      loadData();
    }
  },
  { immediate: true }
);

watch(
  () => [adminStore.users, adminStore.outbounds],
  () => {
    layoutNodes();
  },
  { deep: true }
);

async function loadData() {
  if (!nodeStore.currentNodeId) return;

  try {
    await Promise.all([
      adminStore.fetchUsers(nodeStore.currentNodeId),
      adminStore.fetchOutbounds(nodeStore.currentNodeId),
    ]);
    await nextTick();
    layoutNodes();
  } catch (err) {
    console.error('Failed to load data:', err);
  }
}

onMounted(() => {
  // Initial layout delay to ensure SVG size is correct
  setTimeout(layoutNodes, 100);
  window.addEventListener('resize', layoutNodes);
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

.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s ease-out;
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
</style>

