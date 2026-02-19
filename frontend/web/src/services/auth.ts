import api from './api';
import type { AuthResponse } from '../types';

export const authService = {
  async register(username: string, display_name: string, email: string, password: string): Promise<AuthResponse> {
    const { data } = await api.post<AuthResponse>('/auth/register', {
      username,
      display_name,
      email,
      password,
    });
    return data;
  },

  async login(email: string, password: string): Promise<AuthResponse> {
    const { data } = await api.post<AuthResponse>('/auth/login', { email, password });
    return data;
  },

  logout() {
    localStorage.removeItem('vox_token');
    localStorage.removeItem('vox_user');
  },

  getToken(): string | null {
    return localStorage.getItem('vox_token');
  },

  isAuthenticated(): boolean {
    return !!localStorage.getItem('vox_token');
  },
};
