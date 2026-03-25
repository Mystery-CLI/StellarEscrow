import React from 'react';
import { Input } from '../base/Input';
import { Button } from '../base/Button';
import './TradeForm.css';

export interface TradeFormProps {
  onSubmit: (data: TradeFormData) => void;
  loading?: boolean;
}

export interface TradeFormData {
  seller: string;
  buyer: string;
  amount: string;
  arbitrator?: string;
}

export const TradeForm: React.FC<TradeFormProps> = ({ onSubmit, loading = false }) => {
  const [formData, setFormData] = React.useState<TradeFormData>({
    seller: '',
    buyer: '',
    amount: '',
    arbitrator: '',
  });
  const [errors, setErrors] = React.useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};
    if (!formData.seller.trim()) newErrors.seller = 'Seller address is required';
    if (!formData.buyer.trim()) newErrors.buyer = 'Buyer address is required';
    if (!formData.amount.trim()) newErrors.amount = 'Amount is required';
    if (isNaN(Number(formData.amount))) newErrors.amount = 'Amount must be a number';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      onSubmit(formData);
    }
  };

  return (
    <form className="trade-form" onSubmit={handleSubmit}>
      <Input
        label="Seller Address"
        placeholder="G..."
        value={formData.seller}
        onChange={(e) => setFormData({ ...formData, seller: e.target.value })}
        error={errors.seller}
      />
      <Input
        label="Buyer Address"
        placeholder="G..."
        value={formData.buyer}
        onChange={(e) => setFormData({ ...formData, buyer: e.target.value })}
        error={errors.buyer}
      />
      <Input
        label="Amount (USDC)"
        type="number"
        placeholder="0.00"
        value={formData.amount}
        onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
        error={errors.amount}
      />
      <Input
        label="Arbitrator Address (Optional)"
        placeholder="G..."
        value={formData.arbitrator}
        onChange={(e) => setFormData({ ...formData, arbitrator: e.target.value })}
      />
      <Button type="submit" variant="primary" loading={loading}>
        Create Trade
      </Button>
    </form>
  );
};
