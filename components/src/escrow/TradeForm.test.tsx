import React from 'react';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TradeForm } from './TradeForm';

// Valid 56-char Stellar addresses (G + 55 base-32 uppercase chars)
const VALID_SELLER = 'GBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQP';
const VALID_BUYER  = 'GGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM';

beforeEach(() => {
  localStorage.clear();
  jest.useFakeTimers();
});

afterEach(() => {
  jest.runOnlyPendingTimers();
  jest.useRealTimers();
});

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

describe('TradeForm — rendering', () => {
  it('renders all form fields', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    expect(screen.getByLabelText('Seller Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Buyer Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Amount (USDC)')).toBeInTheDocument();
    expect(screen.getByLabelText('Arbitrator Address (Optional)')).toBeInTheDocument();
  });

  it('disables submit button when loading', () => {
    render(<TradeForm onSubmit={jest.fn()} loading disableAutoSave />);
    expect(screen.getByRole('button', { name: /create trade/i })).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// Submit-time validation
// ---------------------------------------------------------------------------

describe('TradeForm — submit-time validation', () => {
  it('shows all required errors on empty submit', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(screen.getByText('Seller address is required')).toBeInTheDocument();
    expect(screen.getByText('Buyer address is required')).toBeInTheDocument();
    expect(screen.getByText('Amount is required')).toBeInTheDocument();
  });

  it('does not call onSubmit when validation fails', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('calls onSubmit with valid data', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),  { target: { value: '100' } });

    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(onSubmit).toHaveBeenCalledWith({
      seller: VALID_SELLER,
      buyer: VALID_BUYER,
      amount: '100',
      arbitrator: '',
    });
  });
});

// ---------------------------------------------------------------------------
// Stellar address validation
// ---------------------------------------------------------------------------

describe('TradeForm — Stellar address validation', () => {
  it('shows address format error for invalid seller after blur', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: 'G123' } });
    fireEvent.blur(screen.getByLabelText('Seller Address'));
    expect(screen.getByText(/valid Stellar address/i)).toBeInTheDocument();
  });

  it('shows error when buyer equals seller', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_SELLER } });
    fireEvent.blur(screen.getByLabelText('Buyer Address'));
    expect(screen.getByText(/buyer and seller must be different/i)).toBeInTheDocument();
  });

  it('accepts a valid arbitrator address', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);

    fireEvent.change(screen.getByLabelText('Seller Address'),                { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),                 { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),                 { target: { value: '50' } });
    fireEvent.change(screen.getByLabelText('Arbitrator Address (Optional)'), { target: { value: VALID_SELLER } });

    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));
    expect(onSubmit).toHaveBeenCalled();
  });

  it('rejects an invalid arbitrator address', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Arbitrator Address (Optional)'), { target: { value: 'bad' } });
    fireEvent.blur(screen.getByLabelText('Arbitrator Address (Optional)'));
    expect(screen.getByText(/valid Stellar address/i)).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Real-time validation
// ---------------------------------------------------------------------------

describe('TradeForm — real-time validation', () => {
  it('does not show errors before any field is touched', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('shows error immediately after blur on empty required field', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.blur(screen.getByLabelText('Seller Address'));
    expect(screen.getByText('Seller address is required')).toBeInTheDocument();
  });

  it('clears error when field becomes valid after being touched', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    const input = screen.getByLabelText('Seller Address');

    // Touch the field — error appears
    fireEvent.blur(input);
    expect(screen.getByText('Seller address is required')).toBeInTheDocument();

    // Type a valid value — error should clear on next change
    // We fire blur first to ensure touched state is committed, then change
    fireEvent.change(input, { target: { value: VALID_SELLER } });
    fireEvent.blur(input);
    expect(screen.queryByText('Seller address is required')).not.toBeInTheDocument();
    expect(screen.queryByText(/valid Stellar address/i)).not.toBeInTheDocument();
  });

  it('shows amount error for negative value', async () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    const input = screen.getByLabelText('Amount (USDC)');
    act(() => {
      fireEvent.change(input, { target: { value: '-1' } });
    });
    act(() => {
      fireEvent.blur(input);
    });
    await waitFor(() => {
      expect(screen.getByText(/positive number/i)).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// Auto-save
// ---------------------------------------------------------------------------

describe('TradeForm — auto-save', () => {
  it('saves draft to localStorage after debounce', () => {
    render(<TradeForm onSubmit={jest.fn()} />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });

    act(() => {
      jest.advanceTimersByTime(900);
    });

    const raw = localStorage.getItem('stellar_escrow_trade_form_draft');
    expect(raw).not.toBeNull();
    const draft = JSON.parse(raw!);
    expect(draft.seller).toBe(VALID_SELLER);
  });

  it('restores draft from localStorage on mount', () => {
    localStorage.setItem(
      'stellar_escrow_trade_form_draft',
      JSON.stringify({ seller: VALID_SELLER, buyer: '', amount: '42', arbitrator: '' })
    );

    render(<TradeForm onSubmit={jest.fn()} />);

    expect((screen.getByLabelText('Seller Address') as HTMLInputElement).value).toBe(VALID_SELLER);
    expect((screen.getByLabelText('Amount (USDC)') as HTMLInputElement).value).toBe('42');
  });

  it('clears draft on successful submit', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),  { target: { value: '10' } });
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(localStorage.getItem('stellar_escrow_trade_form_draft')).toBeNull();
  });

  it('does not save when disableAutoSave is true', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    act(() => {
      jest.advanceTimersByTime(1000);
    });
    expect(localStorage.getItem('stellar_escrow_trade_form_draft')).toBeNull();
  });

  it('matches snapshot', () => {
    const { container } = render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    expect(container.firstChild).toMatchSnapshot();
  });
});
