import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import {
  Input,
  Button,
  validateAuthField,
  isAuthFieldValid,
} from '@stellar-escrow/components';

const API = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

export default function Login() {
  const navigate = useNavigate();
  const [address, setAddress] = useState('');
  const [touched, setTouched] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [submitAttempted, setSubmitAttempted] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setSubmitAttempted(true);
    setError('');

    const fieldError = validateAuthField('address', address);
    if (fieldError) {
      setError(fieldError);
      return;
    }

    setLoading(true);
    try {
      const res = await fetch(`${API}/users/${encodeURIComponent(address)}`);
      if (res.status === 404) throw new Error('Address not registered. Please register first.');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      
      sessionStorage.setItem('stellar_address', address);
      navigate(`/users/${address}`);
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred.');
    } finally {
      setLoading(false);
    }
  }

  const fieldError = (touched || submitAttempted) ? validateAuthField('address', address) : null;
  const isValid = (touched || submitAttempted) && isAuthFieldValid('address', address);

  return (
    <main style={styles.card} aria-labelledby="login-title">
      <h2 id="login-title">Login</h2>
      <form onSubmit={handleSubmit} style={styles.form} aria-describedby="login-error">
        <label style={styles.label}>
          Stellar Address
          <input
            style={styles.input}
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="G…"
            required
            aria-required="true"
            aria-label="Stellar Address"
          />
        </label>

        {error && (
          <p id="login-error" role="alert" style={styles.error}>
            {error}
          </p>
        )}

        <button style={styles.btn} type="submit" disabled={loading} aria-busy={loading}>
          {loading ? 'Checking…' : 'Login'}
        </button>
      </form>

      <p style={{ marginTop: '1rem', fontSize: '0.875rem' }}>
    <div style={styles.card}>
      <h2 style={{ marginBottom: '1.5rem' }}>Login</h2>
      <form onSubmit={handleSubmit} style={styles.form} noValidate>
        <Input
          label="Stellar Address"
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          onBlur={() => setTouched(true)}
          error={fieldError ?? undefined}
          valid={isValid}
          placeholder="G…"
          disabled={loading}
          autoFocus
        />
        
        {error && !fieldError && <p style={styles.error}>{error}</p>}
        
        <Button 
          type="submit" 
          loading={loading}
          style={{ marginTop: '0.5rem' }}
        >
          Login
        </Button>
      </form>
      <p style={{ marginTop: '1.5rem', fontSize: '0.875rem' }}>
        New user? <Link to="/register">Register</Link>
      </p>
    </main>
  );
}

const styles: Record<string, React.CSSProperties> = {
  card: {
    maxWidth: 480,
    margin: '2rem auto',
    padding: '2rem',
    border: '1px solid #e2e8f0',
    borderRadius: 8,
    backgroundColor: '#fff',
  },
  form: { display: 'flex', flexDirection: 'column', gap: '1rem' },
  label: { display: 'flex', flexDirection: 'column', gap: 4, fontSize: '0.875rem', fontWeight: 500 },
  input: { padding: '0.5rem', border: '1px solid #cbd5e0', borderRadius: 4, fontSize: '0.875rem' },
  btn: {
    padding: '0.6rem',
    background: '#1a1a2e',
    color: '#fff',
    border: 'none',
    borderRadius: 6,
    cursor: 'pointer',
    fontSize: '0.875rem',
  },
  error: { color: '#e53e3e', fontSize: '0.875rem' },
};
  card: { maxWidth: 480, margin: '2rem auto', padding: '2rem', border: '1px solid #e2e8f0', borderRadius: 12, boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)' },
  form: { display: 'flex', flexDirection: 'column', gap: '0.5rem' },
  error: { color: '#dc3545', fontSize: '0.875rem', marginTop: '0.5rem' },
};
