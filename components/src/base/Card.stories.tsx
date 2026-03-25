import type { Meta, StoryObj } from '@storybook/react';
import { Card } from './Card';

const meta = {
  title: 'Base/Card',
  component: Card,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: 'Card Title',
    children: 'This is the card content. You can put any React elements here.',
  },
};

export const WithoutTitle: Story = {
  args: {
    children: 'Card content without a title.',
  },
};

export const WithComplexContent: Story = {
  args: {
    title: 'Trade Information',
    children: (
      <div>
        <p><strong>Trade ID:</strong> 12345</p>
        <p><strong>Amount:</strong> 100.50 USDC</p>
        <p><strong>Status:</strong> Completed</p>
      </div>
    ),
  },
};
