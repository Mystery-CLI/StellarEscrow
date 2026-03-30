import { useParams, Link } from 'react-router-dom';
import { useGetTradeQuery, useGetEventsByTradeQuery } from '@stellar-escrow/state';
import { TradeCard, EventFeed } from '@stellar-escrow/components';

export default function TradeDetail() {
  const { id } = useParams<{ id: string }>();
  const { data: trade, isLoading, error } = useGetTradeQuery(id!);
  const { data: events = [] } = useGetEventsByTradeQuery(id!);

  if (isLoading) return <p role="status" aria-live="polite">Loading trade details…</p>;
  if (error || !trade)
    return (
      <div role="alert">
        <p>Trade not found.</p>
        <Link to="/" className="back-link">
          Back to Dashboard
        </Link>
      </div>
    );

  return (
    <main className="trade-detail-grid" aria-labelledby="trade-title">
      <section>
        <h1 id="trade-title" style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>
          Trade #{trade.id}
        </h1>
        <TradeCard
          tradeId={trade.id}
          seller={trade.seller}
          buyer={trade.buyer}
          amount={trade.amount}
          status={trade.status}
          timestamp={trade.timestamp}
          aria-label={`Trade details for trade #${trade.id}`}
        />
      </section>

      <section aria-labelledby="trade-events-title">
        <h2 id="trade-events-title" style={{ fontSize: '1.1rem', marginBottom: '1rem' }}>
          Events
        </h2>
        <EventFeed events={events} aria-label={`Event feed for trade #${trade.id}`} />
      </section>
    </main>
  );
}