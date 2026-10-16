// hooks/useAutoLock.ts — Otomatik Kilitleme Hook
//
// Belirli bir süre (varsayılan 5 dk.) kullanıcı etkileşimi olmadığında
// vault otomatik olarak kilitlenir ve bellekteki anahtar temizlenir.
//
// İzlenen olaylar: fare hareketi, klavye, tıklama, scroll
// Timer her etkileşimde sıfırlanır.

import { useEffect, useRef } from 'react';
import { useAuthStore } from '../store/authStore';

export function useAutoLock(timeoutMinutes: number = 5) {
  const lock = useAuthStore((s) => s.lock);
  const isLocked = useAuthStore((s) => s.isLocked);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    // Vault kilitliyse timer çalıştırma
    if (isLocked) return;

    const timeoutMs = timeoutMinutes * 60 * 1000;

    const resetTimer = () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      timerRef.current = setTimeout(() => {
        lock();
      }, timeoutMs);
    };

    // Kullanıcı etkileşim olaylarını dinle
    const events = ['mousemove', 'keydown', 'mousedown', 'touchstart', 'scroll'];
    events.forEach((e) => window.addEventListener(e, resetTimer, { passive: true }));

    // İlk timer'ı başlat
    resetTimer();

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      events.forEach((e) => window.removeEventListener(e, resetTimer));
    };
  }, [isLocked, timeoutMinutes, lock]);
}
