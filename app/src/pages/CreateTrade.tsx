import { useNavigate } from 'react-router-dom';
import { useCreateTradeMutation } from '@stellar-escrow/state';
import { TradeForm, type TradeFormData } from '@stellar-escrow/components';

export default function CreateTrade() {
  const navigate = useNavigate();
  const [createTrade, { isLoading, error }] = useCreateTradeMutation();

  const handleSubmit = async (data: TradeFormData) => {
    const result = await createTrade(data);
    if ('data' in result) {
      navigate(`/trades/${result.data.id}`);
    }
  };

  return (
    <div className="create-trade-container">
      <h1 style={{ fontSize: '1.5rem', marginBottom: '1.5rem' }}>Create Trade</h1>
      {error && <p style={{ color: 'red', marginBottom: '1rem' }}>Failed to create trade.</p>}
      <TradeForm onSubmit={handleSubmit} loading={isLoading} />
    </div>
  );
}
