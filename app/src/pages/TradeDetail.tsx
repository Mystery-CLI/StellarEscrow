import { useParams, Link } from 'react-router-dom';
import { useGetTradeQuery, useGetEventsByTradeQuery } from '@stellar-escrow/state';
import { TradeCard, EventFeed } from '@stellar-escrow/components';

export default function TradeDetail() {
  const { id } = useParams<{ id: string }>();
  const { data: trade, isLoading, error } = useGetTradeQuery(id!);
  const { data: events = [] } = useGetEventsByTradeQuery(id!);

  if (isLoading) return <p>Loading…</p>;
  if (error || !trade) return <p>Trade not found. <Link to="/">Back</Link></p>;

  return (
    <div className="trade-detail-grid">
      <div>
        <h1 style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>Trade #{trade.id}</h1>
        <TradeCard
          tradeId={trade.id}
          seller={trade.seller}
          buyer={trade.buyer}
          amount={trade.amount}
          status={trade.status}
          timestamp={trade.timestamp}
        />
      </div>
      <div>
        <h2 style={{ fontSize: '1.1rem', marginBottom: '1rem' }}>Events</h2>
        <EventFeed events={events} />
      </div>
    </div>
  );
}
