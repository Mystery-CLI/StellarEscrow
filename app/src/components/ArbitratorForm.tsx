import { useState } from 'react';
import {
  Box,
  Button,
  CircularProgress,
  TextField,
  Typography,
  Alert,
  Paper,
} from '@mui/material';

/** Stellar G-address: starts with G, 56 alphanumeric chars total */
const STELLAR_ADDRESS_RE = /^G[A-Z2-7]{55}$/;

export interface ArbitratorFormData {
  address: string;
  fee: string; // stroops as string to avoid precision loss
}

interface Props {
  onSubmit: (data: ArbitratorFormData) => Promise<void>;
  loading?: boolean;
}

export default function ArbitratorForm({ onSubmit, loading = false }: Props) {
  const [address, setAddress] = useState('');
  const [fee, setFee] = useState('');
  const [errors, setErrors] = useState<{ address?: string; fee?: string }>({});

  function validate(): boolean {
    const next: typeof errors = {};
    if (!STELLAR_ADDRESS_RE.test(address)) {
      next.address = 'Must be a valid Stellar G-address (56 characters, starts with G)';
    }
    const feeNum = Number(fee);
    if (!fee || isNaN(feeNum) || feeNum <= 0 || !Number.isInteger(feeNum)) {
      next.fee = 'Fee must be a positive integer (in stroops)';
    }
    setErrors(next);
    return Object.keys(next).length === 0;
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!validate()) return;
    await onSubmit({ address, fee });
  }

  return (
    <Paper elevation={2} sx={{ p: 3, maxWidth: 480 }}>
      <Typography variant="h6" gutterBottom>
        Register as Arbitrator
      </Typography>
      <Box component="form" onSubmit={handleSubmit} noValidate sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <TextField
          label="Stellar Address"
          value={address}
          onChange={(e) => setAddress(e.target.value.trim())}
          error={!!errors.address}
          helperText={errors.address ?? 'Your G-address (public key)'}
          disabled={loading}
          fullWidth
          inputProps={{ maxLength: 56 }}
        />
        <TextField
          label="Service Fee (stroops)"
          type="number"
          value={fee}
          onChange={(e) => setFee(e.target.value)}
          error={!!errors.fee}
          helperText={errors.fee ?? '1 XLM = 10,000,000 stroops'}
          disabled={loading}
          fullWidth
          inputProps={{ min: 1, step: 1 }}
        />
        <Button
          type="submit"
          variant="contained"
          disabled={loading}
          startIcon={loading ? <CircularProgress size={18} color="inherit" /> : null}
        >
          {loading ? 'Submitting…' : 'Register'}
        </Button>
      </Box>
    </Paper>
  );
}
