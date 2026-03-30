import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Badge } from './Badge';

describe('Badge', () => {
  it('renders children content correctly', () => {
    render(<Badge>Completed</Badge>);
    expect(screen.getByText('Completed')).toBeInTheDocument();
  });

  it('applies the correct variant class', () => {
    const { container } = render(<Badge variant="success">Success</Badge>);
    expect(container.firstChild).toHaveClass('badge-success');
  });

  it('defaults to the default variant if none is specified', () => {
    const { container } = render(<Badge>Default</Badge>);
    expect(container.firstChild).toHaveClass('badge-default');
  });

  it('applies additional className if provided', () => {
    const { container } = render(<Badge className="custom-badge">Custom</Badge>);
    expect(container.firstChild).toHaveClass('custom-badge');
  });

  it('spreads other props to the span element', () => {
    render(<Badge data-testid="badge-id">Test</Badge>);
    expect(screen.getByTestId('badge-id')).toBeInTheDocument();
  });
});
