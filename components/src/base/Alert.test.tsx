import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Alert } from './Alert';

describe('Alert', () => {
  it('renders alert with message', () => {
    render(<Alert>This is an alert</Alert>);
    expect(screen.getByText('This is an alert')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<Alert title="Warning">Be careful</Alert>);
    expect(screen.getByText('Warning')).toBeInTheDocument();
    expect(screen.getByText('Be careful')).toBeInTheDocument();
  });

  it('applies correct type class', () => {
    render(<Alert type="error">Error message</Alert>);
    const alert = screen.getByRole('alert');
    expect(alert).toHaveClass('alert-error');
  });

  it('renders close button when onClose provided', () => {
    const onClose = jest.fn();
    render(<Alert onClose={onClose}>Closable alert</Alert>);
    const closeButton = screen.getByLabelText('Close alert');
    expect(closeButton).toBeInTheDocument();
  });

  it('calls onClose when close button clicked', () => {
    const onClose = jest.fn();
    render(<Alert onClose={onClose}>Closable alert</Alert>);
    const closeButton = screen.getByLabelText('Close alert');
    fireEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not render close button when onClose not provided', () => {
    render(<Alert>No close button</Alert>);
    expect(screen.queryByLabelText('Close alert')).not.toBeInTheDocument();
  });

  it('has role alert', () => {
    render(<Alert>Alert message</Alert>);
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });

  it('matches snapshot', () => {
    const { container } = render(<Alert type="success" title="Success">Operation completed</Alert>);
    expect(container.firstChild).toMatchSnapshot();
  });
});