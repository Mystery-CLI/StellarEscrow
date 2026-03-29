import React from 'react';
import { render, screen } from '@testing-library/react';
import { Input } from './Input';

describe('Input — rendering', () => {
  it('renders input with label', () => {
    render(<Input label="Email" />);
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
  });

  it('displays error message', () => {
    render(<Input label="Email" error="Invalid email" />);
    expect(screen.getByText('Invalid email')).toBeInTheDocument();
  });

  it('displays helper text when no error', () => {
    render(<Input label="Password" helperText="Min 8 characters" />);
    expect(screen.getByText('Min 8 characters')).toBeInTheDocument();
  });

  it('hides helper text when error is present', () => {
    render(<Input label="Password" error="Required" helperText="Min 8 characters" />);
    expect(screen.queryByText('Min 8 characters')).not.toBeInTheDocument();
  });

  it('disables input when disabled prop is true', () => {
    render(<Input label="Email" disabled />);
    expect(screen.getByLabelText('Email')).toBeDisabled();
  });
});

describe('Input — accessibility', () => {
  it('sets aria-invalid when error exists', () => {
    render(<Input label="Email" error="Invalid" />);
    expect(screen.getByLabelText('Email')).toHaveAttribute('aria-invalid', 'true');
  });

  it('does not set aria-invalid when no error', () => {
    render(<Input label="Email" />);
    expect(screen.getByLabelText('Email')).not.toHaveAttribute('aria-invalid');
  });

  it('links input to error via aria-describedby', () => {
    render(<Input label="Email" error="Bad email" />);
    const input = screen.getByLabelText('Email');
    const describedById = input.getAttribute('aria-describedby');
    expect(describedById).toBeTruthy();
    expect(document.getElementById(describedById!)).toHaveTextContent('Bad email');
  });

  it('links input to helper text via aria-describedby', () => {
    render(<Input label="Email" helperText="Enter your email" />);
    const input = screen.getByLabelText('Email');
    const describedById = input.getAttribute('aria-describedby');
    expect(describedById).toBeTruthy();
    expect(document.getElementById(describedById!)).toHaveTextContent('Enter your email');
  });
});

describe('Input — validation states', () => {
  it('applies input-error class when error is present', () => {
    render(<Input label="Email" error="Required" />);
    expect(screen.getByLabelText('Email')).toHaveClass('input-error');
  });

  it('applies input-valid class when valid is true', () => {
    render(<Input label="Email" valid />);
    expect(screen.getByLabelText('Email')).toHaveClass('input-valid');
  });

  it('does not apply input-valid when error is also present', () => {
    render(<Input label="Email" error="Bad" valid />);
    // error takes precedence — input-error class should be present, not input-valid
    expect(screen.getByLabelText('Email')).toHaveClass('input-error');
    expect(screen.getByLabelText('Email')).not.toHaveClass('input-valid');
  });

  it('shows error icon when error is present', () => {
    render(<Input label="Email" error="Required" />);
    // The ✕ icon is aria-hidden, query by its container class
    const wrapper = document.querySelector('.input-icon--error');
    expect(wrapper).toBeInTheDocument();
  });

  it('shows valid icon when valid is true', () => {
    render(<Input label="Email" valid />);
    const wrapper = document.querySelector('.input-icon--valid');
    expect(wrapper).toBeInTheDocument();
  });

  it('shows no icon in neutral state', () => {
    render(<Input label="Email" />);
    expect(document.querySelector('.input-icon')).not.toBeInTheDocument();
  });

  it('applies wrapper error class', () => {
    render(<Input label="Email" error="Bad" />);
    expect(document.querySelector('.input-wrapper--error')).toBeInTheDocument();
  });

  it('applies wrapper valid class', () => {
    render(<Input label="Email" valid />);
    expect(document.querySelector('.input-wrapper--valid')).toBeInTheDocument();
  });
});
