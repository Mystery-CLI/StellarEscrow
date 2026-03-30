import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Alert } from './Alert';

describe('Alert', () => {
  it('renders correctly with children', () => {
    render(<Alert>Alert message</Alert>);
    expect(screen.getByText('Alert message')).toBeInTheDocument();
  });

  it('renders a title if provided', () => {
    render(<Alert title="Warning">Be careful!</Alert>);
    expect(screen.getByText('Warning')).toBeInTheDocument();
    expect(screen.getByText('Be careful!')).toBeInTheDocument();
  });

  it('applies the correct alert type class', () => {
    const { container } = render(<Alert type="error">Something went wrong</Alert>);
    expect(container.firstChild).toHaveClass('alert-error');
  });

  it('calls onClose when the close button is clicked', () => {
    const onCloseMock = jest.fn();
    render(<Alert onClose={onCloseMock}>Message</Alert>);
    const closeButton = screen.getByLabelText('Close alert');
    fireEvent.click(closeButton);
    expect(onCloseMock).toHaveBeenCalledTimes(1);
  });

  it('does not render a close button if onClose is not provided', () => {
    render(<Alert>Message</Alert>);
    const closeButton = screen.queryByLabelText('Close alert');
    expect(closeButton).not.toBeInTheDocument();
  });

  it('has role="alert"', () => {
    render(<Alert>Accessibility test</Alert>);
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });
});
