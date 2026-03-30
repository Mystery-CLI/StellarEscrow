import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';

const API = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

/**
 * Login: verifies the user exists on-chain by fetching their profile.
 * Real auth (wallet signing) would happen here; for now we just confirm
 * the address is registered and store it in sessionStorage.
 */
export default function Login() {
  const navigate = useNavigate();
  const [address, setAddress] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const res = await fetch(`${API}/users/${encodeURIComponent(address)}`);
      if (res.status === 404) throw new Error('Address not registered. Please register first.');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      sessionStorage.setItem('stellar_address', address);
      navigate(`/users/${address}`);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={styles.card}>
      <h2>Login</h2>
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
        {error && <p style={styles.error}>{error}</p>}
        <button style={styles.btn} type="submit" disabled={loading}>
          {loading ? 'Checking…' : 'Login'}
        </button>
      </form>
      <p style={{ marginTop: '1rem', fontSize: '0.875rem' }}>
        New user? <Link to="/register">Register</Link>
      </p>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  card: { maxWidth: 480, margin: '2rem auto', padding: '2rem', border: '1px solid #e2e8f0', borderRadius: 8 },
  form: { display: 'flex', flexDirection: 'column', gap: '1rem' },
  label: { display: 'flex', flexDirection: 'column', gap: 4, fontSize: '0.875rem', fontWeight: 500 },
  input: { padding: '0.5rem', border: '1px solid #cbd5e0', borderRadius: 4, fontSize: '0.875rem' },
  btn: { padding: '0.6rem', background: '#1a1a2e', color: '#fff', border: 'none', borderRadius: 6, cursor: 'pointer' },
  error: { color: '#e53e3e', fontSize: '0.875rem' },
};
