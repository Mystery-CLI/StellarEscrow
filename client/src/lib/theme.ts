import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';
export type AccentColor = 'indigo' | 'blue' | 'emerald' | 'violet' | 'rose';
export type FontSize = 'sm' | 'md' | 'lg';

export interface ThemePreferences {
  theme: Theme;
  accent: AccentColor;
  fontSize: FontSize;
}

const STORAGE_KEY = 'stellar-escrow-theme';

const defaults: ThemePreferences = { theme: 'light', accent: 'indigo', fontSize: 'md' };

function loadPrefs(): ThemePreferences {
  if (!browser) return defaults;
  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') };
  } catch {
    return defaults;
  }
}

function createThemeStore() {
  const { subscribe, set, update } = writable<ThemePreferences>(loadPrefs());

  function persist(prefs: ThemePreferences) {
    if (browser) localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
    applyToDOM(prefs);
  }

  return {
    subscribe,
    setTheme: (theme: Theme) => update(p => { const n = { ...p, theme }; persist(n); return n; }),
    setAccent: (accent: AccentColor) => update(p => { const n = { ...p, accent }; persist(n); return n; }),
    setFontSize: (fontSize: FontSize) => update(p => { const n = { ...p, fontSize }; persist(n); return n; }),
    toggle: () => update(p => { const n = { ...p, theme: p.theme === 'light' ? 'dark' : 'light' }; persist(n); return n; }),
    init: () => { const prefs = loadPrefs(); set(prefs); persist(prefs); }
  };
}

export const accentColors: Record<AccentColor, { bg: string; text: string; border: string }> = {
  indigo: { bg: '#6366f1', text: '#6366f1', border: '#6366f1' },
  blue:   { bg: '#3b82f6', text: '#3b82f6', border: '#3b82f6' },
  emerald:{ bg: '#10b981', text: '#10b981', border: '#10b981' },
  violet: { bg: '#8b5cf6', text: '#8b5cf6', border: '#8b5cf6' },
  rose:   { bg: '#f43f5e', text: '#f43f5e', border: '#f43f5e' },
};

export const fontSizeMap: Record<FontSize, string> = { sm: '14px', md: '16px', lg: '18px' };

export function applyToDOM(prefs: ThemePreferences) {
  if (!browser) return;
  const root = document.documentElement;
  root.classList.toggle('dark', prefs.theme === 'dark');
  const color = accentColors[prefs.accent].bg;
  root.style.setProperty('--accent', color);
  root.style.setProperty('--font-size-base', fontSizeMap[prefs.fontSize]);
}

export const themeStore = createThemeStore();
