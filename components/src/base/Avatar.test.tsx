import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Avatar } from './Avatar';

describe('Avatar', () => {
  it('renders initials fallback when no image source is provided', () => {
    render(<Avatar fallback="John Doe" />);
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('renders a question mark if no fallback and no src are provided', () => {
    render(<Avatar />);
    expect(screen.getByText('?')).toBeInTheDocument();
  });

  it('renders an image when src is provided', () => {
    const src = 'https://example.com/avatar.jpg';
    render(<Avatar src={src} alt="Test Avatar" />);
    const img = screen.getByRole('img');
    expect(img).toHaveAttribute('src', src);
    expect(img).toHaveAttribute('alt', 'Test Avatar');
  });

  it('falls back to initials when image fails to load', () => {
    render(<Avatar src="invalid.jpg" fallback="John Doe" />);
    const img = screen.getByRole('img');
    fireEvent.error(img);
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('applies the correct size and variant classes', () => {
    const { container } = render(<Avatar size="lg" variant="square" />);
    const avatar = container.firstChild;
    expect(avatar).toHaveClass('avatar-lg');
    expect(avatar).toHaveClass('avatar-square');
  });
});
