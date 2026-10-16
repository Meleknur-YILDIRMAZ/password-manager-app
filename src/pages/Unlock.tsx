// pages/Unlock.tsx — Kilit Açma Ekranı

import { useState, useRef, useEffect } from 'react';
import { Eye, EyeOff, Lock, Unlock as UnlockIcon, AlertCircle } from 'lucide-react';
import { useAuthStore } from '../store/authStore';

export function Unlock() {
  const { unlock, isLoading, error, clearError } = useAuthStore();
  const [password, setPassword] = useState('');
  const [showPass, setShowPass] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    clearError();
    try {
      await unlock(password);
    } catch {
      setPassword('');
      inputRef.current?.focus();
    }
  };

  return (
    <div className="auth-container">
      <div className="auth-card animate-slide-up">
        {/* Logo */}
        <div className="auth-logo">
          <div
            className="auth-logo-icon animate-pulse-glow"
            style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-default)' }}
          >
            <Lock size={28} color="var(--accent-secondary)" />
          </div>
          <h1 className="auth-title">Vault Kilitli</h1>
          <p className="auth-subtitle">
            Devam etmek için master şifrenizi girin
          </p>
        </div>

        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          {/* Şifre alanı */}
          <div className="form-group">
            <label className="form-label">Master Şifre</label>
            <div className="input-wrapper">
              <input
                id="unlock-password"
                ref={inputRef}
                type={showPass ? 'text' : 'password'}
                className="input input-mono"
                placeholder="••••••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete="current-password"
                required
              />
              <div className="input-suffix">
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={() => setShowPass((v) => !v)}
                  tabIndex={-1}
                  aria-label="Şifreyi göster/gizle"
                >
                  {showPass ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>
            </div>
          </div>

          {/* Hata mesajı */}
          {error && (
            <div className="alert alert-danger">
              <AlertCircle size={14} />
              <span>{error}</span>
            </div>
          )}

          {/* Kilit aç butonu */}
          <button
            type="submit"
            className="btn btn-primary btn-lg"
            disabled={isLoading || !password}
          >
            {isLoading ? (
              <>
                <div className="spinner" style={{ width: 16, height: 16 }} />
                <span>Doğrulanıyor...</span>
              </>
            ) : (
              <>
                <UnlockIcon size={16} />
                <span>Kilidi Aç</span>
              </>
            )}
          </button>
        </form>

        <p style={{ textAlign: 'center', fontSize: 11, color: 'var(--text-muted)', marginTop: 20 }}>
          🔒 Vault güvenli, tüm veriler şifreli
        </p>
      </div>
    </div>
  );
}
