import { create } from 'zustand';
import type { User } from '../types';
import { authService } from '../services/auth';
import { wsService } from '../services/websocket';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (email: string, password: string) => Promise<void>;
  register: (username: string, displayName: string, email: string, password: string) => Promise<void>;
  logout: () => void;
  loadFromStorage: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,

  login: async (email, password) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authService.login(email, password);
      localStorage.setItem('vox_token', response.token);
      localStorage.setItem('vox_user', JSON.stringify(response.user));
      wsService.connect(response.token);
      set({
        user: response.user,
        token: response.token,
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (err: any) {
      set({
        error: err.response?.data?.error?.message || 'Erreur de connexion',
        isLoading: false,
      });
      throw err;
    }
  },

  register: async (username, displayName, email, password) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authService.register(username, displayName, email, password);
      localStorage.setItem('vox_token', response.token);
      localStorage.setItem('vox_user', JSON.stringify(response.user));
      wsService.connect(response.token);
      set({
        user: response.user,
        token: response.token,
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (err: any) {
      set({
        error: err.response?.data?.error?.message || "Erreur lors de l'inscription",
        isLoading: false,
      });
      throw err;
    }
  },

  logout: () => {
    authService.logout();
    wsService.disconnect();
    set({ user: null, token: null, isAuthenticated: false });
  },

  loadFromStorage: () => {
    const token = localStorage.getItem('vox_token');
    const userStr = localStorage.getItem('vox_user');
    if (token && userStr) {
      try {
        const user = JSON.parse(userStr);
        wsService.connect(token);
        set({ user, token, isAuthenticated: true });
      } catch {
        authService.logout();
      }
    }
  },
}));
