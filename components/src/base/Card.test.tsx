import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Card } from './Card';

describe('Card', () => {
  it('renders children content correctly', () => {
    render(
      <Card>
        <p>Card content</p>
      </Card>
    );
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('renders a title when provided', () => {
    render(
      <Card title="Card Title">
        <p>Content</p>
      </Card>
    );
    expect(screen.getByText('Card Title')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent('Card Title');
  });

  it('applies additional className if provided', () => {
    const { container } = render(
      <Card className="extra-class">
        <p>Content</p>
      </Card>
    );
    expect(container.firstChild).toHaveClass('card');
    expect(container.firstChild).toHaveClass('extra-class');
  });

  it('spreads other props to the container element', () => {
    render(
      <Card data-testid="card-id">
        <p>Content</p>
      </Card>
    );
    expect(screen.getByTestId('card-id')).toBeInTheDocument();
  });
});
