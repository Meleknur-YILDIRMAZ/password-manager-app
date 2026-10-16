// store/authStore.ts — Kimlik Doğrulama State (Zustand)

import { create } from 'zustand';
import type { VaultSettings } from '../types';
import { getVaultSettings, isLocked, isSetup, lockVault, setupVault, unlockVault } from '../lib/tauri';

interface AuthState {
  // State
  isSetup: boolean;
  isLocked: boolean;
  isLoading: boolean;
  error: string | null;
  settings: VaultSettings | null;

  // Actions
  checkSetup: () => Promise<void>;
  setup: (masterPassword: string) => Promise<void>;
  unlock: (masterPassword: string) => Promise<void>;
  lock: () => Promise<void>;
  loadSettings: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  isSetup: false,
  isLocked: true,
  isLoading: false,
  error: null,
  settings: null,

  checkSetup: async () => {
    set({ isLoading: true, error: null });
    try {
      const setup = await isSetup();
      const locked = setup ? await isLocked() : true;
      set({ isSetup: setup, isLocked: locked });

      if (!locked) {
        await get().loadSettings();
      }
    } catch (err) {
      set({ error: String(err) });
    } finally {
      set({ isLoading: false });
    }
  },

  setup: async (masterPassword: string) => {
    set({ isLoading: true, error: null });
    try {
      await setupVault(masterPassword);
      await get().loadSettings();
      set({ isSetup: true, isLocked: false });
    } catch (err) {
      set({ error: String(err) });
      throw err;
    } finally {
      set({ isLoading: false });
    }
  },

  unlock: async (masterPassword: string) => {
    set({ isLoading: true, error: null });
    try {
      await unlockVault(masterPassword);
      await get().loadSettings();
      set({ isLocked: false });
    } catch (err) {
      // Genel hata mesajı — ayrıntı gösterme (oracle önlemi)
      const errMsg = 'Yanlış master şifre. Lütfen tekrar deneyin.';
      set({ error: errMsg });
      throw new Error(errMsg);
    } finally {
      set({ isLoading: false });
    }
  },

  lock: async () => {
    try {
      await lockVault();
    } catch {
      // Kilitleme hatası sessizce yoksay
    } finally {
      set({ isLocked: true });
    }
  },

  loadSettings: async () => {
    try {
      const settings = await getVaultSettings();
      set({ settings });

      // Temayı uygula
      document.documentElement.setAttribute('data-theme', settings.theme);
    } catch {
      // Ayarlar yüklenemezse varsayılanları kullan
    }
  },

  clearError: () => set({ error: null }),
}));
