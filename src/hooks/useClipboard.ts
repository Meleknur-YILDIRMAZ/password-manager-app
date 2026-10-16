// hooks/useClipboard.ts — Güvenli Pano Yönetimi
//
// Şifre kopyalandıktan belirli bir süre (varsayılan 30 sn.) sonra
// pano otomatik olarak temizlenir.
//
// NOT: Web Clipboard API'si HTTPS veya localhost gerektirir.
// Tauri ortamında bu kısıtlama aşılır.

import { useCallback, useRef } from 'react';

interface UseClipboardOptions {
  clearAfterMs?: number; // Varsayılan: 30000 (30 saniye)
  onCopied?: () => void;
  onCleared?: () => void;
}

export function useClipboard(options: UseClipboardOptions = {}) {
  const { clearAfterMs = 30000, onCopied, onCleared } = options;
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copy = useCallback(
    async (text: string): Promise<boolean> => {
      try {
        await navigator.clipboard.writeText(text);

        onCopied?.();

        // Önceki timer'ı iptal et
        if (timerRef.current) clearTimeout(timerRef.current);

        // Belirli süre sonra panoyu temizle
        timerRef.current = setTimeout(async () => {
          try {
            // Pano hâlâ bizim kopyaladığımız şeyi içeriyorsa temizle
            const current = await navigator.clipboard.readText();
            if (current === text) {
              await navigator.clipboard.writeText('');
              onCleared?.();
            }
          } catch {
            // Pano okuma izni yoksa sessizce yoksay
          }
        }, clearAfterMs);

        return true;
      } catch (err) {
        console.error('Panoya kopyalama başarısız:', err);
        return false;
      }
    },
    [clearAfterMs, onCopied, onCleared]
  );

  const cancel = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  return { copy, cancel };
}
