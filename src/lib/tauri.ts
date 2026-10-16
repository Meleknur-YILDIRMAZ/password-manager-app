// lib/tauri.ts — Tauri IPC komutlarını sarmalayan tip güvenli API katmanı
// Frontend hiçbir zaman doğrudan invoke çağırmamalı, bu fonksiyonları kullanmalı.

import { invoke } from '@tauri-apps/api/core';
import type {
  GeneratedPassword,
  GeneratorOptions,
  HibpResult,
  PasswordAnalysis,
  TotpCode,
  VaultEntry,
  VaultEntryMeta,
  VaultSettings,
} from '../types';

// ─── Auth ────────────────────────────────────────────────────────────────────

export const isSetup = () => invoke<boolean>('is_setup');

export const setupVault = (masterPassword: string) =>
  invoke<void>('setup_vault', { masterPassword });

export const unlockVault = (masterPassword: string) =>
  invoke<void>('unlock_vault', { masterPassword });

export const lockVault = () => invoke<void>('lock_vault');

export const isLocked = () => invoke<boolean>('is_locked');

export const getVaultSettings = () => invoke<VaultSettings>('get_vault_settings');

export const updateVaultSettings = (settings: VaultSettings) =>
  invoke<void>('update_vault_settings', { settings });

// ─── Vault CRUD ──────────────────────────────────────────────────────────────

export const createVaultEntry = (entry: VaultEntry) =>
  invoke<string>('create_vault_entry', { entry });

export const updateVaultEntry = (entry: VaultEntry) =>
  invoke<void>('update_vault_entry', { entry });

export const deleteVaultEntry = (id: string) =>
  invoke<void>('delete_vault_entry', { id });

export const listVaultEntries = () =>
  invoke<VaultEntryMeta[]>('list_vault_entries');

export const getVaultEntry = (id: string) =>
  invoke<VaultEntry>('get_vault_entry', { id });

export const analyzePasswords = () =>
  invoke<PasswordAnalysis[]>('analyze_passwords');

// ─── Generator ───────────────────────────────────────────────────────────────

export const generatePassword = (options: GeneratorOptions) =>
  invoke<GeneratedPassword>('generate_password', { options });

export const getHibpPrefix = (password: string) =>
  invoke<string>('get_hibp_prefix', { password });

// ─── TOTP ────────────────────────────────────────────────────────────────────

export const generateTotp = (secretBase32: string) =>
  invoke<TotpCode>('generate_totp', { secretBase32 });

export const parseOtpauthUri = (uri: string) =>
  invoke<string>('parse_otpauth_uri', { uri });

// ─── HIBP ────────────────────────────────────────────────────────────────────

export const checkHibp = (password: string) =>
  invoke<HibpResult>('check_hibp', { password });
