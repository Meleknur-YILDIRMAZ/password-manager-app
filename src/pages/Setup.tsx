// pages/Setup.tsx — İlk Kurulum Sayfası (Master Şifre Oluşturma)

import { useState } from 'react';
import { Eye, EyeOff, Lock, ShieldCheck, AlertTriangle } from 'lucide-react';
import { useAuthStore } from '../store/authStore';
import { PasswordStrength } from '../components/PasswordStrength';

export function Setup() {
  const { setup, isLoading, error, clearError } = useAuthStore();
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [showPass, setShowPass] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [localError, setLocalError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError('');
    clearError();

    if (password.length < 8) {
      setLocalError('Master şifre en az 8 karakter olmalıdır.');
      return;
    }
    if (password !== confirm) {
      setLocalError('Şifreler eşleşmiyor.');
      return;
    }

    try {
      await setup(password);
    } catch {
      // Hata store'da işleniyor
    }
  };

  const displayError = localError || error;

  return (
    <div className="auth-container">
      <div className="auth-card animate-slide-up">
        {/* Logo ve başlık */}
        <div className="auth-logo">
          <div className="auth-logo-icon">
            <Lock size={28} color="white" />
          </div>
          <h1 className="auth-title">Vault Kurulumu</h1>
          <p className="auth-subtitle">
            Master şifrenizi belirleyin. Bu şifre tüm verilerinizi şifreler.
          </p>
        </div>

        {/* Kritik uyarı */}
        <div className="alert alert-warning" style={{ marginBottom: 20 }}>
          <AlertTriangle size={16} style={{ flexShrink: 0, marginTop: 1 }} />
          <div>
            <strong>Önemli:</strong> Master şifrenizi unutursanız, verileriniz
            <strong> kurtarılamaz</strong>. Bu şifreyi güvenli bir yerde saklayın.
          </div>
        </div>

        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          {/* Master şifre */}
          <div className="form-group">
            <label className="form-label">Master Şifre</label>
            <div className="input-wrapper">
              <input
                id="master-password"
                type={showPass ? 'text' : 'password'}
                className="input input-mono"
                placeholder="En az 8 karakter..."
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete="new-password"
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

            {/* Şifre gücü göstergesi */}
            {password && (
              <div style={{ marginTop: 6 }}>
                <PasswordStrength password={password} />
              </div>
            )}
          </div>

          {/* Şifre onayı */}
          <div className="form-group">
            <label className="form-label">Şifreyi Onayla</label>
            <div className="input-wrapper">
              <input
                id="confirm-password"
                type={showConfirm ? 'text' : 'password'}
                className="input input-mono"
                placeholder="Şifreyi tekrar girin..."
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                autoComplete="new-password"
                required
              />
              <div className="input-suffix">
                {confirm && (
                  <ShieldCheck
                    size={15}
                    color={confirm === password ? 'var(--success)' : 'var(--danger)'}
                  />
                )}
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={() => setShowConfirm((v) => !v)}
                  tabIndex={-1}
                  aria-label="Onay şifresi göster/gizle"
                >
                  {showConfirm ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>
            </div>
          </div>

          {/* Hata mesajı */}
          {displayError && (
            <div className="alert alert-danger">
              <AlertTriangle size={14} />
              <span>{displayError}</span>
            </div>
          )}

          {/* Gönder butonu */}
          <button
            type="submit"
            className="btn btn-primary btn-lg"
            disabled={isLoading || !password || !confirm}
            style={{ marginTop: 4 }}
          >
            {isLoading ? (
              <>
                <div className="spinner" />
                <span>Vault oluşturuluyor...</span>
              </>
            ) : (
              <>
                <Lock size={16} />
                <span>Vault'u Oluştur</span>
              </>
            )}
          </button>
        </form>

        <p style={{ textAlign: 'center', fontSize: 11, color: 'var(--text-muted)', marginTop: 16 }}>
          Şifreleme: Argon2id + AES-256-GCM • Verileriniz yalnızca bu cihazda saklanır
        </p>
      </div>
    </div>
  );
}
