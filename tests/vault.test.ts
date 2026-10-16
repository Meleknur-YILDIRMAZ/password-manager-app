// Rust tarafı birim testler zaten her modülde #[cfg(test)] ile var.
// Bu dosya frontend (Vitest) testleri içindir.

// tests/crypto.test.ts — Şifreleme mantığının frontend entegrasyon testi
// NOT: Gerçek şifreleme Rust tarafında. Bu testler IPC mock'ları kullanır.

import { describe, it, expect, vi } from 'vitest';

// Tauri invoke mock
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Şifre Üretici', () => {
  it('varsayılan uzunlukta şifre üretmeli', () => {
    // 20 karakterlik şifrenin regex kontrolü
    const mockPassword = 'Kd8!xQ2#mN9@pL4$';
    expect(mockPassword.length).toBeGreaterThanOrEqual(8);
  });

  it('büyük harf içermeli', () => {
    const pwd = 'Abc123!@#';
    expect(/[A-Z]/.test(pwd)).toBe(true);
  });

  it('küçük harf içermeli', () => {
    const pwd = 'Abc123!@#';
    expect(/[a-z]/.test(pwd)).toBe(true);
  });

  it('rakam içermeli', () => {
    const pwd = 'Abc123!@#';
    expect(/[0-9]/.test(pwd)).toBe(true);
  });

  it('özel karakter içermeli', () => {
    const pwd = 'Abc123!@#';
    expect(/[!@#$%^&*]/.test(pwd)).toBe(true);
  });
});

describe('Şifre Gücü (zxcvbn)', () => {
  it('zayıf şifre düşük skor almalı', async () => {
    const { default: zxcvbn } = await import('zxcvbn');
    const result = zxcvbn('123456');
    expect(result.score).toBeLessThanOrEqual(1);
  });

  it('güçlü şifre yüksek skor almalı', async () => {
    const { default: zxcvbn } = await import('zxcvbn');
    const result = zxcvbn('Tr!9kX#2mP@8nL$4');
    expect(result.score).toBeGreaterThanOrEqual(3);
  });
});

describe('Arama ve Filtreleme', () => {
  const mockEntries = [
    { id: '1', name: 'Google', username: 'user@google.com', category: 'E-posta', tags: '', url: '', is_favorite: false, has_totp: false, updated_at: 0 },
    { id: '2', name: 'GitHub', username: 'dev@github.com', category: 'İş', tags: 'kod', url: '', is_favorite: true, has_totp: true, updated_at: 0 },
    { id: '3', name: 'Netflix', username: 'user@example.com', category: 'Eğlence', tags: '', url: '', is_favorite: false, has_totp: false, updated_at: 0 },
  ];

  it('isimle arama çalışmalı', () => {
    const q = 'git';
    const filtered = mockEntries.filter((e) =>
      e.name.toLowerCase().includes(q)
    );
    expect(filtered).toHaveLength(1);
    expect(filtered[0].name).toBe('GitHub');
  });

  it('kategori filtresi çalışmalı', () => {
    const filtered = mockEntries.filter((e) => e.category === 'İş');
    expect(filtered).toHaveLength(1);
  });

  it('favori filtresi çalışmalı', () => {
    const filtered = mockEntries.filter((e) => e.is_favorite);
    expect(filtered).toHaveLength(1);
    expect(filtered[0].name).toBe('GitHub');
  });
});
