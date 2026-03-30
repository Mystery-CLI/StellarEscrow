import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { EventFeed, Event } from './EventFeed';

const mockEvents: Event[] = [
  {
    id: '1',
    type: 'trade_created',
    tradeId: '12345',
    timestamp: '2024-03-25 10:00:00',
    data: {},
  },
  {
    id: '2',
    type: 'trade_funded',
    tradeId: '12345',
    timestamp: '2024-03-25 10:05:00',
    data: {},
  },
  {
    id: '3',
    type: 'trade_completed',
    tradeId: '12345',
    timestamp: '2024-03-25 10:10:00',
    data: {},
  },
];

describe('EventFeed', () => {
  it('renders an empty state message when there are no events', () => {
    render(<EventFeed events={[]} />);
    expect(screen.getByText('No events yet')).toBeInTheDocument();
  });

  it('renders a list of events when provided', () => {
    render(<EventFeed events={mockEvents} />);
    expect(screen.getAllByRole('button')).toHaveLength(3);
    expect(screen.getByText('trade_created')).toBeInTheDocument();
    expect(screen.getByText('trade_funded')).toBeInTheDocument();
    expect(screen.getByText('trade_completed')).toBeInTheDocument();
    expect(screen.getAllByText('Trade #12345')).toHaveLength(3);
  });

  it('calls onEventClick when an event item is clicked', () => {
    const onEventClickMock = jest.fn();
    render(<EventFeed events={mockEvents} onEventClick={onEventClickMock} />);
    const eventItem = screen.getAllByRole('button')[0];
    fireEvent.click(eventItem);
    expect(onEventClickMock).toHaveBeenCalledWith(mockEvents[0]);
  });

  it('applies the correct variant based on event type', () => {
    const { container } = render(<EventFeed events={mockEvents} />);
    const successBadge = container.querySelector('.badge-success');
    expect(successBadge).toHaveTextContent('trade_completed');

    const warningBadge = container.querySelector('.badge-warning');
    expect(warningBadge).toHaveTextContent('trade_funded');
  });

  it('has correct accessibility attributes', () => {
    render(<EventFeed events={mockEvents} />);
    expect(screen.getByRole('log', { name: 'Event feed' })).toBeInTheDocument();
  });
});
