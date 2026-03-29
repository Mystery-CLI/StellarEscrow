import type { Meta, StoryObj } from '@storybook/react';
import { EventFeed } from './EventFeed';

const meta = {
  title: 'Escrow/EventFeed',
  component: EventFeed,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof EventFeed>;

export default meta;
type Story = StoryObj<typeof meta>;

const mockEvents = [
  {
    id: '1',
    type: 'trade_created',
    tradeId: '12345',
    timestamp: '2024-03-25 10:30:00',
    data: {},
  },
  {
    id: '2',
    type: 'trade_funded',
    tradeId: '12345',
    timestamp: '2024-03-25 10:35:00',
    data: {},
  },
  {
    id: '3',
    type: 'trade_completed',
    tradeId: '12345',
    timestamp: '2024-03-25 10:40:00',
    data: {},
  },
];

export const Default: Story = {
  args: {
    events: mockEvents,
  },
};

export const Empty: Story = {
  args: {
    events: [],
  },
};

export const WithDispute: Story = {
  args: {
    events: [
      ...mockEvents,
      {
        id: '4',
        type: 'trade_disputed',
        tradeId: '12345',
        timestamp: '2024-03-25 10:45:00',
        data: {},
      },
    ],
  },
};
