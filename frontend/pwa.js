/**
 * StellarEscrow PWA module
 * Handles SW registration, install prompt, push subscriptions, and offline indicator.
 */

const VAPID_PUBLIC_KEY = ''; // Set your VAPID public key here

// ── Service Worker registration ───────────────────────────────────────────────

export async function registerServiceWorker() {
  if (!('serviceWorker' in navigator)) return null;
  try {
    const reg = await navigator.serviceWorker.register('/sw.js', { scope: '/' });
    console.log('[PWA] Service worker registered', reg.scope);

    // Listen for messages from SW (e.g. SYNC_COMPLETE)
    navigator.serviceWorker.addEventListener('message', (event) => {
      if (event.data?.type === 'SYNC_COMPLETE') {
        document.dispatchEvent(new CustomEvent('pwa:synced'));
      }
    });

    return reg;
  } catch (err) {
    console.warn('[PWA] Service worker registration failed:', err);
    return null;
  }
}

// ── Install prompt ────────────────────────────────────────────────────────────

let _deferredPrompt = null;

window.addEventListener('beforeinstallprompt', (e) => {
  e.preventDefault();
  _deferredPrompt = e;
  document.dispatchEvent(new CustomEvent('pwa:installable'));
});

window.addEventListener('appinstalled', () => {
  _deferredPrompt = null;
  document.dispatchEvent(new CustomEvent('pwa:installed'));
});

export async function promptInstall() {
  if (!_deferredPrompt) return false;
  _deferredPrompt.prompt();
  const { outcome } = await _deferredPrompt.userChoice;
  _deferredPrompt = null;
  return outcome === 'accepted';
}

export function isInstallable() {
  return _deferredPrompt !== null;
}

// ── Push notifications ────────────────────────────────────────────────────────

export async function subscribePush(registration) {
  if (!('PushManager' in window) || !VAPID_PUBLIC_KEY) return null;
  try {
    const permission = await Notification.requestPermission();
    if (permission !== 'granted') return null;

    const subscription = await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY),
    });
    return subscription;
  } catch (err) {
    console.warn('[PWA] Push subscription failed:', err);
    return null;
  }
}

function urlBase64ToUint8Array(base64String) {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(base64);
  return Uint8Array.from([...raw].map((c) => c.charCodeAt(0)));
}

// ── Offline indicator ─────────────────────────────────────────────────────────

export function initOfflineIndicator() {
  const indicator = document.getElementById('offline-indicator');
  if (!indicator) return;

  function update() {
    const offline = !navigator.onLine;
    indicator.hidden = !offline;
    indicator.setAttribute('aria-hidden', String(!offline));
    document.body.classList.toggle('offline', offline);
  }

  window.addEventListener('online', update);
  window.addEventListener('offline', update);
  update();
}
