// types/index.ts — Uygulama genelinde kullanılan TypeScript tipleri
// Rust tarafındaki struct'larla birebir eşleşmelidir.

export interface VaultEntry {
  id: string;
  name: string;
  username?: string;
  password?: string;
  url?: string;
  notes?: string;
  category: string;
  tags: string;
  totp_secret?: string;
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
}

export interface VaultEntryMeta {
  id: string;
  name: string;
  username?: string;
  url?: string;
  category: string;
  tags: string;
  is_favorite: boolean;
  has_totp: boolean;
  updated_at: number;
}

export interface VaultSettings {
  auto_lock_minutes: number;
  hibp_enabled: boolean;
  theme: 'dark' | 'light';
  clipboard_clear_seconds: number;
}

export interface GeneratorOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  symbols: boolean;
  exclude_ambiguous: boolean;
}

export interface GeneratedPassword {
  password: string;
}

export interface TotpCode {
  code: string;
  remaining_seconds: number;
  period: number;
}

export interface HibpResult {
  is_pwned: boolean;
  pwn_count?: number;
}

export interface PasswordAnalysis {
  id: string;
  name: string;
  is_weak: boolean;
  is_duplicate: boolean;
  duplicate_with: string[];
}

export type AppPage =
  | 'setup'
  | 'unlock'
  | 'dashboard';

export type ThemeMode = 'dark' | 'light';

// Şifre gücü seviyeleri (zxcvbn 0-4)
export const STRENGTH_LABELS = ['Çok Zayıf', 'Zayıf', 'Orta', 'Güçlü', 'Çok Güçlü'] as const;
export const STRENGTH_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#10b981'] as const;
