// components/PasswordGenerator.tsx — Rastgele Şifre Üretici

import { useState, useCallback } from 'react';
import { RefreshCw, Copy, Check, ChevronDown, ChevronUp } from 'lucide-react';
import { generatePassword } from '../lib/tauri';
import type { GeneratorOptions } from '../types';
import { PasswordStrength } from './PasswordStrength';
import { useClipboard } from '../hooks/useClipboard';

interface PasswordGeneratorProps {
  onSelect?: (password: string) => void;
  compact?: boolean;
}

const DEFAULT_OPTIONS: GeneratorOptions = {
  length: 20,
  uppercase: true,
  lowercase: true,
  digits: true,
  symbols: true,
  exclude_ambiguous: false,
};

export function PasswordGenerator({ onSelect, compact = false }: PasswordGeneratorProps) {
  const [options, setOptions] = useState<GeneratorOptions>(DEFAULT_OPTIONS);
  const [generated, setGenerated] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [copied, setCopied] = useState(false);
  const [showOptions, setShowOptions] = useState(!compact);
  const { copy } = useClipboard({ clearAfterMs: 30000 });

  const generate = useCallback(async () => {
    setIsGenerating(true);
    try {
      const result = await generatePassword(options);
      setGenerated(result.password);
    } catch (err) {
      console.error('Şifre üretme hatası:', err);
    } finally {
      setIsGenerating(false);
    }
  }, [options]);

  const handleCopy = async () => {
    if (!generated) return;
    await copy(generated);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const toggleOption = (key: keyof GeneratorOptions) => {
    if (typeof options[key] === 'boolean') {
      setOptions((prev) => ({ ...prev, [key]: !prev[key] }));
    }
  };

  return (
    <div className="generator-panel">
      {/* Üretilen şifre çıktısı */}
      <div className="generator-output">
        <span style={{ flex: 1, minWidth: 0, wordBreak: 'break-all' }}>
          {generated || (
            <span className="text-muted">Şifre üretmek için butona tıklayın</span>
          )}
        </span>
        <div style={{ display: 'flex', gap: 4, flexShrink: 0 }}>
          <button
            className="btn btn-ghost btn-icon"
            onClick={handleCopy}
            disabled={!generated}
            title="Kopyala"
          >
            {copied ? <Check size={14} color="var(--success)" /> : <Copy size={14} />}
          </button>
          <button
            className="btn btn-primary btn-icon"
            onClick={generate}
            disabled={isGenerating}
            title="Yeni şifre üret"
          >
            <RefreshCw size={14} className={isGenerating ? 'animate-spin' : ''} />
          </button>
        </div>
      </div>

      {/* Şifre gücü göstergesi */}
      {generated && <PasswordStrength password={generated} />}

      {/* Seçenekler toggle */}
      {compact && (
        <button
          className="btn btn-ghost btn-sm"
          onClick={() => setShowOptions((v) => !v)}
          style={{ width: '100%', justifyContent: 'space-between' }}
        >
          <span>Seçenekler</span>
          {showOptions ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
        </button>
      )}

      {showOptions && (
        <>
          {/* Uzunluk slider */}
          <div className="slider-group">
            <div className="slider-header">
              <span>Uzunluk</span>
              <span className="slider-value">{options.length}</span>
            </div>
            <input
              type="range"
              min={8}
              max={64}
              value={options.length}
              onChange={(e) => setOptions((o) => ({ ...o, length: Number(e.target.value) }))}
              style={{ width: '100%', accentColor: 'var(--accent-primary)' }}
            />
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, color: 'var(--text-muted)' }}>
              <span>8</span>
              <span>64</span>
            </div>
          </div>

          {/* Karakter seti toggle'ları */}
          <div className="toggle-grid">
            {[
              { key: 'uppercase', label: 'A-Z Büyük' },
              { key: 'lowercase', label: 'a-z Küçük' },
              { key: 'digits', label: '0-9 Rakam' },
              { key: 'symbols', label: '!@# Özel' },
            ].map(({ key, label }) => (
              <div key={key} className="toggle-item">
                <span>{label}</span>
                <button
                  onClick={() => toggleOption(key as keyof GeneratorOptions)}
                  style={{
                    width: 36,
                    height: 20,
                    borderRadius: 10,
                    background: options[key as keyof GeneratorOptions]
                      ? 'var(--accent-primary)'
                      : 'var(--bg-tertiary)',
                    border: '1px solid var(--border-default)',
                    cursor: 'pointer',
                    position: 'relative',
                    transition: 'background var(--transition-fast)',
                    flexShrink: 0,
                  }}
                  aria-label={label}
                >
                  <div
                    style={{
                      width: 14,
                      height: 14,
                      borderRadius: '50%',
                      background: 'white',
                      position: 'absolute',
                      top: 2,
                      left: options[key as keyof GeneratorOptions] ? 19 : 2,
                      transition: 'left var(--transition-fast)',
                    }}
                  />
                </button>
              </div>
            ))}
          </div>

          {/* Belirsiz karakter hariç tut */}
          <div className="toggle-item" style={{ gridColumn: 'span 2' }}>
            <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
              Belirsiz karakterleri hariç tut (l, I, 1, O, 0)
            </span>
            <button
              onClick={() => toggleOption('exclude_ambiguous')}
              style={{
                width: 36, height: 20, borderRadius: 10,
                background: options.exclude_ambiguous ? 'var(--accent-primary)' : 'var(--bg-tertiary)',
                border: '1px solid var(--border-default)',
                cursor: 'pointer',
                position: 'relative',
                transition: 'background var(--transition-fast)',
                flexShrink: 0,
              }}
              aria-label="Belirsiz karakterleri hariç tut"
            >
              <div style={{
                width: 14, height: 14, borderRadius: '50%', background: 'white',
                position: 'absolute', top: 2,
                left: options.exclude_ambiguous ? 19 : 2,
                transition: 'left var(--transition-fast)',
              }} />
            </button>
          </div>
        </>
      )}

      {/* Kullan butonu (eğer onSelect varsa) */}
      {onSelect && generated && (
        <button
          className="btn btn-primary w-full"
          onClick={() => onSelect(generated)}
        >
          Bu Şifreyi Kullan
        </button>
      )}
    </div>
  );
}
