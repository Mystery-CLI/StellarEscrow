import React from 'react';
import { render, screen } from '@testing-library/react';
import { TradeStatus } from './TradeStatus';

describe('TradeStatus', () => {
  it('renders created status', () => {
    render(<TradeStatus status="created" />);
    expect(screen.getByText('Created')).toBeInTheDocument();
  });

  it('renders funded status', () => {
    render(<TradeStatus status="funded" />);
    expect(screen.getByText('Funded')).toBeInTheDocument();
  });

  it('renders completed status', () => {
    render(<TradeStatus status="completed" />);
    expect(screen.getByText('Completed')).toBeInTheDocument();
  });

  it('renders disputed status', () => {
    render(<TradeStatus status="disputed" />);
    expect(screen.getByText('Disputed')).toBeInTheDocument();
  });

  it('renders cancelled status', () => {
    render(<TradeStatus status="cancelled" />);
    expect(screen.getByText('Cancelled')).toBeInTheDocument();
  });

  it('applies correct variant for created', () => {
    render(<TradeStatus status="created" />);
    const badge = screen.getByText('Created');
    expect(badge).toHaveClass('badge-info');
  });

  it('applies correct variant for funded', () => {
    render(<TradeStatus status="funded" />);
    const badge = screen.getByText('Funded');
    expect(badge).toHaveClass('badge-warning');
  });

  it('applies correct variant for completed', () => {
    render(<TradeStatus status="completed" />);
    const badge = screen.getByText('Completed');
    expect(badge).toHaveClass('badge-success');
  });

  it('applies correct variant for disputed', () => {
    render(<TradeStatus status="disputed" />);
    const badge = screen.getByText('Disputed');
    expect(badge).toHaveClass('badge-danger');
  });

  it('applies correct variant for cancelled', () => {
    render(<TradeStatus status="cancelled" />);
    const badge = screen.getByText('Cancelled');
    expect(badge).toHaveClass('badge-default');
  });

  it('matches snapshot', () => {
    const { container } = render(<TradeStatus status="completed" />);
    expect(container.firstChild).toMatchSnapshot();
  });
});