import { createSlice, PayloadAction } from '@reduxjs/toolkit';

export type SupportedLocale = 'en' | 'fr' | 'ar' | 'es';

// Languages that require right-to-left layout
const RTL_LOCALES: ReadonlySet<SupportedLocale> = new Set(['ar']);

export interface LocaleState {
  locale: SupportedLocale;
  isRTL: boolean;
  currency: string;
}

const LOCALE_CURRENCY: Record<SupportedLocale, string> = {
  en: 'USD',
  fr: 'EUR',
  ar: 'SAR',
  es: 'MXN',
};

const stored = (typeof localStorage !== 'undefined'
  ? localStorage.getItem('stellar_escrow_locale')
  : null) as SupportedLocale | null;

const defaultLocale: SupportedLocale =
  stored && ['en', 'fr', 'ar', 'es'].includes(stored) ? stored : 'en';

const initialState: LocaleState = {
  locale: defaultLocale,
  isRTL: RTL_LOCALES.has(defaultLocale),
  currency: LOCALE_CURRENCY[defaultLocale],
};

const localeSlice = createSlice({
  name: 'locale',
  initialState,
  reducers: {
    setLocale: (state, action: PayloadAction<SupportedLocale>) => {
      state.locale = action.payload;
      state.isRTL = RTL_LOCALES.has(action.payload);
      state.currency = LOCALE_CURRENCY[action.payload];
    },
  },
});

export const { setLocale } = localeSlice.actions;
export default localeSlice.reducer;
