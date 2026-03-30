import React from 'react';
import { render, screen } from '@testing-library/react';
import { Card } from './Card';

describe('Card', () => {
  it('renders card with content', () => {
    render(<Card>Card content</Card>);
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<Card title="Card Title">Content</Card>);
    expect(screen.getByText('Card Title')).toBeInTheDocument();
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('does not render title when not provided', () => {
    render(<Card>No title</Card>);
    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
  });

  it('applies card class', () => {
    const { container } = render(<Card>Content</Card>);
    const card = container.querySelector('.card');
    expect(card?.className).toContain('card');
  });

  it('applies additional className', () => {
    const { container } = render(<Card className="custom-card">Content</Card>);
    const card = container.querySelector('.card');
    expect(card?.className).toContain('custom-card');
  });

  it('passes through other props', () => {
    render(<Card data-testid="card">Test</Card>);
    expect(screen.getByTestId('card')).toBeInTheDocument();
  });

  it('matches snapshot', () => {
    const { container } = render(<Card title="Test Card">Test content</Card>);
    expect(container.firstChild).toMatchSnapshot();
  });
});