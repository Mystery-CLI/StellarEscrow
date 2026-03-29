import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';

const API = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

interface Profile {
  address: string;
  username_hash: string;
  contact_hash: string;
  avatar_hash: string | null;
  verification: string;
  two_fa_enabled: boolean;
  registered_at: number;
  updated_at: number;
}

interface Analytics {
  total_trades: number;
  trades_as_seller: number;
  trades_as_buyer: number;
  total_volume: number;
  completed_trades: number;
  disputed_trades: number;
  cancelled_trades: number;
}

interface Preference { key: string; value: string }

export default function UserProfile() {
  const { address } = useParams<{ address: string }>();
  const navigate = useNavigate();

  const [profile, setProfile] = useState<Profile | null>(null);
  const [analytics, setAnalytics] = useState<Analytics | null>(null);
  const [prefs, setPrefs] = useState<Preference[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  // Edit state
  const [editing, setEditing] = useState(false);
  const [usernameHash, setUsernameHash] = useState('');
  const [contactHash, setContactHash] = useState('');
  const [avatarHash, setAvatarHash] = useState('');
  const [saveError, setSaveError] = useState('');

  // Preference state
  const [prefKey, setPrefKey] = useState('');
  const [prefValue, setPrefValue] = useState('');

  useEffect(() => {
    if (!address) return;
    Promise.all([
      fetch(`${API}/users/${address}`).then((r) => r.json()),
      fetch(`${API}/users/${address}/analytics`).then((r) => r.json()),
      fetch(`${API}/users/${address}/preferences`).then((r) => r.json()),
    ])
      .then(([p, a, pr]) => {
        setProfile(p);
        setAnalytics(a);
        setPrefs(pr);
        setUsernameHash(p.username_hash ?? '');
        setContactHash(p.contact_hash ?? '');
        setAvatarHash(p.avatar_hash ?? '');
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [address]);

  async function saveProfile() {
    setSaveError('');
    try {
      const res = await fetch(`${API}/users/${address}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username_hash: usernameHash || undefined,
          contact_hash: contactHash || undefined,
          avatar_hash: avatarHash || undefined,
        }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      setProfile(await res.json());
      setEditing(false);
    } catch (e: any) {
      setSaveError(e.message);
    }
  }

  async function savePref() {
    if (!prefKey) return;
    const res = await fetch(`${API}/users/${address}/preferences`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key: prefKey, value: prefValue }),
    });
    if (res.ok) {
      const updated = await res.json();
      setPrefs((prev) => {
        const idx = prev.findIndex((p) => p.key === updated.key);
        return idx >= 0 ? prev.map((p, i) => (i === idx ? updated : p)) : [...prev, updated];
      });
      setPrefKey('');
      setPrefValue('');
    }
  }

  if (loading) return <p>Loading profile…</p>;
  if (error) return <p style={{ color: 'red' }}>{error}</p>;
  if (!profile) return <p>User not found.</p>;

  const verificationColor: Record<string, string> = {
    Verified: '#38a169',
    Pending: '#d69e2e',
    Rejected: '#e53e3e',
    Unverified: '#718096',
  };

  return (
    <div style={styles.page}>
      {/* Profile card */}
      <div style={styles.card}>
        <div style={styles.header}>
          <h2 style={{ margin: 0 }}>Profile</h2>
          <span
            style={{
              ...styles.badge,
              background: verificationColor[profile.verification] ?? '#718096',
            }}
          >
            {profile.verification}
          </span>
        </div>

        <p style={styles.mono}>{profile.address}</p>

        {editing ? (
          <div style={styles.form}>
            <label style={styles.label}>
              Username Hash
              <input style={styles.input} value={usernameHash} onChange={(e) => setUsernameHash(e.target.value)} />
            </label>
            <label style={styles.label}>
              Contact Hash
              <input style={styles.input} value={contactHash} onChange={(e) => setContactHash(e.target.value)} />
            </label>
            <label style={styles.label}>
              Avatar Hash (optional)
              <input style={styles.input} value={avatarHash} onChange={(e) => setAvatarHash(e.target.value)} />
            </label>
            {saveError && <p style={styles.error}>{saveError}</p>}
            <div style={{ display: 'flex', gap: 8 }}>
              <button style={styles.btn} onClick={saveProfile}>Save</button>
              <button style={{ ...styles.btn, background: '#718096' }} onClick={() => setEditing(false)}>Cancel</button>
            </div>
          </div>
        ) : (
          <div>
            <Row label="Username Hash" value={profile.username_hash} />
            <Row label="Contact Hash" value={profile.contact_hash} />
            {profile.avatar_hash && <Row label="Avatar Hash" value={profile.avatar_hash} />}
            <Row label="2FA" value={profile.two_fa_enabled ? 'Enabled' : 'Disabled'} />
            <button style={{ ...styles.btn, marginTop: '1rem' }} onClick={() => setEditing(true)}>
              Edit Profile
            </button>
          </div>
        )}
      </div>

      {/* Analytics card */}
      {analytics && (
        <div style={styles.card}>
          <h3>Analytics</h3>
          <div style={styles.grid}>
            <Stat label="Total Trades" value={analytics.total_trades} />
            <Stat label="As Seller" value={analytics.trades_as_seller} />
            <Stat label="As Buyer" value={analytics.trades_as_buyer} />
            <Stat label="Completed" value={analytics.completed_trades} />
            <Stat label="Disputed" value={analytics.disputed_trades} />
            <Stat label="Cancelled" value={analytics.cancelled_trades} />
            <Stat label="Total Volume" value={`${(analytics.total_volume / 1e7).toFixed(2)} USDC`} />
          </div>
        </div>
      )}

      {/* Preferences card */}
      <div style={styles.card}>
        <h3>Preferences</h3>
        {prefs.length === 0 ? (
          <p style={{ color: '#718096', fontSize: '0.875rem' }}>No preferences set.</p>
        ) : (
          <table style={styles.table}>
            <thead>
              <tr>
                <th style={styles.th}>Key</th>
                <th style={styles.th}>Value</th>
              </tr>
            </thead>
            <tbody>
              {prefs.map((p) => (
                <tr key={p.key}>
                  <td style={styles.td}>{p.key}</td>
                  <td style={styles.td}>{p.value}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div style={{ display: 'flex', gap: 8, marginTop: '1rem' }}>
          <input
            style={{ ...styles.input, flex: 1 }}
            placeholder="key"
            value={prefKey}
            onChange={(e) => setPrefKey(e.target.value)}
          />
          <input
            style={{ ...styles.input, flex: 2 }}
            placeholder="value"
            value={prefValue}
            onChange={(e) => setPrefValue(e.target.value)}
          />
          <button style={styles.btn} onClick={savePref}>Set</button>
        </div>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '0.4rem 0', borderBottom: '1px solid #f0f0f0' }}>
      <span style={{ color: '#718096', fontSize: '0.875rem' }}>{label}</span>
      <span style={{ fontSize: '0.875rem', fontFamily: 'monospace', maxWidth: '60%', overflow: 'hidden', textOverflow: 'ellipsis' }}>{value}</span>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div style={{ textAlign: 'center', padding: '0.75rem', background: '#f7fafc', borderRadius: 6 }}>
      <div style={{ fontSize: '1.25rem', fontWeight: 700 }}>{value}</div>
      <div style={{ fontSize: '0.75rem', color: '#718096' }}>{label}</div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  page: { maxWidth: 640, margin: '2rem auto', display: 'flex', flexDirection: 'column', gap: '1.5rem' },
  card: { padding: '1.5rem', border: '1px solid #e2e8f0', borderRadius: 8 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.75rem' },
  badge: { color: '#fff', padding: '2px 10px', borderRadius: 12, fontSize: '0.75rem', fontWeight: 600 },
  mono: { fontFamily: 'monospace', fontSize: '0.8rem', color: '#4a5568', wordBreak: 'break-all' },
  form: { display: 'flex', flexDirection: 'column', gap: '0.75rem' },
  label: { display: 'flex', flexDirection: 'column', gap: 4, fontSize: '0.875rem', fontWeight: 500 },
  input: { padding: '0.5rem', border: '1px solid #cbd5e0', borderRadius: 4, fontSize: '0.875rem' },
  btn: { padding: '0.5rem 1rem', background: '#1a1a2e', color: '#fff', border: 'none', borderRadius: 6, cursor: 'pointer', fontSize: '0.875rem' },
  error: { color: '#e53e3e', fontSize: '0.875rem' },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))', gap: '0.75rem' },
  table: { width: '100%', borderCollapse: 'collapse', fontSize: '0.875rem' },
  th: { textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e2e8f0', color: '#718096' },
  td: { padding: '0.4rem 0.5rem', borderBottom: '1px solid #f0f0f0' },
};
