// Types partagés correspondant aux modèles backend

export interface User {
  id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  bio?: string;
  status: UserStatus;
}

export type UserStatus = 'online' | 'idle' | 'donotdisturb' | 'invisible' | 'offline';

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Space {
  id: string;
  name: string;
  description?: string;
  icon_url?: string;
  owner_id: string;
  is_public: boolean;
  member_count: number;
  created_at: string;
  updated_at: string;
}

export interface SpaceMember {
  space_id: string;
  user_id: string;
  role: SpaceRole;
  joined_at: string;
}

export type SpaceRole = 'owner' | 'admin' | 'moderator' | 'member';

export interface Channel {
  id: string;
  space_id: string;
  name: string;
  description?: string;
  channel_type: ChannelType;
  position: number;
  is_ephemeral: boolean;
  ephemeral_duration_hours?: number;
  created_at: string;
  updated_at: string;
}

export type ChannelType = 'text' | 'voice' | 'announcement' | 'thread';

export interface Message {
  id: string;
  channel_id: string;
  author_id: string;
  author_username: string;
  author_display_name: string;
  author_avatar_url?: string;
  content: string;
  edited: boolean;
  reply_to?: string;
  created_at: string;
  updated_at: string;
}

// WebSocket Events
export interface WsClientEvent {
  type: 'Subscribe' | 'Unsubscribe' | 'SendMessage' | 'StartTyping' | 'UpdateStatus' | 'Ping'
    | 'JoinVoice' | 'LeaveVoice' | 'UpdateVoiceState' | 'VoiceSignal';
  data?: any;
}

export interface WsServerEvent {
  type: 'Ready' | 'NewMessage' | 'MessageEdited' | 'MessageDeleted' | 'UserTyping' | 'PresenceUpdate' | 'Pong' | 'Error'
    | 'VoiceUserJoined' | 'VoiceUserLeft' | 'VoiceStateUpdate' | 'VoiceSignal';
  data?: any;
}

// ========== Phase 2 Types ==========

export interface FileInfo {
  id: string;
  filename: string;
  original_filename: string;
  mime_type: string;
  size_bytes: number;
  url?: string;
  thumbnail_url?: string;
  created_at: string;
}

export interface Role {
  id: string;
  space_id: string;
  name: string;
  color?: string;
  icon?: string;
  position: number;
  is_default: boolean;
  permissions: string[];
}

export interface VoiceParticipant {
  user_id: string;
  username: string;
  is_muted: boolean;
  is_deafened: boolean;
  is_screen_sharing: boolean;
}

export interface VoiceChannelInfo {
  channel_id: string;
  participants: VoiceParticipant[];
  participant_count: number;
}

export interface PermissionInfo {
  key: string;
  category: string;
}
