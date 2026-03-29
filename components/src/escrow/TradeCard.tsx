import React from 'react';
import { Card } from '../base/Card';
import { Badge } from '../base/Badge';
import './TradeCard.css';

export interface TradeCardProps {
  tradeId: string;
  seller: string;
  buyer: string;
  amount: string;
  status: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';
  timestamp: string;
  onClick?: () => void;
}

export const TradeCard: React.FC<TradeCardProps> = ({
  tradeId,
  seller,
  buyer,
  amount,
  status,
  timestamp,
  onClick,
}) => {
  const statusVariant = status === 'completed' ? 'success' : status === 'disputed' ? 'danger' : 'info';

  return (
    <Card className="trade-card" onClick={onClick} role="button" tabIndex={0}>
      <div className="trade-card-header">
        <div>
          <h4 className="trade-id">Trade #{tradeId}</h4>
          <p className="trade-timestamp">{timestamp}</p>
        </div>
        <Badge variant={statusVariant}>{status}</Badge>
      </div>
      <div className="trade-card-body">
        <div className="trade-party">
          <span className="party-label">Seller:</span>
          <span className="party-address">{seller.slice(0, 10)}...</span>
        </div>
        <div className="trade-party">
          <span className="party-label">Buyer:</span>
          <span className="party-address">{buyer.slice(0, 10)}...</span>
        </div>
        <div className="trade-amount">
          <span className="amount-label">Amount:</span>
          <span className="amount-value">{amount} USDC</span>
        </div>
      </div>
    </Card>
  );
};
