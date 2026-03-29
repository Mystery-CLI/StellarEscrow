import type { Meta, StoryObj } from '@storybook/react';
import { Alert } from './Alert';

const meta = {
  title: 'Base/Alert',
  component: Alert,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Success: Story = {
  args: {
    type: 'success',
    title: 'Success',
    children: 'Your action was completed successfully.',
  },
};

export const Error: Story = {
  args: {
    type: 'error',
    title: 'Error',
    children: 'Something went wrong. Please try again.',
  },
};

export const Warning: Story = {
  args: {
    type: 'warning',
    title: 'Warning',
    children: 'Please review this information before proceeding.',
  },
};

export const Info: Story = {
  args: {
    type: 'info',
    title: 'Information',
    children: 'This is an informational message.',
  },
};

export const WithClose: Story = {
  args: {
    type: 'success',
    title: 'Success',
    children: 'You can close this alert.',
    onClose: () => console.log('Alert closed'),
  },
};
