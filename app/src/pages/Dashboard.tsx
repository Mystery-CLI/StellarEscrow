import { Link } from 'react-router-dom';
import { useGetTradesQuery } from '@stellar-escrow/state';
import { TradeCard } from '@stellar-escrow/components';
import OnboardingFlow from '../components/OnboardingFlow';

export default function Dashboard() {
  const { data: trades = [], isLoading, error } = useGetTradesQuery({});

  if (isLoading) return <p>Loading trades…</p>;
  if (error) return <p>Failed to load trades.</p>;

  return (
    <div>
      <div className="dashboard-header">
      <OnboardingFlow />
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
        <h1 style={{ fontSize: '1.5rem' }}>Trades</h1>
        <Link to="/trades/new" className="dashboard-new-btn">
          + New Trade
        </Link>
      </div>
      {trades.length === 0 ? (
        <p style={{ color: '#666' }}>No trades yet.</p>
      ) : (
        <div className="trades-grid">
          {trades.map((trade) => (
            <Link key={trade.id} to={`/trades/${trade.id}`} style={{ textDecoration: 'none' }}>
              <TradeCard
                tradeId={trade.id}
                seller={trade.seller}
                buyer={trade.buyer}
                amount={trade.amount}
                status={trade.status}
                timestamp={trade.timestamp}
              />
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
