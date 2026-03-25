import type { Meta, StoryObj } from '@storybook/react';
import { TradeStatus } from './TradeStatus';

const meta = {
  title: 'Escrow/TradeStatus',
  component: TradeStatus,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof TradeStatus>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Created: Story = {
  args: {
    status: 'created',
  },
};

export const Funded: Story = {
  args: {
    status: 'funded',
  },
};

export const Completed: Story = {
  args: {
    status: 'completed',
  },
};

export const Disputed: Story = {
  args: {
    status: 'disputed',
  },
};

export const Cancelled: Story = {
  args: {
    status: 'cancelled',
  },
};
