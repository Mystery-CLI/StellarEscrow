import React from 'react';
import { Badge } from '../base/Badge';
import './TradeStatus.css';

export type TradeStatusType = 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';

export interface TradeStatusProps {
  status: TradeStatusType;
}

const statusConfig: Record<TradeStatusType, { label: string; variant: 'default' | 'success' | 'warning' | 'danger' | 'info' }> = {
  created: { label: 'Created', variant: 'info' },
  funded: { label: 'Funded', variant: 'warning' },
  completed: { label: 'Completed', variant: 'success' },
  disputed: { label: 'Disputed', variant: 'danger' },
  cancelled: { label: 'Cancelled', variant: 'default' },
};

export const TradeStatus: React.FC<TradeStatusProps> = ({ status }) => {
  const config = statusConfig[status];
  return <Badge variant={config.variant}>{config.label}</Badge>;
};
