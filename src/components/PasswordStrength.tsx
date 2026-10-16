// components/PasswordStrength.tsx — zxcvbn Şifre Gücü Göstergesi

import zxcvbn from 'zxcvbn';

interface PasswordStrengthProps {
  password: string;
  showCrackTime?: boolean;
}

export function PasswordStrength({ password, showCrackTime = true }: PasswordStrengthProps) {
  if (!password) return null;

  const result = zxcvbn(password);
  const score = result.score; // 0-4

  const labels = ['Çok Zayıf', 'Zayıf', 'Orta', 'Güçlü', 'Çok Güçlü'];
  const crackTime = result.crack_times_display.offline_slow_hashing_1e4_per_second;

  return (
    <div className="strength-meter">
      <div className="strength-bars">
        {[0, 1, 2, 3, 4].map((i) => (
          <div
            key={i}
            className={`strength-bar ${i <= score ? `filled-${score}` : ''}`}
          />
        ))}
      </div>
      <div className={`strength-label strength-text-${score}`}>
        <span>{labels[score]}</span>
        {showCrackTime && (
          <span className="text-muted text-sm">
            Kırma süresi: {crackTime}
          </span>
        )}
      </div>
    </div>
  );
}
