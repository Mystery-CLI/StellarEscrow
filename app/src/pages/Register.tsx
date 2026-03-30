import React, { useState, useEffect, useMemo, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import {
  Input,
  Button,
  validateAuthField,
  isAuthFieldValid,
  type AuthFieldName,
} from '@stellar-escrow/components';

const API = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';
const AUTO_SAVE_KEY = 'stellar_escrow_register_draft';

interface RegisterFormData {
  address: string;
  usernameHash: string;
  contactHash: string;
}

export default function Register() {
  const navigate = useNavigate();

  // -- State -----------------------------------------------------------------
  const [formData, setFormData] = useState<RegisterFormData>(() => {
    const saved = localStorage.getItem(AUTO_SAVE_KEY);
    return saved ? JSON.parse(saved) : { address: '', usernameHash: '', contactHash: '' };
  });
  const [touched, setTouched] = useState<Partial<Record<AuthFieldName, boolean>>>({});
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [saveState, setSaveState] = useState<'idle' | 'saving' | 'saved'>('idle');
  const [submitAttempted, setSubmitAttempted] = useState(false);

  const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // -- Auto-save -------------------------------------------------------------
  useEffect(() => {
    const hasContent = Object.values(formData).some((v) => v.trim() !== '');
    if (!hasContent) return;

    setSaveState('saving');
    if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);

    autoSaveTimer.current = setTimeout(() => {
      localStorage.setItem(AUTO_SAVE_KEY, JSON.stringify(formData));
      setSaveState('saved');
      setTimeout(() => setSaveState('idle'), 2000);
    }, 800);

    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, [formData]);

  // -- Handlers --------------------------------------------------------------
  const handleChange = (field: AuthFieldName) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData((prev) => ({ ...prev, [field]: e.target.value }));
  };

  const handleBlur = (field: AuthFieldName) => () => {
    setTouched((prev) => ({ ...prev, [field]: true }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitAttempted(true);
    setError('');

    // Check all fields
    const fields: (keyof RegisterFormData)[] = ['address', 'usernameHash', 'contactHash'];
    const hasErrors = fields.some((f) => validateAuthField(f, formData[f]) !== null);

    if (hasErrors) {
      setError('Please fix the errors before submitting.');
      return;
    }

    setLoading(true);
    try {
      const res = await fetch(`${API}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          address: formData.address,
          username_hash: formData.usernameHash,
          contact_hash: formData.contactHash,
        }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body?.error?.detail ?? `HTTP ${res.status}`);
      }
      localStorage.removeItem(AUTO_SAVE_KEY);
      navigate(`/users/${formData.address}`);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  // -- Helpers ---------------------------------------------------------------
  const getFieldError = (field: keyof RegisterFormData) => {
    if (touched[field] || submitAttempted) {
      return validateAuthField(field, formData[field]);
    }
    return null;
  };

  const isFieldValidState = (field: keyof RegisterFormData) => {
    return (touched[field] || submitAttempted) && isAuthFieldValid(field, formData[field]);
  };

  return (
    <div style={styles.card}>
      <div style={styles.header}>
        <h2>Register</h2>
        {saveState !== 'idle' && (
          <span style={{ ...styles.saveIndicator, color: saveState === 'saved' ? '#198754' : '#666' }}>
            {saveState === 'saving' ? 'Saving...' : 'Draft saved'}
          </span>
        )}
      </div>
      
      <p style={styles.hint}>
        Hashes are SHA-256 of the plaintext, computed client-side before submission.
      </p>

      <form onSubmit={handleSubmit} style={styles.form} noValidate>
        <Input
          label="Stellar Address"
          value={formData.address}
          onChange={handleChange('address')}
          onBlur={handleBlur('address')}
          error={getFieldError('address') ?? undefined}
          valid={isFieldValidState('address')}
          placeholder="G…"
          disabled={loading}
        />

        <Input
          label="Username Hash (SHA-256)"
          value={formData.usernameHash}
          onChange={handleChange('usernameHash')}
          onBlur={handleBlur('usernameHash')}
          error={getFieldError('usernameHash') ?? undefined}
          valid={isFieldValidState('usernameHash')}
          placeholder="64-char hex"
          disabled={loading}
        />

        <Input
          label="Contact Hash (SHA-256)"
          value={formData.contactHash}
          onChange={handleChange('contactHash')}
          onBlur={handleBlur('contactHash')}
          error={getFieldError('contactHash') ?? undefined}
          valid={isFieldValidState('contactHash')}
          placeholder="64-char hex"
          disabled={loading}
        />

        {error && <p style={styles.error}>{error}</p>}

        <Button type="submit" loading={loading} style={{ marginTop: '0.5rem' }}>
          Register
        </Button>
      </form>

      <p style={{ marginTop: '1.5rem', fontSize: '0.875rem' }}>
        Already registered? <Link to="/login">Login</Link>
      </p>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  card: { maxWidth: 480, margin: '2rem auto', padding: '2rem', border: '1px solid #e2e8f0', borderRadius: 12, boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)' },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' },
  saveIndicator: { fontSize: '0.75rem', fontStyle: 'italic' },
  hint: { fontSize: '0.8rem', color: '#666', marginBottom: '1.5rem' },
  form: { display: 'flex', flexDirection: 'column', gap: '0.5rem' },
  error: { color: '#dc3545', fontSize: '0.875rem', marginTop: '0.5rem' },
};
