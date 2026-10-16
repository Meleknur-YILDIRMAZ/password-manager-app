// pages/Dashboard.tsx — Ana Dashboard (Kayıt Listesi + Detay Paneli)

import { useEffect, useState, useCallback } from 'react';
import {
  Search, Plus, Star, Lock, Shield, Settings,
  Copy, Check, Eye, EyeOff, Trash2, Edit3, X, AlertTriangle,
  Key, Globe, User, FileText, Clock, Zap, AlertCircle
} from 'lucide-react';

import { useAuthStore } from '../store/authStore';
import { useVaultStore, EMPTY_ENTRY } from '../store/vaultStore';
import { useAutoLock } from '../hooks/useAutoLock';
import { useClipboard } from '../hooks/useClipboard';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/ToastContainer';
import { PasswordStrength } from '../components/PasswordStrength';
import { PasswordGenerator } from '../components/PasswordGenerator';
import { TOTPDisplay } from '../components/TOTPDisplay';
import { checkHibp } from '../lib/tauri';
import type { VaultEntry } from '../types';

const CATEGORIES = ['Genel', 'Sosyal Medya', 'Banka/Finans', 'E-posta', 'İş', 'Alışveriş', 'Eğlence', 'Diğer'];

export function Dashboard() {
  const { lock, settings } = useAuthStore();
  const {
    entries, selectedEntry, selectedId, isLoading,
    filteredEntries, categories, searchQuery, selectedCategory,
    loadEntries, selectEntry, createEntry, updateEntry, deleteEntry,
    setSearchQuery, setSelectedCategory, runAnalysis, analysis,
  } = useVaultStore();

  const { toasts, success, error: toastError, info, dismiss } = useToast();
  const { copy } = useClipboard({
    clearAfterMs: (settings?.clipboard_clear_seconds ?? 30) * 1000,
    onCleared: () => info('Pano temizlendi'),
  });

  // Auto-lock hook
  useAutoLock(settings?.auto_lock_minutes ?? 5);

  // UI State
  const [showForm, setShowForm] = useState(false);
  const [editingEntry, setEditingEntry] = useState<Partial<VaultEntry> | null>(null);
  const [showPassInDetail, setShowPassInDetail] = useState(false);
  const [showGenInForm, setShowGenInForm] = useState(false);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [hibpLoading, setHibpLoading] = useState(false);
  const [hibpResult, setHibpResult] = useState<{ pwned: boolean; count?: number } | null>(null);

  useEffect(() => {
    loadEntries();
    runAnalysis();
  }, []);

  const handleCopy = async (text: string, fieldId: string) => {
    await copy(text);
    setCopiedField(fieldId);
    success('Kopyalandı — 30 saniye sonra temizlenecek');
    setTimeout(() => setCopiedField(null), 2000);
  };

  const handleSelectEntry = useCallback(
    async (id: string) => {
      setShowPassInDetail(false);
      setHibpResult(null);
      await selectEntry(id);
    },
    [selectEntry]
  );

  const handleNewEntry = () => {
    setEditingEntry({ ...EMPTY_ENTRY });
    setShowForm(true);
    setShowGenInForm(false);
  };

  const handleEditEntry = () => {
    if (!selectedEntry) return;
    setEditingEntry({ ...selectedEntry });
    setShowForm(true);
  };

  const handleSaveEntry = async () => {
    if (!editingEntry?.name?.trim()) {
      toastError('Kayıt adı zorunludur.');
      return;
    }

    try {
      if (editingEntry.id) {
        await updateEntry(editingEntry as VaultEntry);
        success('Kayıt güncellendi');
        await selectEntry(editingEntry.id);
      } else {
        const id = await createEntry(editingEntry as Omit<VaultEntry, 'id' | 'created_at' | 'updated_at'>);
        success('Kayıt oluşturuldu');
        await selectEntry(id);
      }
      setShowForm(false);
      setEditingEntry(null);
      runAnalysis();
    } catch (err) {
      toastError(String(err));
    }
  };

  const handleDeleteEntry = async (id: string) => {
    try {
      await deleteEntry(id);
      success('Kayıt silindi');
      setDeleteConfirmId(null);
    } catch (err) {
      toastError(String(err));
    }
  };

  const handleHibpCheck = async () => {
    if (!selectedEntry?.password) return;
    setHibpLoading(true);
    try {
      const result = await checkHibp(selectedEntry.password);
      setHibpResult({ pwned: result.is_pwned, count: result.pwn_count });
    } catch {
      toastError('HIBP kontrolü başarısız. İnternet bağlantısını kontrol edin.');
    } finally {
      setHibpLoading(false);
    }
  };

  const filtered = filteredEntries();
  const cats = categories();

  // Analiz uyarılarını bul
  const analysisMap = new Map(analysis.map((a) => [a.id, a]));

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString('tr-TR', {
      day: 'numeric', month: 'long', year: 'numeric'
    });
  };

  const getInitial = (name: string) => name.charAt(0).toUpperCase();

  return (
    <div className="app-container">
      {/* ── Sidebar ── */}
      <aside className="sidebar">
        {/* Logo */}
        <div className="sidebar-logo">
          <div className="sidebar-logo-icon">
            <Shield size={18} color="white" />
          </div>
          <span className="sidebar-logo-text">Vault</span>
          <button
            className="btn btn-ghost btn-icon"
            onClick={lock}
            title="Kilitle"
            style={{ marginLeft: 'auto' }}
          >
            <Lock size={14} />
          </button>
        </div>

        {/* Navigasyon */}
        <nav className="sidebar-nav">
          <span className="section-title">Filtreler</span>

          {[
            { id: 'Tümü', label: 'Tüm Kayıtlar', count: entries.length },
            { id: 'Favoriler', label: 'Favoriler', count: entries.filter((e) => e.is_favorite).length },
          ].map(({ id, label, count }) => (
            <button
              key={id}
              className={`nav-item ${selectedCategory === id ? 'active' : ''}`}
              onClick={() => setSelectedCategory(id)}
            >
              {id === 'Favoriler' ? <Star size={14} /> : <Shield size={14} />}
              <span>{label}</span>
              {count > 0 && <span className="nav-item-count">{count}</span>}
            </button>
          ))}

          {cats.length > 0 && (
            <>
              <div className="divider" style={{ margin: '8px 0' }} />
              <span className="section-title">Kategoriler</span>
              {cats.map((cat) => (
                <button
                  key={cat}
                  className={`nav-item ${selectedCategory === cat ? 'active' : ''}`}
                  onClick={() => setSelectedCategory(cat)}
                >
                  <Key size={14} />
                  <span>{cat}</span>
                  <span className="nav-item-count">{entries.filter((e) => e.category === cat).length}</span>
                </button>
              ))}
            </>
          )}

          {/* Analiz uyarıları */}
          {analysis.some((a) => a.is_weak || a.is_duplicate) && (
            <>
              <div className="divider" style={{ margin: '8px 0' }} />
              <div
                className="alert alert-warning"
                style={{ margin: '0 4px', padding: '8px 10px', fontSize: 11 }}
              >
                <AlertTriangle size={13} />
                <span>
                  {analysis.filter((a) => a.is_weak).length} zayıf,{' '}
                  {analysis.filter((a) => a.is_duplicate).length} tekrarlanan şifre
                </span>
              </div>
            </>
          )}
        </nav>

        {/* Alt butonlar */}
        <div style={{ padding: '0 8px 8px' }}>
          <div className="divider" style={{ marginBottom: 8 }} />
          <button className="nav-item" style={{ width: '100%' }}>
            <Settings size={14} />
            <span>Ayarlar</span>
          </button>
        </div>
      </aside>

      {/* ── Ana İçerik ── */}
      <main className="main-content">
        {/* Topbar */}
        <div className="topbar">
          <div className="search-bar" style={{ flex: 1 }}>
            <Search size={14} color="var(--text-muted)" />
            <input
              id="vault-search"
              type="text"
              placeholder="Kayıt ara..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button className="btn btn-ghost btn-icon" onClick={() => setSearchQuery('')} style={{ padding: 2 }}>
                <X size={12} />
              </button>
            )}
          </div>
          <button className="btn btn-primary" onClick={handleNewEntry} id="new-entry-btn">
            <Plus size={14} />
            <span>Yeni Kayıt</span>
          </button>
        </div>

        {/* Kayıt listesi */}
        <div className="entry-list">
          {isLoading && !entries.length ? (
            <div className="empty-state">
              <div className="spinner" style={{ width: 32, height: 32 }} />
            </div>
          ) : filtered.length === 0 ? (
            <div className="empty-state">
              <div className="empty-state-icon">🔐</div>
              <p className="empty-state-title">
                {searchQuery ? 'Sonuç bulunamadı' : 'Henüz kayıt yok'}
              </p>
              <p className="empty-state-desc">
                {searchQuery
                  ? 'Farklı bir arama terimi deneyin'
                  : '"Yeni Kayıt" butonuyla ilk şifrenizi ekleyin'}
              </p>
            </div>
          ) : (
            filtered.map((entry) => {
              const alert = analysisMap.get(entry.id);
              return (
                <div
                  key={entry.id}
                  className={`entry-item ${selectedId === entry.id ? 'active' : ''}`}
                  onClick={() => handleSelectEntry(entry.id)}
                  id={`entry-${entry.id}`}
                >
                  <div className="entry-icon">
                    {getInitial(entry.name)}
                  </div>
                  <div className="entry-info">
                    <div className="entry-name">{entry.name}</div>
                    <div className="entry-username">{entry.username || entry.url || entry.category}</div>
                  </div>
                  <div className="entry-badges">
                    {entry.is_favorite && <Star size={12} color="var(--warning)" fill="var(--warning)" />}
                    {entry.has_totp && <span className="badge badge-accent">2FA</span>}
                    {alert?.is_weak && <span className="badge badge-warning">Zayıf</span>}
                    {alert?.is_duplicate && <span className="badge badge-danger">Tekrar</span>}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </main>

      {/* ── Detay Paneli ── */}
      {selectedEntry && !showForm && (
        <aside className="detail-panel animate-fade-in">
          <div className="detail-header">
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <div className="entry-icon" style={{ width: 32, height: 32, fontSize: 13 }}>
                {getInitial(selectedEntry.name)}
              </div>
              <div>
                <div style={{ fontWeight: 600, fontSize: 14 }}>{selectedEntry.name}</div>
                <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>{selectedEntry.category}</div>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 4 }}>
              <button className="btn btn-ghost btn-icon" onClick={handleEditEntry} title="Düzenle">
                <Edit3 size={14} />
              </button>
              <button
                className="btn btn-ghost btn-icon"
                onClick={() => setDeleteConfirmId(selectedEntry.id)}
                title="Sil"
                style={{ color: 'var(--danger)' }}
              >
                <Trash2 size={14} />
              </button>
              <button className="btn btn-ghost btn-icon" onClick={() => selectEntry(null)} title="Kapat">
                <X size={14} />
              </button>
            </div>
          </div>

          <div className="detail-body">
            {/* Kullanıcı adı */}
            {selectedEntry.username && (
              <div className="field-row">
                <div className="field-label"><User size={10} style={{ marginRight: 4 }} />Kullanıcı Adı</div>
                <div className="field-value">
                  <span className="flex-1 truncate">{selectedEntry.username}</span>
                  <div className="field-actions">
                    <button
                      className="btn btn-ghost btn-icon"
                      onClick={() => handleCopy(selectedEntry.username!, `username-${selectedEntry.id}`)}
                    >
                      {copiedField === `username-${selectedEntry.id}` ? <Check size={13} color="var(--success)" /> : <Copy size={13} />}
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Şifre */}
            {selectedEntry.password && (
              <div className="field-row">
                <div className="field-label"><Key size={10} style={{ marginRight: 4 }} />Şifre</div>
                <div className={`field-value password-field ${showPassInDetail ? '' : 'blurred'}`}
                  onClick={() => !showPassInDetail && setShowPassInDetail(true)}>
                  <span className="flex-1 truncate">{selectedEntry.password}</span>
                  <div className="field-actions">
                    <button className="btn btn-ghost btn-icon" onClick={() => setShowPassInDetail((v) => !v)}>
                      {showPassInDetail ? <EyeOff size={13} /> : <Eye size={13} />}
                    </button>
                    <button
                      className="btn btn-ghost btn-icon"
                      onClick={() => handleCopy(selectedEntry.password!, `password-${selectedEntry.id}`)}
                    >
                      {copiedField === `password-${selectedEntry.id}` ? <Check size={13} color="var(--success)" /> : <Copy size={13} />}
                    </button>
                  </div>
                </div>

                {/* Şifre gücü */}
                {showPassInDetail && (
                  <div style={{ marginTop: 6 }}>
                    <PasswordStrength password={selectedEntry.password} />
                  </div>
                )}

                {/* HIBP kontrolü */}
                {settings?.hibp_enabled && (
                  <div style={{ marginTop: 6, display: 'flex', alignItems: 'center', gap: 8 }}>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={handleHibpCheck}
                      disabled={hibpLoading}
                      id="hibp-check-btn"
                    >
                      {hibpLoading ? <div className="spinner" style={{ width: 12, height: 12 }} /> : <Zap size={12} />}
                      <span>Sızıntı Kontrolü</span>
                    </button>
                    {hibpResult && (
                      <span className={`badge ${hibpResult.pwned ? 'badge-danger' : 'badge-success'}`}>
                        {hibpResult.pwned
                          ? `⚠ ${hibpResult.count?.toLocaleString()} ihlalde görüldü`
                          : '✓ Sızıntı yok'}
                      </span>
                    )}
                  </div>
                )}
              </div>
            )}

            {/* URL */}
            {selectedEntry.url && (
              <div className="field-row">
                <div className="field-label"><Globe size={10} style={{ marginRight: 4 }} />URL</div>
                <div className="field-value">
                  <span className="flex-1 truncate" style={{ color: 'var(--accent-secondary)', fontSize: 13 }}>
                    {selectedEntry.url}
                  </span>
                  <div className="field-actions">
                    <button className="btn btn-ghost btn-icon" onClick={() => handleCopy(selectedEntry.url!, `url-${selectedEntry.id}`)}>
                      {copiedField === `url-${selectedEntry.id}` ? <Check size={13} color="var(--success)" /> : <Copy size={13} />}
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Notlar */}
            {selectedEntry.notes && (
              <div className="field-row">
                <div className="field-label"><FileText size={10} style={{ marginRight: 4 }} />Notlar</div>
                <div className="field-value" style={{ minHeight: 60, alignItems: 'flex-start', flexDirection: 'column' }}>
                  <span style={{ fontSize: 13, lineHeight: 1.6, whiteSpace: 'pre-wrap', userSelect: 'text' }}>
                    {selectedEntry.notes}
                  </span>
                </div>
              </div>
            )}

            {/* TOTP */}
            {selectedEntry.totp_secret && (
              <div className="field-row">
                <div className="field-label">2FA Kodu</div>
                <TOTPDisplay secretBase32={selectedEntry.totp_secret} />
              </div>
            )}

            {/* Meta bilgiler */}
            <div style={{ marginTop: 'auto', paddingTop: 12, borderTop: '1px solid var(--border-subtle)' }}>
              <div style={{ display: 'flex', gap: 4, fontSize: 11, color: 'var(--text-muted)', alignItems: 'center' }}>
                <Clock size={10} />
                <span>Güncellendi: {formatDate(selectedEntry.updated_at)}</span>
              </div>
              {selectedEntry.tags && (
                <div style={{ marginTop: 8, display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                  {selectedEntry.tags.split(',').map((tag) => tag.trim()).filter(Boolean).map((tag) => (
                    <span key={tag} className="badge badge-neutral">{tag}</span>
                  ))}
                </div>
              )}
            </div>
          </div>
        </aside>
      )}

      {/* ── Kayıt Formu (Yeni/Düzenle) ── */}
      {showForm && editingEntry && (
        <div className="dialog-overlay" onClick={(e) => e.target === e.currentTarget && setShowForm(false)}>
          <div className="dialog-content">
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 20 }}>
              <h2 className="dialog-title">{editingEntry.id ? 'Kaydı Düzenle' : 'Yeni Kayıt'}</h2>
              <button className="btn btn-ghost btn-icon" onClick={() => setShowForm(false)}>
                <X size={16} />
              </button>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
              {/* Ad */}
              <div className="form-group">
                <label className="form-label">Kayıt Adı *</label>
                <input
                  className="input"
                  placeholder="ör. Google, GitHub..."
                  value={editingEntry.name || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, name: e.target.value }))}
                  autoFocus
                />
              </div>

              {/* Kullanıcı adı */}
              <div className="form-group">
                <label className="form-label">Kullanıcı Adı / E-posta</label>
                <input
                  className="input"
                  placeholder="kullanici@email.com"
                  value={editingEntry.username || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, username: e.target.value }))}
                />
              </div>

              {/* Şifre */}
              <div className="form-group">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <label className="form-label">Şifre</label>
                  <button
                    className="btn btn-ghost btn-sm"
                    onClick={() => setShowGenInForm((v) => !v)}
                    type="button"
                  >
                    <Zap size={11} />
                    <span>Üret</span>
                  </button>
                </div>
                <input
                  className="input input-mono"
                  type="password"
                  placeholder="••••••••"
                  value={editingEntry.password || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, password: e.target.value }))}
                />
                {editingEntry.password && (
                  <div style={{ marginTop: 6 }}>
                    <PasswordStrength password={editingEntry.password} />
                  </div>
                )}
              </div>

              {/* Şifre üretici */}
              {showGenInForm && (
                <PasswordGenerator
                  compact
                  onSelect={(pwd) => {
                    setEditingEntry((p) => ({ ...p, password: pwd }));
                    setShowGenInForm(false);
                  }}
                />
              )}

              {/* URL */}
              <div className="form-group">
                <label className="form-label">URL</label>
                <input
                  className="input"
                  placeholder="https://example.com"
                  value={editingEntry.url || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, url: e.target.value }))}
                />
              </div>

              {/* Kategori */}
              <div className="form-group">
                <label className="form-label">Kategori</label>
                <select
                  className="input"
                  value={editingEntry.category || 'Genel'}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, category: e.target.value }))}
                  style={{ cursor: 'pointer' }}
                >
                  {CATEGORIES.map((cat) => (
                    <option key={cat} value={cat}>{cat}</option>
                  ))}
                </select>
              </div>

              {/* TOTP Secret */}
              <div className="form-group">
                <label className="form-label">2FA Secret (opsiyonel)</label>
                <input
                  className="input input-mono"
                  placeholder="JBSWY3DPEHPK3PXP"
                  value={editingEntry.totp_secret || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...(p ?? {}), totp_secret: e.target.value }))}
                />
              </div>

              {/* Notlar */}
              <div className="form-group">
                <label className="form-label">Notlar</label>
                <textarea
                  className="input"
                  placeholder="Ek bilgiler..."
                  rows={3}
                  value={editingEntry.notes || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, notes: e.target.value }))}
                  style={{ resize: 'vertical', minHeight: 70 }}
                />
              </div>

              {/* Etiketler */}
              <div className="form-group">
                <label className="form-label">Etiketler (virgülle ayırın)</label>
                <input
                  className="input"
                  placeholder="iş, kişisel, önemli"
                  value={editingEntry.tags || ''}
                  onChange={(e) => setEditingEntry((p) => ({ ...p, tags: e.target.value }))}
                />
              </div>

              {/* Favori */}
              <div className="toggle-item">
                <span>Favorilere ekle</span>
                <button
                  onClick={() => setEditingEntry((p) => ({ ...(p ?? {}), is_favorite: !(p?.is_favorite) }))}
                  style={{
                    width: 36, height: 20, borderRadius: 10,
                    background: editingEntry.is_favorite ? 'var(--warning)' : 'var(--bg-tertiary)',
                    border: '1px solid var(--border-default)', cursor: 'pointer',
                    position: 'relative', transition: 'background var(--transition-fast)', flexShrink: 0,
                  }}
                >
                  <div style={{
                    width: 14, height: 14, borderRadius: '50%', background: 'white',
                    position: 'absolute', top: 2,
                    left: editingEntry.is_favorite ? 19 : 2,
                    transition: 'left var(--transition-fast)',
                  }} />
                </button>
              </div>

              {/* Butonlar */}
              <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
                <button className="btn btn-secondary flex-1" onClick={() => setShowForm(false)}>
                  İptal
                </button>
                <button className="btn btn-primary flex-1" onClick={handleSaveEntry} id="save-entry-btn">
                  {editingEntry.id ? 'Güncelle' : 'Oluştur'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── Silme Onayı ── */}
      {deleteConfirmId && (
        <div className="dialog-overlay" onClick={() => setDeleteConfirmId(null)}>
          <div className="dialog-content" style={{ maxWidth: 360 }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', gap: 12, alignItems: 'flex-start', marginBottom: 20 }}>
              <AlertCircle size={24} color="var(--danger)" style={{ flexShrink: 0 }} />
              <div>
                <h2 style={{ fontSize: 16, fontWeight: 600, marginBottom: 6 }}>Kaydı Sil</h2>
                <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>
                  Bu işlem geri alınamaz. Şifre kalıcı olarak silinecek.
                </p>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              <button className="btn btn-secondary flex-1" onClick={() => setDeleteConfirmId(null)}>İptal</button>
              <button className="btn btn-danger flex-1" onClick={() => handleDeleteEntry(deleteConfirmId)}>
                Sil
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Toast bildirimleri */}
      <ToastContainer toasts={toasts} dismiss={dismiss} />
    </div>
  );
}
