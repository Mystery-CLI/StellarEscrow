import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';

const API = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

export default function Register() {
  const navigate = useNavigate();
  const [address, setAddress] = useState('');
  const [usernameHash, setUsernameHash] = useState('');
  const [contactHash, setContactHash] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const res = await fetch(`${API}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          address,
          username_hash: usernameHash,
          contact_hash: contactHash,
        }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body?.error?.detail ?? `HTTP ${res.status}`);
      }
      navigate(`/users/${address}`);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={styles.card}>
      <h2>Register</h2>
      <p style={styles.hint}>
        Hashes are SHA-256 of the plaintext, computed client-side before submission.
      </p>
      <form onSubmit={handleSubmit} style={styles.form}>
        <label style={styles.label}>
          Stellar Address
          <input
            style={styles.input}
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="G…"
            required
          />
        </label>
        <label style={styles.label}>
          Username Hash (SHA-256)
          <input
            style={styles.input}
            value={usernameHash}
            onChange={(e) => setUsernameHash(e.target.value)}
            placeholder="64-char hex"
            required
          />
        </label>
        <label style={styles.label}>
          Contact Hash (SHA-256)
          <input
            style={styles.input}
            value={contactHash}
            onChange={(e) => setContactHash(e.target.value)}
            placeholder="64-char hex"
            required
          />
        </label>
        {error && <p style={styles.error}>{error}</p>}
        <button style={styles.btn} type="submit" disabled={loading}>
          {loading ? 'Registering…' : 'Register'}
        </button>
      </form>
      <p style={{ marginTop: '1rem', fontSize: '0.875rem' }}>
        Already registered? <Link to="/login">Login</Link>
      </p>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  card: { maxWidth: 480, margin: '2rem auto', padding: '2rem', border: '1px solid #e2e8f0', borderRadius: 8 },
  hint: { fontSize: '0.8rem', color: '#666', marginBottom: '1rem' },
  form: { display: 'flex', flexDirection: 'column', gap: '1rem' },
  label: { display: 'flex', flexDirection: 'column', gap: 4, fontSize: '0.875rem', fontWeight: 500 },
  input: { padding: '0.5rem', border: '1px solid #cbd5e0', borderRadius: 4, fontSize: '0.875rem' },
  btn: { padding: '0.6rem', background: '#1a1a2e', color: '#fff', border: 'none', borderRadius: 6, cursor: 'pointer' },
  error: { color: '#e53e3e', fontSize: '0.875rem' },
};
