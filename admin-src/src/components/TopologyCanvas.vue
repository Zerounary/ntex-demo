<template>
  <div class="topology-canvas relative w-full h-full bg-gray-50 border border-gray-200 rounded-lg overflow-hidden">
    <div class="absolute top-4 left-4 z-10 flex space-x-2">
      <button
        @click="zoomIn"
        class="px-3 py-1 bg-white border border-gray-300 rounded shadow hover:bg-gray-50"
      >
        放大
      </button>
      <button
        @click="zoomOut"
        class="px-3 py-1 bg-white border border-gray-300 rounded shadow hover:bg-gray-50"
      >
        缩小
      </button>
      <button
        @click="resetView"
        class="px-3 py-1 bg-white border border-gray-300 rounded shadow hover:bg-gray-50"
      >
        重置
      </button>
    </div>
    <svg
      ref="svgRef"
      class="w-full h-full"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @wheel="handleWheel"
    >
      <g :transform="`translate(${panX}, ${panY}) scale(${zoom})`">
        <!-- 连接线 -->
        <g class="connections">
          <line
            v-for="(conn, index) in connections"
            :key="`conn-${index}`"
            :x1="conn.x1"
            :y1="conn.y1"
            :x2="conn.x2"
            :y2="conn.y2"
            stroke="#94a3b8"
            stroke-width="2"
            marker-end="url(#arrowhead)"
          />
        </g>

        <!-- 节点 -->
        <g class="nodes">
          <g
            v-for="node in nodes"
            :key="`node-${node.id}`"
            :transform="`translate(${node.x}, ${node.y})`"
            class="cursor-move"
            @mousedown.stop="startDrag(node, $event)"
            @click="selectNode(node)"
          >
            <circle
              :r="node.radius"
              :fill="node.color"
              :stroke="selectedNode?.id === node.id ? '#3b82f6' : '#64748b'"
              :stroke-width="selectedNode?.id === node.id ? 3 : 2"
              class="hover:opacity-80"
            />
            <text
              x="0"
              y="0"
              text-anchor="middle"
              dominant-baseline="middle"
              class="text-sm font-semibold fill-white pointer-events-none"
            >
              {{ node.label }}
            </text>
          </g>
        </g>

        <!-- 用户 -->
        <g class="users">
          <circle
            v-for="user in users"
            :key="`user-${user.id}`"
            :cx="user.x"
            :cy="user.y"
            :r="user.radius"
            :fill="user.color"
            :stroke="selectedUser?.id === user.id ? '#3b82f6' : '#64748b'"
            :stroke-width="selectedUser?.id === user.id ? 2 : 1"
            class="cursor-pointer hover:opacity-80"
            @click="selectUser(user)"
          />
        </g>

        <!-- 上游代理 -->
        <g class="outbounds">
          <rect
            v-for="outbound in outbounds"
            :key="`outbound-${outbound.id}`"
            :x="outbound.x - outbound.width / 2"
            :y="outbound.y - outbound.height / 2"
            :width="outbound.width"
            :height="outbound.height"
            :fill="outbound.color"
            :stroke="selectedOutbound?.id === outbound.id ? '#3b82f6' : '#64748b'"
            :stroke-width="selectedOutbound?.id === outbound.id ? 2 : 1"
            rx="4"
            class="cursor-pointer hover:opacity-80"
            @click="selectOutbound(outbound)"
          />
          <text
            v-for="outbound in outbounds"
            :key="`outbound-text-${outbound.id}`"
            :x="outbound.x"
            :y="outbound.y"
            text-anchor="middle"
            dominant-baseline="middle"
            class="text-xs font-medium fill-white pointer-events-none"
          >
            {{ outbound.label }}
          </text>
        </g>
      </g>

      <!-- 箭头标记 -->
      <defs>
        <marker
          id="arrowhead"
          markerWidth="10"
          markerHeight="10"
          refX="9"
          refY="3"
          orient="auto"
        >
          <polygon points="0 0, 10 3, 0 6" fill="#94a3b8" />
        </marker>
      </defs>
    </svg>

    <!-- 详情面板 -->
    <div
      v-if="selectedItem"
      class="absolute top-4 right-4 bg-white border border-gray-200 rounded-lg shadow-lg p-4 max-w-xs z-20"
    >
      <div class="flex justify-between items-center mb-2">
        <h3 class="font-semibold text-gray-800">详情</h3>
        <button
          @click="selectedItem = null"
          class="text-gray-500 hover:text-gray-700"
        >
          ✕
        </button>
      </div>
      <pre class="text-xs overflow-auto max-h-64">{{ JSON.stringify(selectedItem.data, null, 2) }}</pre>
    </div>
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

const selectedItem = computed(() => {
  if (selectedNode.value) return { type: 'node', data: selectedNode.value.data };
  if (selectedUser.value) return { type: 'user', data: selectedUser.value.data };
  if (selectedOutbound.value) return { type: 'outbound', data: selectedOutbound.value.data };
  return null;
});

const nodes = ref<CanvasNode[]>([]);
const users = ref<CanvasUser[]>([]);
const outbounds = ref<CanvasOutbound[]>([]);
const connections = ref<Connection[]>([]);

const layoutNodes = () => {
  if (!nodeStore.currentNodeId) return;

  // 中心节点
  const centerX = 400;
  const centerY = 300;
  const nodeRadius = 40;

  nodes.value = [
    {
      id: `node-${nodeStore.currentNodeId}`,
      x: centerX,
      y: centerY,
      radius: nodeRadius,
      color: '#3b82f6',
      label: `节点 ${nodeStore.currentNodeId}`,
      data: nodeStore.currentNode,
    },
  ];

  // 用户布局（围绕节点）
  const userCount = adminStore.users.length;
  const userRadius = 15;
  const userDistance = 120;
  users.value = adminStore.users.map((user, index) => {
    const angle = (index / userCount) * Math.PI * 2;
    return {
      id: `user-${user.id}`,
      x: centerX + Math.cos(angle) * userDistance,
      y: centerY + Math.sin(angle) * userDistance,
      radius: userRadius,
      color: '#10b981',
      data: user,
    };
  });

  // 上游代理布局（节点下方）
  const outboundWidth = 80;
  const outboundHeight = 40;
  const outboundSpacing = 100;
  const outboundStartX = centerX - ((adminStore.outbounds.length - 1) * outboundSpacing) / 2;
  outbounds.value = adminStore.outbounds.map((outbound, index) => ({
    id: `outbound-${outbound.tag}`,
    x: outboundStartX + index * outboundSpacing,
    y: centerY + 150,
    width: outboundWidth,
    height: outboundHeight,
    color: '#f59e0b',
    label: outbound.tag,
    data: outbound,
  }));

  // 生成连接线
  updateConnections();
};

const updateConnections = () => {
  connections.value = [];

  if (nodes.value.length === 0) return;

  const node = nodes.value[0];

  // 用户到节点的连接
  users.value.forEach((user) => {
    connections.value.push({
      x1: user.x,
      y1: user.y,
      x2: node.x,
      y2: node.y,
    });
  });

  // 节点到上游代理的连接
  outbounds.value.forEach((outbound) => {
    connections.value.push({
      x1: node.x,
      y1: node.y,
      x2: outbound.x,
      y2: outbound.y,
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

const loadData = async () => {
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
};
</script>

<style scoped></style>

