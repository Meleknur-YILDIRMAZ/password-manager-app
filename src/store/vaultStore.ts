// store/vaultStore.ts — Vault Kayıt State (Zustand)

import { create } from 'zustand';
import type { PasswordAnalysis, VaultEntry, VaultEntryMeta } from '../types';
import {
  analyzePasswords,
  createVaultEntry,
  deleteVaultEntry,
  getVaultEntry,
  listVaultEntries,
  updateVaultEntry,
} from '../lib/tauri';

interface VaultState {
  // State
  entries: VaultEntryMeta[];
  selectedEntry: VaultEntry | null;
  selectedId: string | null;
  isLoading: boolean;
  isLoadingDetail: boolean;
  error: string | null;
  searchQuery: string;
  selectedCategory: string;
  analysis: PasswordAnalysis[];

  // Computed
  filteredEntries: () => VaultEntryMeta[];
  categories: () => string[];

  // Actions
  loadEntries: () => Promise<void>;
  selectEntry: (id: string | null) => Promise<void>;
  createEntry: (entry: Omit<VaultEntry, 'id' | 'created_at' | 'updated_at'>) => Promise<string>;
  updateEntry: (entry: VaultEntry) => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  setSearchQuery: (q: string) => void;
  setSelectedCategory: (cat: string) => void;
  runAnalysis: () => Promise<void>;
  clearError: () => void;
  clear: () => void;
}

const EMPTY_ENTRY: Omit<VaultEntry, 'id' | 'created_at' | 'updated_at'> = {
  name: '',
  username: '',
  password: '',
  url: '',
  notes: '',
  category: 'Genel',
  tags: '',
  totp_secret: '',
  is_favorite: false,
};

export const useVaultStore = create<VaultState>((set, get) => ({
  entries: [],
  selectedEntry: null,
  selectedId: null,
  isLoading: false,
  isLoadingDetail: false,
  error: null,
  searchQuery: '',
  selectedCategory: 'Tümü',
  analysis: [],

  filteredEntries: () => {
    const { entries, searchQuery, selectedCategory } = get();
    let filtered = entries;

    // Kategori filtresi
    if (selectedCategory !== 'Tümü' && selectedCategory !== 'Favoriler') {
      filtered = filtered.filter((e) => e.category === selectedCategory);
    }
    if (selectedCategory === 'Favoriler') {
      filtered = filtered.filter((e) => e.is_favorite);
    }

    // Arama filtresi
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (e) =>
          e.name.toLowerCase().includes(q) ||
          e.username?.toLowerCase().includes(q) ||
          e.url?.toLowerCase().includes(q) ||
          e.category.toLowerCase().includes(q) ||
          e.tags.toLowerCase().includes(q)
      );
    }

    return filtered;
  },

  categories: () => {
    const { entries } = get();
    const cats = new Set(entries.map((e) => e.category));
    return Array.from(cats).sort();
  },

  loadEntries: async () => {
    set({ isLoading: true, error: null });
    try {
      const entries = await listVaultEntries();
      set({ entries });
    } catch (err) {
      set({ error: String(err) });
    } finally {
      set({ isLoading: false });
    }
  },

  selectEntry: async (id: string | null) => {
    if (!id) {
      set({ selectedEntry: null, selectedId: null });
      return;
    }

    set({ selectedId: id, isLoadingDetail: true });
    try {
      const entry = await getVaultEntry(id);
      set({ selectedEntry: entry });
    } catch (err) {
      set({ error: String(err), selectedEntry: null, selectedId: null });
    } finally {
      set({ isLoadingDetail: false });
    }
  },

  createEntry: async (entryData) => {
    set({ isLoading: true, error: null });
    try {
      const now = Math.floor(Date.now() / 1000);
      const entry: VaultEntry = {
        ...entryData,
        id: '',
        created_at: now,
        updated_at: now,
      };
      const id = await createVaultEntry(entry);
      await get().loadEntries();
      return id;
    } catch (err) {
      const msg = String(err);
      set({ error: msg });
      throw new Error(msg);
    } finally {
      set({ isLoading: false });
    }
  },

  updateEntry: async (entry) => {
    set({ isLoading: true, error: null });
    try {
      await updateVaultEntry(entry);
      await get().loadEntries();
      // Seçili kaydı da güncelle
      if (get().selectedId === entry.id) {
        set({ selectedEntry: entry });
      }
    } catch (err) {
      const msg = String(err);
      set({ error: msg });
      throw new Error(msg);
    } finally {
      set({ isLoading: false });
    }
  },

  deleteEntry: async (id) => {
    set({ isLoading: true, error: null });
    try {
      await deleteVaultEntry(id);
      if (get().selectedId === id) {
        set({ selectedEntry: null, selectedId: null });
      }
      await get().loadEntries();
    } catch (err) {
      const msg = String(err);
      set({ error: msg });
      throw new Error(msg);
    } finally {
      set({ isLoading: false });
    }
  },

  setSearchQuery: (q) => set({ searchQuery: q }),
  setSelectedCategory: (cat) => set({ selectedCategory: cat }),

  runAnalysis: async () => {
    try {
      const analysis = await analyzePasswords();
      set({ analysis });
    } catch {
      // Analiz hatası sessizce yoksay
    }
  },

  clearError: () => set({ error: null }),

  clear: () =>
    set({
      entries: [],
      selectedEntry: null,
      selectedId: null,
      searchQuery: '',
      selectedCategory: 'Tümü',
      analysis: [],
    }),
}));

export { EMPTY_ENTRY };
