import { writable, readable } from 'svelte/store';
import { browser } from '$app/environment';

/** Reactive online/offline status */
export const isOnline = readable(true, (set) => {
  if (!browser) return;
  set(navigator.onLine);
  const on = () => set(true);
  const off = () => set(false);
  window.addEventListener('online', on);
  window.addEventListener('offline', off);
  return () => { window.removeEventListener('online', on); window.removeEventListener('offline', off); };
});

/** Push notification permission state */
export const pushPermission = writable<NotificationPermission>('default');

export async function requestPushPermission(): Promise<boolean> {
  if (!browser || !('Notification' in window)) return false;
  const result = await Notification.requestPermission();
  pushPermission.set(result);
  return result === 'granted';
}

/** Subscribe to push notifications via the service worker */
export async function subscribePush(vapidPublicKey: string): Promise<PushSubscription | null> {
  if (!browser || !('serviceWorker' in navigator) || !('PushManager' in window)) return null;
  const reg = await navigator.serviceWorker.ready;
  const existing = await reg.pushManager.getSubscription();
  if (existing) return existing;
  return reg.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: urlBase64ToUint8Array(vapidPublicKey)
  });
}

function urlBase64ToUint8Array(base64: string): Uint8Array {
  const padding = '='.repeat((4 - (base64.length % 4)) % 4);
  const b64 = (base64 + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(b64);
  return Uint8Array.from([...raw].map((c) => c.charCodeAt(0)));
}
