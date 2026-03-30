import { useState } from 'react';
import { Alert, Box, Paper, Snackbar, Typography } from '@mui/material';
import ArbitratorForm, { type ArbitratorFormData } from '../components/ArbitratorForm';

interface RegisteredProfile {
  address: string;
  fee: string;
  registeredAt: string;
}

/**
 * Calls register_arbitrator_self on the Soroban contract via Freighter.
 * Falls back gracefully when Freighter is not installed.
 */
async function submitRegistration(data: ArbitratorFormData): Promise<void> {
  // Dynamic import so the app still loads without Freighter installed
  const freighter = await import('@stellar/freighter-api').catch(() => null);
  if (!freighter) {
    throw new Error('Freighter wallet extension not found. Please install it to continue.');
  }

  const isConnected = await freighter.isConnected();
  if (!isConnected) {
    throw new Error('Freighter is not connected. Please unlock your wallet.');
  }

  // Build the contract invocation XDR.
  // In a real integration this would use @stellar/stellar-sdk to build the
  // transaction and then sign + submit it. Here we demonstrate the shape.
  const { signTransaction } = freighter;

  // Placeholder: replace CONTRACT_ID and NETWORK_PASSPHRASE with env vars.
  const contractId = import.meta.env.VITE_CONTRACT_ID ?? '';
  const networkPassphrase = import.meta.env.VITE_NETWORK_PASSPHRASE ?? 'Test SDF Network ; September 2015';

  if (!contractId) {
    throw new Error('VITE_CONTRACT_ID is not configured.');
  }

  // Build a minimal transaction envelope XDR calling register_arbitrator_self.
  // Full SDK integration would use SorobanRpc + TransactionBuilder here.
  const txXdr = buildRegisterArbitratorTx(contractId, data.address, BigInt(data.fee), networkPassphrase);

  await signTransaction(txXdr, { networkPassphrase });
  // After signing, submit via Soroban RPC (omitted — depends on deployment env).
}

/** Stub: replace with real @stellar/stellar-sdk transaction builder. */
function buildRegisterArbitratorTx(
  _contractId: string,
  _arbitrator: string,
  _fee: bigint,
  _networkPassphrase: string,
): string {
  // Return a placeholder XDR string; real impl uses stellar-sdk TransactionBuilder.
  return 'PLACEHOLDER_XDR';
}

export default function ArbitratorRegister() {
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<RegisteredProfile | null>(null);
  const [toastOpen, setToastOpen] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  async function handleSubmit(data: ArbitratorFormData) {
    setLoading(true);
    setErrorMsg(null);
    try {
      await submitRegistration(data);
      setProfile({
        address: data.address,
        fee: data.fee,
        registeredAt: new Date().toUTCString(),
      });
      setToastOpen(true);
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : 'Registration failed.');
    } finally {
      setLoading(false);
    }
  }

  return (
    <Box sx={{ maxWidth: 560, mx: 'auto', mt: 4, display: 'flex', flexDirection: 'column', gap: 3 }}>
      <Typography variant="h5" fontWeight={700}>
        Arbitrator Registration
      </Typography>

      {errorMsg && <Alert severity="error">{errorMsg}</Alert>}

      <ArbitratorForm onSubmit={handleSubmit} loading={loading} />

      {profile && (
        <Paper elevation={1} sx={{ p: 3 }}>
          <Typography variant="subtitle1" fontWeight={600} gutterBottom>
            Registered Profile
          </Typography>
          <Typography variant="body2" color="text.secondary">Address</Typography>
          <Typography variant="body1" sx={{ wordBreak: 'break-all', mb: 1 }}>{profile.address}</Typography>
          <Typography variant="body2" color="text.secondary">Service Fee</Typography>
          <Typography variant="body1" sx={{ mb: 1 }}>{Number(profile.fee).toLocaleString()} stroops</Typography>
          <Typography variant="body2" color="text.secondary">Registered At</Typography>
          <Typography variant="body1">{profile.registeredAt}</Typography>
        </Paper>
      )}

      <Snackbar
        open={toastOpen}
        autoHideDuration={5000}
        onClose={() => setToastOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert severity="success" onClose={() => setToastOpen(false)}>
          Successfully registered as an arbitrator!
        </Alert>
      </Snackbar>
    </Box>
  );
}
