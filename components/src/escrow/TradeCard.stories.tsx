import type { Meta, StoryObj } from '@storybook/react';
import { TradeCard } from './TradeCard';

const meta = {
  title: 'Escrow/TradeCard',
  component: TradeCard,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof TradeCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Created: Story = {
  args: {
    tradeId: '12345',
    seller: 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE',
    buyer: 'GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD',
    amount: '100.50',
    status: 'created',
    timestamp: '2024-03-25 10:30:00',
  },
};

export const Funded: Story = {
  args: {
    ...Created.args,
    status: 'funded',
  },
};

export const Completed: Story = {
  args: {
    ...Created.args,
    status: 'completed',
  },
};

export const Disputed: Story = {
  args: {
    ...Created.args,
    status: 'disputed',
  },
};
