// components/TOTPDisplay.tsx — TOTP Kodu Gösterimi (Gerçek zamanlı güncellenen)

import { useState, useEffect } from 'react';
import { Copy, Check } from 'lucide-react';
import { generateTotp } from '../lib/tauri';
import type { TotpCode } from '../types';
import { useClipboard } from '../hooks/useClipboard';

interface TOTPDisplayProps {
  secretBase32: string;
}

export function TOTPDisplay({ secretBase32 }: TOTPDisplayProps) {
  const [totp, setTotp] = useState<TotpCode | null>(null);
  const [copied, setCopied] = useState(false);
  const { copy } = useClipboard({ clearAfterMs: 30000 });

  useEffect(() => {
    let mounted = true;

    const fetchCode = async () => {
      try {
        const code = await generateTotp(secretBase32);
        if (mounted) setTotp(code);
      } catch {
        // Geçersiz secret — sessizce yoksay
      }
    };

    fetchCode();

    // Her saniye kalan süreyi güncelle, periyot sonunda kodu yenile
    const interval = setInterval(async () => {
      if (!mounted) return;
      setTotp((prev) => {
        if (prev && prev.remaining_seconds > 1) {
          return { ...prev, remaining_seconds: prev.remaining_seconds - 1 };
        }
        // Periyot doldu — yeni kodu çek
        fetchCode();
        return prev;
      });
    }, 1000);

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [secretBase32]);

  const handleCopy = async () => {
    if (!totp) return;
    await copy(totp.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (!totp) return null;

  // Çember progress: remaining / period
  const progress = totp.remaining_seconds / totp.period;
  const circumference = 2 * Math.PI * 11; // r=11
  const strokeDashoffset = circumference * (1 - progress);
  const isUrgent = totp.remaining_seconds <= 5;

  return (
    <div className="totp-display">
      {/* Geri sayım çemberi */}
      <svg width="28" height="28" viewBox="0 0 28 28" style={{ flexShrink: 0 }}>
        <circle
          cx="14" cy="14" r="11"
          fill="none"
          stroke="var(--border-default)"
          strokeWidth="2.5"
        />
        <circle
          cx="14" cy="14" r="11"
          fill="none"
          stroke={isUrgent ? 'var(--danger)' : 'var(--accent-primary)'}
          strokeWidth="2.5"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          transform="rotate(-90 14 14)"
          style={{ transition: 'stroke-dashoffset 0.9s linear, stroke 0.3s ease' }}
        />
        <text
          x="14" y="14"
          textAnchor="middle"
          dominantBaseline="central"
          fontSize="8"
          fontWeight="700"
          fill={isUrgent ? 'var(--danger)' : 'var(--accent-secondary)'}
          fontFamily="var(--font-mono)"
        >
          {totp.remaining_seconds}
        </text>
      </svg>

      {/* 6 haneli kod */}
      <span className="totp-code" style={{ color: isUrgent ? 'var(--danger)' : undefined }}>
        {totp.code.slice(0, 3)} {totp.code.slice(3)}
      </span>

      {/* Kopyala */}
      <button
        className="btn btn-ghost btn-icon"
        onClick={handleCopy}
        title="Kodu kopyala"
        style={{ marginLeft: 'auto' }}
      >
        {copied ? <Check size={14} color="var(--success)" /> : <Copy size={14} />}
      </button>
    </div>
  );
}
