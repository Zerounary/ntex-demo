import { defineStore } from 'pinia';
import type {
  User,
  OutboundConfig,
  RoutingConfig,
  UserMapping,
  MaintenanceMode,
} from '@/api/types';
import * as adminApi from '@/api/admin';

export const useAdminStore = defineStore('admin', {
  state: () => ({
    // 用户管理
    users: [] as User[],
    usersLoading: false,
    usersError: null as string | null,

    // 上游代理管理
    outbounds: [] as OutboundConfig[],
    userMapping: {} as UserMapping,
    outboundsLoading: false,
    outboundsError: null as string | null,

    // 路由配置
    routing: null as RoutingConfig | null,
    routingLoading: false,
    routingError: null as string | null,

    // 维护模式
    maintenanceMode: null as MaintenanceMode | null,
    maintenanceLoading: false,
    maintenanceError: null as string | null,
  }),

  actions: {
    // ========== 用户管理 ==========
    async fetchUsers(nodeId: number) {
      this.usersLoading = true;
      this.usersError = null;
      try {
        this.users = await adminApi.getUsers(nodeId);
      } catch (err: any) {
        this.usersError = err.message || '获取用户列表失败';
        console.error('Failed to fetch users:', err);
      } finally {
        this.usersLoading = false;
      }
    },

    async addUser(nodeId: number, user: { uuid: string; st?: number; dt?: number }) {
      try {
        const newUser = await adminApi.addUser(nodeId, user);
        this.users.push(newUser);
        return newUser;
      } catch (err: any) {
        throw err;
      }
    },

    async updateUser(
      nodeId: number,
      userId: number,
      user: { uuid?: string; st?: number; dt?: number }
    ) {
      try {
        await adminApi.updateUser(nodeId, userId, user);
        const index = this.users.findIndex((u) => u.id === userId);
        if (index !== -1) {
          this.users[index] = { ...this.users[index], ...user };
        }
      } catch (err: any) {
        throw err;
      }
    },

    async deleteUser(nodeId: number, userId: number) {
      try {
        await adminApi.deleteUser(nodeId, userId);
        this.users = this.users.filter((u) => u.id !== userId);
      } catch (err: any) {
        throw err;
      }
    },

    // ========== 上游代理管理 ==========
    async fetchOutbounds(nodeId: number) {
      this.outboundsLoading = true;
      this.outboundsError = null;
      try {
        const data = await adminApi.getOutbounds(nodeId);
        this.outbounds = data.outbounds;
        this.userMapping = data.user_mapping;
      } catch (err: any) {
        this.outboundsError = err.message || '获取上游代理列表失败';
        console.error('Failed to fetch outbounds:', err);
      } finally {
        this.outboundsLoading = false;
      }
    },

    async addOutbound(
      nodeId: number,
      outbound: { tag: string; protocol?: string; settings: any; stream_settings?: any }
    ) {
      try {
        const newOutbound = await adminApi.addOutbound(nodeId, {
          tag: outbound.tag,
          protocol: outbound.protocol,
          settings: outbound.settings,
        });
        // 注意：后端 API 可能不支持 stream_settings，这里只保存 settings
        this.outbounds.push(newOutbound);
        return newOutbound;
      } catch (err: any) {
        throw err;
      }
    },

    async updateOutbound(
      nodeId: number,
      tag: string,
      outbound: { protocol?: string; settings?: any }
    ) {
      try {
        await adminApi.updateOutbound(nodeId, tag, outbound);
        const index = this.outbounds.findIndex((o) => o.tag === tag);
        if (index !== -1) {
          this.outbounds[index] = { ...this.outbounds[index], ...outbound };
        }
      } catch (err: any) {
        throw err;
      }
    },

    async deleteOutbound(nodeId: number, tag: string) {
      try {
        await adminApi.deleteOutbound(nodeId, tag);
        this.outbounds = this.outbounds.filter((o) => o.tag !== tag);
        // 清理相关的用户映射
        Object.keys(this.userMapping).forEach((uuid) => {
          if (this.userMapping[uuid] === tag) {
            delete this.userMapping[uuid];
          }
        });
      } catch (err: any) {
        throw err;
      }
    },

    // ========== 路由配置管理 ==========
    async fetchRouting(nodeId: number) {
      this.routingLoading = true;
      this.routingError = null;
      try {
        this.routing = await adminApi.getRouting(nodeId);
      } catch (err: any) {
        this.routingError = err.message || '获取路由配置失败';
        console.error('Failed to fetch routing:', err);
      } finally {
        this.routingLoading = false;
      }
    },

    async updateRouting(
      nodeId: number,
      routing: {
        domain_strategy?: string;
        rules?: any[];
        routing?: any;
      }
    ) {
      try {
        this.routing = await adminApi.updateRouting(nodeId, routing);
      } catch (err: any) {
        throw err;
      }
    },

    async addRoutingRule(nodeId: number, rule: any) {
      try {
        this.routing = await adminApi.addRoutingRule(nodeId, rule);
      } catch (err: any) {
        throw err;
      }
    },

    async updateRoutingRule(nodeId: number, index: number, rule: any) {
      try {
        await adminApi.updateRoutingRule(nodeId, index, rule);
        if (this.routing) {
          this.routing.rules[index] = rule;
        }
      } catch (err: any) {
        throw err;
      }
    },

    async deleteRoutingRule(nodeId: number, index: number) {
      try {
        await adminApi.deleteRoutingRule(nodeId, index);
        if (this.routing) {
          this.routing.rules.splice(index, 1);
        }
      } catch (err: any) {
        throw err;
      }
    },

    // ========== 用户映射管理 ==========
    async addMapping(nodeId: number, uuid: string, outboundTag: string) {
      try {
        await adminApi.addMapping(nodeId, { uuid, outbound_tag: outboundTag });
        this.userMapping[uuid] = outboundTag;
      } catch (err: any) {
        throw err;
      }
    },

    async updateMapping(nodeId: number, uuid: string, outboundTag: string) {
      try {
        await adminApi.updateMapping(nodeId, uuid, outboundTag);
        this.userMapping[uuid] = outboundTag;
      } catch (err: any) {
        throw err;
      }
    },

    async deleteMapping(nodeId: number, uuid: string) {
      try {
        await adminApi.deleteMapping(nodeId, uuid);
        delete this.userMapping[uuid];
      } catch (err: any) {
        throw err;
      }
    },

    // ========== 维护模式管理 ==========
    async fetchMaintenanceMode(nodeId: number) {
      this.maintenanceLoading = true;
      this.maintenanceError = null;
      try {
        this.maintenanceMode = await adminApi.getMaintenanceMode(nodeId);
      } catch (err: any) {
        this.maintenanceError = err.message || '获取维护模式失败';
        console.error('Failed to fetch maintenance mode:', err);
      } finally {
        this.maintenanceLoading = false;
      }
    },

    async setMaintenanceMode(nodeId: number, enabled: boolean) {
      try {
        this.maintenanceMode = await adminApi.setMaintenanceMode(nodeId, enabled);
      } catch (err: any) {
        throw err;
      }
    },

    // ========== 重置状态 ==========
    reset() {
      this.users = [];
      this.outbounds = [];
      this.userMapping = {};
      this.routing = null;
      this.maintenanceMode = null;
    },
  },
});

