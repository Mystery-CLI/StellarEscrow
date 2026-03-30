import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { EventFeed, Event } from './EventFeed';

describe('EventFeed', () => {
  const mockEvents: Event[] = [
    {
      id: '1',
      type: 'trade_created',
      tradeId: '123',
      timestamp: '2024-03-25 10:30:00',
      data: { amount: '100' }
    },
    {
      id: '2',
      type: 'trade_completed',
      tradeId: '124',
      timestamp: '2024-03-25 11:00:00',
      data: { amount: '200' }
    }
  ];

  it('renders empty state when no events', () => {
    render(<EventFeed events={[]} />);
    expect(screen.getByText('No events yet')).toBeInTheDocument();
  });

  it('renders events list', () => {
    render(<EventFeed events={mockEvents} />);
    expect(screen.getByText('trade_created')).toBeInTheDocument();
    expect(screen.getByText('trade_completed')).toBeInTheDocument();
    expect(screen.getByText('Trade #123')).toBeInTheDocument();
    expect(screen.getByText('Trade #124')).toBeInTheDocument();
  });

  it('displays timestamps', () => {
    render(<EventFeed events={mockEvents} />);
    expect(screen.getByText('2024-03-25 10:30:00')).toBeInTheDocument();
    expect(screen.getByText('2024-03-25 11:00:00')).toBeInTheDocument();
  });

  it('applies correct badge variants', () => {
    render(<EventFeed events={mockEvents} />);
    const createdBadge = screen.getByText('trade_created');
    const completedBadge = screen.getByText('trade_completed');
    expect(createdBadge).toHaveClass('badge-info');
    expect(completedBadge).toHaveClass('badge-success');
  });

  it('calls onEventClick when event clicked', () => {
    const onEventClick = jest.fn();
    render(<EventFeed events={mockEvents} onEventClick={onEventClick} />);
    const eventItem = screen.getByText('Trade #123').closest('.event-item');
    fireEvent.click(eventItem!);
    expect(onEventClick).toHaveBeenCalledWith(mockEvents[0]);
  });

  it('has correct accessibility attributes', () => {
    render(<EventFeed events={mockEvents} />);
    expect(screen.getByRole('log')).toBeInTheDocument();
    expect(screen.getByLabelText('Event feed')).toBeInTheDocument();
  });

  it('makes event items keyboard accessible', () => {
    render(<EventFeed events={mockEvents} />);
    const eventItems = screen.getAllByRole('button');
    expect(eventItems).toHaveLength(2);
    eventItems.forEach((item: HTMLElement) => {
      expect(item).toHaveAttribute('tabIndex', '0');
    });
  });

  it('matches snapshot', () => {
    const { container } = render(<EventFeed events={mockEvents} />);
    expect(container.firstChild).toMatchSnapshot();
  });
});