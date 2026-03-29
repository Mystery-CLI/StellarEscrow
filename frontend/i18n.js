/**
 * StellarEscrow i18n module
 * Supports language switching, RTL layouts, and currency localization.
 */

import en from './locales/en.js';
import fr from './locales/fr.js';
import ar from './locales/ar.js';
import es from './locales/es.js';

const LOCALES = { en, fr, ar, es };

// Languages that use right-to-left text direction
const RTL_LANGS = new Set(['ar', 'he', 'fa', 'ur']);

// Map locale → preferred currency code for display formatting
const LOCALE_CURRENCY = {
  en: 'USD',
  fr: 'EUR',
  ar: 'SAR',
  es: 'MXN',
};

const STORAGE_KEY = 'stellar_escrow_locale';

let currentLocale = localStorage.getItem(STORAGE_KEY) || navigator.language.split('-')[0] || 'en';
if (!LOCALES[currentLocale]) currentLocale = 'en';

/**
 * Resolve a dot-separated key path against the current translations.
 * Falls back to English, then the raw key.
 */
function t(key) {
  const parts = key.split('.');
  const resolve = (obj) => parts.reduce((o, k) => (o && o[k] !== undefined ? o[k] : undefined), obj);
  return resolve(LOCALES[currentLocale]) ?? resolve(LOCALES.en) ?? key;
}

/**
 * Format a numeric amount using the Intl API.
 * @param {number} amount
 * @param {string} [currency] - ISO 4217 code; defaults to locale preference
 */
function formatCurrency(amount, currency) {
  const cur = currency ?? LOCALE_CURRENCY[currentLocale] ?? 'USD';
  return new Intl.NumberFormat(currentLocale, {
    style: 'currency',
    currency: cur,
    minimumFractionDigits: 2,
  }).format(amount);
}

/**
 * Format a date/time string using the Intl API.
 * @param {string|Date} date
 */
function formatDate(date) {
  return new Intl.DateTimeFormat(currentLocale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(date));
}

/**
 * Apply the current locale to the document:
 * - Sets <html lang> and dir attributes
 * - Translates all elements with data-i18n attributes
 * - Translates aria-label attributes via data-i18n-aria
 */
function applyLocale() {
  const isRTL = RTL_LANGS.has(currentLocale);
  document.documentElement.lang = currentLocale;
  document.documentElement.dir = isRTL ? 'rtl' : 'ltr';
  document.body.classList.toggle('rtl', isRTL);

  document.querySelectorAll('[data-i18n]').forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });

  document.querySelectorAll('[data-i18n-aria]').forEach((el) => {
    el.setAttribute('aria-label', t(el.dataset.i18nAria));
  });

  document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
    el.setAttribute('placeholder', t(el.dataset.i18nPlaceholder));
  });
}

/**
 * Switch to a new locale and persist the choice.
 * @param {string} locale - One of the supported locale codes
 */
function setLocale(locale) {
  if (!LOCALES[locale]) {
    console.warn(`[i18n] Unsupported locale: ${locale}`);
    return;
  }
  currentLocale = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  applyLocale();
  document.dispatchEvent(new CustomEvent('localechange', { detail: { locale } }));
}

function getLocale() {
  return currentLocale;
}

function getSupportedLocales() {
  return Object.keys(LOCALES);
}

// Apply on load
applyLocale();

export { t, setLocale, getLocale, getSupportedLocales, formatCurrency, formatDate };
