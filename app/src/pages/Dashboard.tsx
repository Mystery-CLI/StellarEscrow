import { Link } from 'react-router-dom';
import { Trade, useGetTradesQuery } from '@stellar-escrow/state';
import { TradeCard } from '@stellar-escrow/components';
import OnboardingFlow from '../components/OnboardingFlow';

export default function Dashboard() {
  const { data: trades = [], isLoading, error } = useGetTradesQuery({});

  if (isLoading) {
    return (
      <section role="status" aria-live="polite">
        <p>Loading trades…</p>
      </section>
    );
  }

  if (error) {
    return (
      <section role="alert" aria-live="assertive">
        <p>Failed to load trades. Please refresh or try again later.</p>
      </section>
    );
  }

  return (
    <main aria-labelledby="dashboard-title">
      <section aria-label="Onboarding">
        <OnboardingFlow />
      </section>

      <section
        className="dashboard-header"
        role="region"
        aria-labelledby="dashboard-title"
      >
    <div>
      <div className="dashboard-header">
        <OnboardingFlow />
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: '1.5rem',
          }}
        >
          <h1 id="dashboard-title" style={{ fontSize: '1.5rem' }}>
            Trades
          </h1>

          <Link
            to="/trades/new"
            className="dashboard-new-btn"
            role="button"
            aria-label="Create a new trade"
          >
            + New Trade
          </Link>
          <h1 style={{ fontSize: '1.5rem' }}>Trades</h1>
          <Link to="/trades/new" className="dashboard-new-btn">
            + New Trade
          </Link>
        </div>
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

        {trades.length === 0 ? (
          <p
            style={{ color: '#666' }}
            role="status"
            aria-live="polite"
          >
            No trades yet.
          </p>
        ) : (
          <div
            className="trades-grid"
            role="list"
            aria-label="List of trades"
          >
            {trades.map((trade: Trade) => (
              <Link
                key={trade.id}
                to={`/trades/${trade.id}`}
                style={{ textDecoration: 'none' }}
                role="listitem"
                aria-label={`View trade ${trade.id}`}
              >
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
      </section>
    </main>
  );
}