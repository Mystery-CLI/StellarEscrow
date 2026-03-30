import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { TradeStatus } from './TradeStatus';

describe('TradeStatus', () => {
  it('renders the correct label for each status', () => {
    const statuses: Array<Parameters<typeof TradeStatus>[0]['status']> = [
      'created',
      'funded',
      'completed',
      'disputed',
      'cancelled',
    ];

    statuses.forEach((status) => {
      const { unmount } = render(<TradeStatus status={status} />);
      const label = status.charAt(0).toUpperCase() + status.slice(1);
      expect(screen.getByText(label)).toBeInTheDocument();
      unmount();
    });
  });

  it('applies the correct variant via Badge', () => {
    const { container } = render(<TradeStatus status="completed" />);
    const badge = container.querySelector('.badge');
    expect(badge).toHaveClass('badge-success');
  });

  it('renders "Disputed" as danger variant', () => {
    const { container } = render(<TradeStatus status="disputed" />);
    const badge = container.querySelector('.badge');
    expect(badge).toHaveClass('badge-danger');
  });
});
