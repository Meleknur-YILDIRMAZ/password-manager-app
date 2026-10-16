// App.tsx — Ana Uygulama Bileşeni
// Vault durumuna göre doğru sayfayı render eder (setup / unlock / dashboard)

import { useEffect } from 'react';
import { useAuthStore } from './store/authStore';
import { Setup } from './pages/Setup';
import { Unlock } from './pages/Unlock';
import { Dashboard } from './pages/Dashboard';

export default function App() {
  const { isSetup, isLocked, isLoading, checkSetup } = useAuthStore();

  useEffect(() => {
    checkSetup();
  }, [checkSetup]);

  if (isLoading && !isSetup) {
    return (
      <div
        style={{
          height: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'var(--bg-primary)',
        }}
      >
        <div className="spinner" style={{ width: 32, height: 32 }} />
      </div>
    );
  }

  // İlk kez kurulum
  if (!isSetup) {
    return <Setup />;
  }

  // Vault kilitli
  if (isLocked) {
    return <Unlock />;
  }

  // Vault açık — ana dashboard
  return <Dashboard />;
}
