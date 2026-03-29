import React from 'react';
import { Input } from '../base/Input';
import { Button } from '../base/Button';
import {
  validateField,
  validateTradeForm,
  isFieldValid,
  type FieldName,
  type FormErrors,
} from '../validation/tradeSchema';
import './TradeForm.css';

// ---------------------------------------------------------------------------
// Auto-save helpers
// ---------------------------------------------------------------------------

const AUTO_SAVE_KEY = 'stellar_escrow_trade_form_draft';
const AUTO_SAVE_DEBOUNCE_MS = 800;

function loadDraft(): Partial<TradeFormData> {
  try {
    const raw = localStorage.getItem(AUTO_SAVE_KEY);
    return raw ? (JSON.parse(raw) as Partial<TradeFormData>) : {};
  } catch {
    return {};
  }
}

function saveDraft(data: TradeFormData): void {
  try {
    localStorage.setItem(AUTO_SAVE_KEY, JSON.stringify(data));
  } catch {
    // localStorage unavailable (SSR, private mode) — silently ignore
  }
}

function clearDraft(): void {
  try {
    localStorage.removeItem(AUTO_SAVE_KEY);
  } catch { /* ignore */ }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface TradeFormProps {
  onSubmit: (data: TradeFormData) => void;
  loading?: boolean;
  /** Disable auto-save (useful in tests) */
  disableAutoSave?: boolean;
}

export interface TradeFormData {
  seller: string;
  buyer: string;
  amount: string;
  arbitrator: string;
}

type TouchedFields = Partial<Record<FieldName, boolean>>;
type SaveState = 'idle' | 'saving' | 'saved';

const EMPTY_FORM: TradeFormData = { seller: '', buyer: '', amount: '', arbitrator: '' };

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TradeForm: React.FC<TradeFormProps> = ({
  onSubmit,
  loading = false,
  disableAutoSave = false,
}) => {
  const draft = React.useMemo(() => (disableAutoSave ? {} : loadDraft()), [disableAutoSave]);

  const [formData, setFormData] = React.useState<TradeFormData>({
    ...EMPTY_FORM,
    ...draft,
  });
  const [errors, setErrors] = React.useState<FormErrors>({});
  const [touched, setTouched] = React.useState<TouchedFields>({});
  const [saveState, setSaveState] = React.useState<SaveState>('idle');
  const [submitAttempted, setSubmitAttempted] = React.useState(false);

  const autoSaveTimer = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  // ── Validation context (seller needed for buyer cross-check) ──────────────
  const validationCtx = React.useMemo(
    () => ({ seller: formData.seller, buyer: formData.buyer }),
    [formData.seller, formData.buyer]
  );

  // ── Derive per-field error — only show after field is touched or submit attempted ──
  const visibleErrors = React.useMemo<FormErrors>(() => {
    const result: FormErrors = {};
    const fields: FieldName[] = ['seller', 'buyer', 'amount', 'arbitrator'];
    for (const field of fields) {
      if (touched[field] || submitAttempted) {
        const err = validateField(field, formData[field], validationCtx);
        if (err) result[field] = err;
      }
    }
    return result;
  }, [formData, touched, submitAttempted, validationCtx]);

  // ── Auto-save with debounce ───────────────────────────────────────────────
  React.useEffect(() => {
    if (disableAutoSave) return;

    // Don't save if form is completely empty
    const hasContent = (Object.values(formData) as string[]).some((v) => v.trim() !== '');
    if (!hasContent) return;

    if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);

    setSaveState('saving');
    autoSaveTimer.current = setTimeout(() => {
      saveDraft(formData);
      setSaveState('saved');
      // Reset to idle after 2s
      setTimeout(() => setSaveState('idle'), 2000);
    }, AUTO_SAVE_DEBOUNCE_MS);

    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, [formData, disableAutoSave]);

  // ── Field change handler ──────────────────────────────────────────────────
  const handleChange = (field: FieldName) => (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setFormData((prev: TradeFormData) => ({ ...prev, [field]: value }));

    // Real-time validation: only show errors once the field has been touched
    if (touched[field]) {
      const err = validateField(field, value, { ...validationCtx, [field]: value });
      setErrors((prev: FormErrors) => ({ ...prev, [field]: err ?? undefined }));
    }
  };

  // ── Blur handler — marks field as touched and validates immediately ───────
  const handleBlur = (field: FieldName) => () => {
    setTouched((prev: TouchedFields) => ({ ...prev, [field]: true }));
    const err = validateField(field, formData[field], validationCtx);
    setErrors((prev: FormErrors) => ({ ...prev, [field]: err ?? undefined }));
  };

  // ── Submit ────────────────────────────────────────────────────────────────
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitAttempted(true);

    const allErrors = validateTradeForm(formData);
    setErrors(allErrors);

    if (Object.keys(allErrors).length === 0) {
      clearDraft();
      setSaveState('idle');
      onSubmit(formData);
    }
  };

  // ── Helpers ───────────────────────────────────────────────────────────────
  const fieldValid = (field: FieldName) =>
    (touched[field] || submitAttempted) && isFieldValid(field, formData[field], validationCtx);

  return (
    <form className="trade-form" onSubmit={handleSubmit} noValidate>

      {/* Auto-save indicator */}
      {!disableAutoSave && (
        <div
          className={`trade-form__save-indicator trade-form__save-indicator--${saveState}`}
          aria-live="polite"
          aria-atomic="true"
        >
          {saveState === 'saving' && <><span className="trade-form__save-spinner" aria-hidden="true" /> Saving draft…</>}
          {saveState === 'saved'  && <><span aria-hidden="true">✓</span> Draft saved</>}
        </div>
      )}

      <Input
        label="Seller Address"
        placeholder="GABC…XYZ (56 characters)"
        value={formData.seller}
        onChange={handleChange('seller')}
        onBlur={handleBlur('seller')}
        error={visibleErrors.seller}
        valid={fieldValid('seller')}
        autoComplete="off"
        spellCheck={false}
      />

      <Input
        label="Buyer Address"
        placeholder="GABC…XYZ (56 characters)"
        value={formData.buyer}
        onChange={handleChange('buyer')}
        onBlur={handleBlur('buyer')}
        error={visibleErrors.buyer}
        valid={fieldValid('buyer')}
        autoComplete="off"
        spellCheck={false}
      />

      <Input
        label="Amount (USDC)"
        type="number"
        placeholder="0.00"
        min="0.000001"
        step="any"
        value={formData.amount}
        onChange={handleChange('amount')}
        onBlur={handleBlur('amount')}
        error={visibleErrors.amount}
        valid={fieldValid('amount')}
      />

      <Input
        label="Arbitrator Address (Optional)"
        placeholder="GABC…XYZ (56 characters)"
        value={formData.arbitrator}
        onChange={handleChange('arbitrator')}
        onBlur={handleBlur('arbitrator')}
        error={visibleErrors.arbitrator}
        valid={fieldValid('arbitrator')}
        helperText={!visibleErrors.arbitrator ? 'Leave blank to trade without an arbitrator' : undefined}
        autoComplete="off"
        spellCheck={false}
      />

      <Button type="submit" variant="primary" loading={loading}>
        Create Trade
      </Button>
    </form>
  );
};
