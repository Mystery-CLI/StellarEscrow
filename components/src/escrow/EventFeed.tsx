import React from 'react';
import { Badge } from '../base/Badge';
import './EventFeed.css';

export interface Event {
  id: string;
  type: string;
  tradeId: string;
  timestamp: string;
  data: Record<string, any>;
}

export interface EventFeedProps {
  events: Event[];
  onEventClick?: (event: Event) => void;
}

export const EventFeed: React.FC<EventFeedProps> = ({ events, onEventClick }) => {
  const getEventVariant = (type: string): 'default' | 'success' | 'warning' | 'danger' | 'info' => {
    if (type.includes('completed') || type.includes('confirmed')) return 'success';
    if (type.includes('disputed')) return 'danger';
    if (type.includes('funded')) return 'warning';
    return 'info';
  };

  return (
    <div className="event-feed" role="log" aria-label="Event feed">
      {events.length === 0 ? (
        <p className="event-feed-empty">No events yet</p>
      ) : (
        <ul className="event-list">
          {events.map((event) => (
            <li
              key={event.id}
              className="event-item"
              onClick={() => onEventClick?.(event)}
              role="button"
              tabIndex={0}
            >
              <div className="event-header">
                <Badge variant={getEventVariant(event.type)}>{event.type}</Badge>
                <span className="event-time">{event.timestamp}</span>
              </div>
              <p className="event-trade">Trade #{event.tradeId}</p>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};
