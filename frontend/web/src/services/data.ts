import api from './api';
import type { Space, Channel, Message, SpaceMember, FileInfo, Role, VoiceParticipant, VoiceChannelInfo, PermissionInfo } from '../types';

export const spaceService = {
  async list(): Promise<Space[]> {
    const { data } = await api.get<Space[]>('/spaces');
    return data;
  },
  async get(id: string): Promise<Space> {
    const { data } = await api.get<Space>(`/spaces/${id}`);
    return data;
  },
  async create(name: string, description?: string, is_public?: boolean): Promise<Space> {
    const { data } = await api.post<Space>('/spaces', { name, description, is_public });
    return data;
  },
  async update(id: string, updates: Partial<Space>): Promise<Space> {
    const { data } = await api.put<Space>(`/spaces/${id}`, updates);
    return data;
  },
  async delete(id: string): Promise<void> {
    await api.delete(`/spaces/${id}`);
  },
  async join(id: string): Promise<SpaceMember> {
    const { data } = await api.post<SpaceMember>(`/spaces/${id}/join`);
    return data;
  },
  async leave(id: string): Promise<void> {
    await api.post(`/spaces/${id}/leave`);
  },
  async members(id: string): Promise<SpaceMember[]> {
    const { data } = await api.get<SpaceMember[]>(`/spaces/${id}/members`);
    return data;
  },
};

export const channelService = {
  async list(spaceId: string): Promise<Channel[]> {
    const { data } = await api.get<Channel[]>(`/channels/space/${spaceId}`);
    return data;
  },
  async get(id: string): Promise<Channel> {
    const { data } = await api.get<Channel>(`/channels/${id}`);
    return data;
  },
  async create(spaceId: string, name: string, description?: string, channel_type?: string): Promise<Channel> {
    const { data } = await api.post<Channel>(`/channels/space/${spaceId}`, {
      name, description, channel_type,
    });
    return data;
  },
  async update(id: string, updates: Partial<Channel>): Promise<Channel> {
    const { data } = await api.put<Channel>(`/channels/${id}`, updates);
    return data;
  },
  async delete(id: string): Promise<void> {
    await api.delete(`/channels/${id}`);
  },
};

export const messageService = {
  async list(channelId: string, before?: string, limit?: number): Promise<Message[]> {
    const params = new URLSearchParams();
    if (before) params.set('before', before);
    if (limit) params.set('limit', limit.toString());
    const { data } = await api.get<Message[]>(`/messages/channel/${channelId}?${params}`);
    return data;
  },
  async create(channelId: string, content: string, reply_to?: string): Promise<Message> {
    const { data } = await api.post<Message>(`/messages/channel/${channelId}`, {
      content, reply_to,
    });
    return data;
  },
  async update(id: string, content: string): Promise<Message> {
    const { data } = await api.put<Message>(`/messages/${id}`, { content });
    return data;
  },
  async delete(id: string): Promise<void> {
    await api.delete(`/messages/${id}`);
  },
};

// ========== Fichiers ==========

export const fileService = {
  async upload(channelId: string, file: File): Promise<FileInfo> {
    const formData = new FormData();
    formData.append('file', file);
    const { data } = await api.post<FileInfo>(`/files/upload/${channelId}`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return data;
  },
  async listByChannel(channelId: string, limit?: number, offset?: number): Promise<FileInfo[]> {
    const params = new URLSearchParams();
    if (limit) params.set('limit', limit.toString());
    if (offset) params.set('offset', offset.toString());
    const { data } = await api.get<FileInfo[]>(`/files/channel/${channelId}?${params}`);
    return data;
  },
  async get(fileId: string): Promise<FileInfo> {
    const { data } = await api.get<FileInfo>(`/files/${fileId}`);
    return data;
  },
  async delete(fileId: string): Promise<void> {
    await api.delete(`/files/${fileId}`);
  },
};

// ========== Rôles ==========

export const roleService = {
  async list(spaceId: string): Promise<Role[]> {
    const { data } = await api.get<Role[]>(`/spaces/${spaceId}/roles`);
    return data;
  },
  async create(spaceId: string, name: string, permissions: string[], color?: string): Promise<Role> {
    const { data } = await api.post<Role>(`/spaces/${spaceId}/roles`, { name, permissions, color });
    return data;
  },
  async get(roleId: string): Promise<Role> {
    const { data } = await api.get<Role>(`/roles/${roleId}`);
    return data;
  },
  async update(roleId: string, updates: Partial<Role>): Promise<Role> {
    const { data } = await api.put<Role>(`/roles/${roleId}`, updates);
    return data;
  },
  async delete(roleId: string): Promise<void> {
    await api.delete(`/roles/${roleId}`);
  },
  async assignToMember(spaceId: string, userId: string, roleId: string): Promise<void> {
    await api.post(`/spaces/${spaceId}/members/${userId}/roles/${roleId}`);
  },
  async revokeFromMember(spaceId: string, userId: string, roleId: string): Promise<void> {
    await api.delete(`/spaces/${spaceId}/members/${userId}/roles/${roleId}`);
  },
  async getMemberRoles(spaceId: string, userId: string): Promise<Role[]> {
    const { data } = await api.get<Role[]>(`/spaces/${spaceId}/members/${userId}/roles`);
    return data;
  },
  async listPermissions(): Promise<PermissionInfo[]> {
    const { data } = await api.get<PermissionInfo[]>('/permissions');
    return data;
  },
};

// ========== Voice ==========

export const voiceService = {
  async getParticipants(channelId: string): Promise<VoiceParticipant[]> {
    const { data } = await api.get<VoiceParticipant[]>(`/voice/channels/${channelId}/participants`);
    return data;
  },
  async getChannelInfo(channelId: string): Promise<VoiceChannelInfo> {
    const { data } = await api.get<VoiceChannelInfo>(`/voice/channels/${channelId}/info`);
    return data;
  },
};
