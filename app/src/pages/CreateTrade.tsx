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
    <section
      className="create-trade-container"
      role="region"
      aria-labelledby="create-trade-title"
    >
      <h1
        id="create-trade-title"
        style={{ fontSize: '1.5rem', marginBottom: '1.5rem' }}
      >
        Create Trade
      </h1>

      {error && (
        <p
          id="create-trade-error"
          role="alert"
          aria-live="assertive"
          style={{ color: 'red', marginBottom: '1rem' }}
        >
          Failed to create trade. Please try again.
        </p>
      )}

      <div
        role="form"
        aria-busy={isLoading}
        aria-describedby={error ? 'create-trade-error' : undefined}
      >
        <TradeForm onSubmit={handleSubmit} loading={isLoading} />
      </div>
    </section>
  );
}