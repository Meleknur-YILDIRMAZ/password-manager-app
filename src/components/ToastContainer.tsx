// components/ToastContainer.tsx — Bildirim Toast Container

import { CheckCircle, XCircle, Info, X } from 'lucide-react';
import type { Toast } from '../hooks/useToast';

interface ToastContainerProps {
  toasts: Toast[];
  dismiss: (id: string) => void;
}

const icons = {
  success: <CheckCircle size={16} color="var(--success)" />,
  error: <XCircle size={16} color="var(--danger)" />,
  info: <Info size={16} color="var(--info)" />,
};

export function ToastContainer({ toasts, dismiss }: ToastContainerProps) {
  return (
    <div className="toast-container">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast-${toast.type}`}>
          {icons[toast.type]}
          <span style={{ flex: 1 }}>{toast.message}</span>
          <button
            className="btn btn-ghost btn-icon"
            onClick={() => dismiss(toast.id)}
            style={{ width: 24, height: 24, minWidth: 24 }}
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
