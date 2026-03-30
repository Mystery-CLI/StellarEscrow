import { derived, writable } from 'svelte/store';
import { browser } from '$app/environment';
import { register, init, getLocaleFromNavigator, locale, t, waitLocale } from 'svelte-i18n';

// ── Supported locales ──────────────────────────────────────────────────────────
export const SUPPORTED_LOCALES = ['en', 'es', 'fr', 'pt', 'ar', 'zh'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

/** Locales that require right-to-left text direction */
export const RTL_LOCALES = new Set<SupportedLocale>(['ar']);

export interface LocaleMeta {
  code: SupportedLocale;
  label: string;
  nativeLabel: string;
  dir: 'ltr' | 'rtl';
}

export const LOCALE_META: LocaleMeta[] = [
  { code: 'en', label: 'English',    nativeLabel: 'English',    dir: 'ltr' },
  { code: 'es', label: 'Spanish',    nativeLabel: 'Español',    dir: 'ltr' },
  { code: 'fr', label: 'French',     nativeLabel: 'Français',   dir: 'ltr' },
  { code: 'pt', label: 'Portuguese', nativeLabel: 'Português',  dir: 'ltr' },
  { code: 'ar', label: 'Arabic',     nativeLabel: 'العربية',    dir: 'rtl' },
  { code: 'zh', label: 'Chinese',    nativeLabel: '中文',        dir: 'ltr' },
];

const STORAGE_KEY = 'stellar-escrow-locale';

// ── Register locale loaders (lazy) ─────────────────────────────────────────────
register('en', () => import('./locales/en.json'));
register('es', () => import('./locales/es.json'));
register('fr', () => import('./locales/fr.json'));
register('pt', () => import('./locales/pt.json'));
register('ar', () => import('./locales/ar.json'));
register('zh', () => import('./locales/zh.json'));

// ── Determine initial locale ───────────────────────────────────────────────────
function getInitialLocale(): SupportedLocale {
  if (browser) {
    const saved = localStorage.getItem(STORAGE_KEY) as SupportedLocale | null;
    if (saved && SUPPORTED_LOCALES.includes(saved)) return saved;

    const nav = getLocaleFromNavigator() ?? 'en';
    // Match language code only (strip region, e.g. 'en-US' → 'en')
    const base = nav.split('-')[0] as SupportedLocale;
    if (SUPPORTED_LOCALES.includes(base)) return base;
  }
  return 'en';
}

// ── Initialise svelte-i18n ─────────────────────────────────────────────────────
export function setupI18n() {
  const initialLocale = getInitialLocale();
  init({
    fallbackLocale: 'en',
    initialLocale,
  });
}

// ── Reactive RTL store ─────────────────────────────────────────────────────────
export const isRTL = derived(locale, ($locale) =>
  RTL_LOCALES.has(($locale ?? 'en') as SupportedLocale)
);

export const textDir = derived(isRTL, ($rtl) => ($rtl ? 'rtl' : 'ltr') as 'rtl' | 'ltr');

// ── Locale switcher ────────────────────────────────────────────────────────────
export function setLocale(newLocale: SupportedLocale) {
  locale.set(newLocale);
  if (browser) {
    localStorage.setItem(STORAGE_KEY, newLocale);
    // Update <html dir> and <html lang> immediately
    document.documentElement.lang = newLocale;
    document.documentElement.dir = RTL_LOCALES.has(newLocale) ? 'rtl' : 'ltr';
  }
}

// ── Currency formatting ────────────────────────────────────────────────────────

/** Map locale codes to BCP-47 tags for Intl.NumberFormat */
const LOCALE_BCP47: Record<SupportedLocale, string> = {
  en: 'en-US',
  es: 'es-ES',
  fr: 'fr-FR',
  pt: 'pt-BR',
  ar: 'ar-SA',
  zh: 'zh-CN',
};

/**
 * Format a USDC micro-unit amount (7 decimal places) as a locale-aware string.
 * E.g. 10_000_000n → "$10.00" in en-US, "10,00 $US" in fr-FR etc.
 */
export function formatUsdc(microAmount: bigint, localeCode: SupportedLocale = 'en'): string {
  const amount = Number(microAmount) / 1_000_000;
  const bcp47 = LOCALE_BCP47[localeCode] ?? 'en-US';
  return new Intl.NumberFormat(bcp47, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount);
}

/**
 * Format a plain number as a locale-aware decimal.
 */
export function formatNumber(value: number, localeCode: SupportedLocale = 'en'): string {
  const bcp47 = LOCALE_BCP47[localeCode] ?? 'en-US';
  return new Intl.NumberFormat(bcp47).format(value);
}

// Re-export svelte-i18n primitives for convenience
export { t, locale, waitLocale };
