import { create } from 'zustand';
import type { Space, Channel, Message } from '../types';
import { spaceService, channelService, messageService } from '../services/data';

interface AppState {
  // Espaces
  spaces: Space[];
  currentSpace: Space | null;
  loadSpaces: () => Promise<void>;
  selectSpace: (space: Space) => Promise<void>;
  createSpace: (name: string, description?: string) => Promise<void>;

  // Canaux
  channels: Channel[];
  currentChannel: Channel | null;
  loadChannels: (spaceId: string) => Promise<void>;
  selectChannel: (channel: Channel) => void;
  createChannel: (spaceId: string, name: string, description?: string, type?: string) => Promise<void>;

  // Messages
  messages: Message[];
  loadMessages: (channelId: string) => Promise<void>;
  addMessage: (message: Message) => void;
  sendMessage: (channelId: string, content: string) => Promise<void>;

  // UI
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Espaces
  spaces: [],
  currentSpace: null,

  loadSpaces: async () => {
    try {
      const spaces = await spaceService.list();
      set({ spaces });
    } catch (err) {
      console.error('Erreur chargement espaces:', err);
    }
  },

  selectSpace: async (space) => {
    set({ currentSpace: space, currentChannel: null, messages: [] });
    await get().loadChannels(space.id);
  },

  createSpace: async (name, description) => {
    const space = await spaceService.create(name, description);
    set((state) => ({ spaces: [...state.spaces, space] }));
    // Sélectionner automatiquement le nouvel espace
    await get().selectSpace(space);
  },

  // Canaux
  channels: [],
  currentChannel: null,

  loadChannels: async (spaceId) => {
    try {
      const channels = await channelService.list(spaceId);
      set({ channels });
      // Sélectionner le premier canal texte automatiquement
      const firstText = channels.find((c) => c.channel_type.toLowerCase() === 'text');
      if (firstText) {
        get().selectChannel(firstText);
      }
    } catch (err) {
      console.error('Erreur chargement canaux:', err);
    }
  },

  selectChannel: (channel) => {
    set({ currentChannel: channel, messages: [] });
    get().loadMessages(channel.id);
  },

  createChannel: async (spaceId, name, description, type) => {
    const channel = await channelService.create(spaceId, name, description, type);
    set((state) => ({ channels: [...state.channels, channel] }));
    // Sélectionner automatiquement le nouveau canal
    get().selectChannel(channel);
  },

  // Messages
  messages: [],

  loadMessages: async (channelId) => {
    try {
      const messages = await messageService.list(channelId);
      set({ messages: messages.reverse() }); // Les messages arrivent DESC, on les inverse
    } catch (err) {
      console.error('Erreur chargement messages:', err);
    }
  },

  addMessage: (message) => {
    set((state) => ({ messages: [...state.messages, message] }));
  },

  sendMessage: async (channelId, content) => {
    try {
      const message = await messageService.create(channelId, content);
      set((state) => ({ messages: [...state.messages, message] }));
    } catch (err) {
      console.error('Erreur envoi message:', err);
    }
  },

  // UI
  isSidebarOpen: true,
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),
}));
