import type { Meta, StoryObj } from '@storybook/react';
import { TradeForm } from './TradeForm';

const meta = {
  title: 'Escrow/TradeForm',
  component: TradeForm,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof TradeForm>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    onSubmit: (data) => console.log('Form submitted:', data),
  },
};

export const Loading: Story = {
  args: {
    loading: true,
    onSubmit: (data) => console.log('Form submitted:', data),
  },
};
