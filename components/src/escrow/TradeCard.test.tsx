import React from 'react';
import { render, screen } from '@testing-library/react';
import { TradeCard } from './TradeCard';

describe('TradeCard', () => {
  const defaultProps = {
    tradeId: '123',
    seller: 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE',
    buyer: 'GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD',
    amount: '100.50',
    status: 'created' as const,
    timestamp: '2024-03-25 10:30:00',
  };

  it('renders trade card with trade ID', () => {
    render(<TradeCard {...defaultProps} />);
    expect(screen.getByText('Trade #123')).toBeInTheDocument();
  });

  it('displays status badge', () => {
    render(<TradeCard {...defaultProps} />);
    expect(screen.getByText('created')).toBeInTheDocument();
  });

  it('displays amount', () => {
    render(<TradeCard {...defaultProps} />);
    expect(screen.getByText('100.50 USDC')).toBeInTheDocument();
  });

  it('truncates addresses', () => {
    render(<TradeCard {...defaultProps} />);
    expect(screen.getByText('GBUQWP3BOU...')).toBeInTheDocument();
  });

  it('matches snapshot', () => {
    const { container } = render(<TradeCard {...defaultProps} />);
    expect(container.firstChild).toMatchSnapshot();
  });
});
