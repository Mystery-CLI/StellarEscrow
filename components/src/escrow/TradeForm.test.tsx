import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { TradeForm } from './TradeForm';

describe('TradeForm', () => {
  it('renders form fields', () => {
    render(<TradeForm onSubmit={jest.fn()} />);
    expect(screen.getByLabelText('Seller Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Buyer Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Amount (USDC)')).toBeInTheDocument();
  });

  it('validates required fields', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} />);
    
    const submitButton = screen.getByText('Create Trade');
    fireEvent.click(submitButton);
    
    expect(onSubmit).not.toHaveBeenCalled();
    expect(screen.getByText('Seller address is required')).toBeInTheDocument();
  });

  it('submits form with valid data', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} />);
    
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: 'G123' } });
    fireEvent.change(screen.getByLabelText('Buyer Address'), { target: { value: 'G456' } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'), { target: { value: '100' } });
    
    fireEvent.click(screen.getByText('Create Trade'));
    
    expect(onSubmit).toHaveBeenCalledWith({
      seller: 'G123',
      buyer: 'G456',
      amount: '100',
      arbitrator: '',
    });
  });

  it('disables submit button when loading', () => {
    render(<TradeForm onSubmit={jest.fn()} loading={true} />);
    const submitButton = screen.getByText('Create Trade');
    expect(submitButton).toBeDisabled();
  });
});
