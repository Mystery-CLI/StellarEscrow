/**
 * StellarEscrow — Browser Compatibility
 * Detects browser/feature support, applies polyfills, and shows warnings.
 */
(function () {
  'use strict';

  // ---------------------------------------------------------------------------
  // Browser detection
  // ---------------------------------------------------------------------------
  var ua = navigator.userAgent;

  var browser = (function () {
    if (/Edg\//.test(ua))     return { name: 'Edge',    version: (ua.match(/Edg\/([\d.]+)/)    || [])[1] };
    if (/OPR\//.test(ua))     return { name: 'Opera',   version: (ua.match(/OPR\/([\d.]+)/)    || [])[1] };
    if (/Chrome\//.test(ua))  return { name: 'Chrome',  version: (ua.match(/Chrome\/([\d.]+)/) || [])[1] };
    if (/Firefox\//.test(ua)) return { name: 'Firefox', version: (ua.match(/Firefox\/([\d.]+)/)|| [])[1] };
    if (/Safari\//.test(ua))  return { name: 'Safari',  version: (ua.match(/Version\/([\d.]+)/)|| [])[1] };
    if (/Trident\//.test(ua)) return { name: 'IE',      version: (ua.match(/rv:([\d.]+)/)      || [])[1] };
    return { name: 'Unknown', version: '0' };
  })();

  // Minimum supported versions
  var MIN_VERSIONS = { Chrome: 90, Firefox: 88, Safari: 14, Edge: 90, Opera: 76 };

  var majorVersion = parseInt((browser.version || '0').split('.')[0], 10);
  var isSupported   = browser.name !== 'IE' &&
                      (!(browser.name in MIN_VERSIONS) || majorVersion >= MIN_VERSIONS[browser.name]);
  var isIE          = browser.name === 'IE';

  // ---------------------------------------------------------------------------
  // Feature detection
  // ---------------------------------------------------------------------------
  var features = {
    webSocket:        typeof WebSocket !== 'undefined',
    fetch:            typeof fetch !== 'undefined',
    promise:          typeof Promise !== 'undefined',
    customElements:   typeof customElements !== 'undefined',
    intersectionObs:  typeof IntersectionObserver !== 'undefined',
    cssGrid:          (function () {
                        try { return CSS.supports('display', 'grid'); } catch (e) { return false; }
                      })(),
    cssCustomProps:   (function () {
                        try { return CSS.supports('--a', '0'); } catch (e) { return false; }
                      })(),
    localStorage:     (function () {
                        try { localStorage.setItem('_t', '1'); localStorage.removeItem('_t'); return true; }
                        catch (e) { return false; }
                      })(),
    clipboard:        !!(navigator.clipboard && navigator.clipboard.writeText),
    notifications:    typeof Notification !== 'undefined',
    serviceWorker:    'serviceWorker' in navigator,
  };

  // ---------------------------------------------------------------------------
  // Polyfills
  // ---------------------------------------------------------------------------

  // Element.closest
  if (!Element.prototype.closest) {
    Element.prototype.closest = function (selector) {
      var el = this;
      while (el && el.nodeType === 1) {
        if (el.matches ? el.matches(selector) : el.msMatchesSelector(selector)) return el;
        el = el.parentElement || el.parentNode;
      }
      return null;
    };
  }

  // Element.matches
  if (!Element.prototype.matches) {
    Element.prototype.matches =
      Element.prototype.msMatchesSelector ||
      Element.prototype.webkitMatchesSelector;
  }

  // NodeList.forEach
  if (typeof NodeList !== 'undefined' && NodeList.prototype && !NodeList.prototype.forEach) {
    NodeList.prototype.forEach = Array.prototype.forEach;
  }

  // Object.assign
  if (typeof Object.assign !== 'function') {
    Object.assign = function (target) {
      for (var i = 1; i < arguments.length; i++) {
        var src = arguments[i];
        if (src) for (var k in src) if (Object.prototype.hasOwnProperty.call(src, k)) target[k] = src[k];
      }
      return target;
    };
  }

  // Array.from
  if (!Array.from) {
    Array.from = function (arrayLike) { return Array.prototype.slice.call(arrayLike); };
  }

  // CustomEvent constructor (IE)
  if (typeof CustomEvent !== 'function') {
    function CustomEvent(event, params) {
      params = params || { bubbles: false, cancelable: false, detail: null };
      var evt = document.createEvent('CustomEvent');
      evt.initCustomEvent(event, params.bubbles, params.cancelable, params.detail);
      return evt;
    }
    CustomEvent.prototype = window.Event.prototype;
    window.CustomEvent = CustomEvent;
  }

  // requestAnimationFrame
  if (!window.requestAnimationFrame) {
    var lastTime = 0;
    window.requestAnimationFrame = function (cb) {
      var now = Date.now();
      var delay = Math.max(0, 16 - (now - lastTime));
      lastTime = now + delay;
      return setTimeout(function () { cb(now + delay); }, delay);
    };
    window.cancelAnimationFrame = clearTimeout;
  }

  // ---------------------------------------------------------------------------
  // CSS fallbacks for browsers without custom properties
  // ---------------------------------------------------------------------------
  if (!features.cssCustomProps) {
    var style = document.createElement('style');
    style.textContent = [
      'body{background:#0f0f1a;color:#e8e8f0;font-family:Inter,-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif}',
      '.btn{background:#6366f1;color:#fff;border:none;border-radius:6px;padding:8px 16px;cursor:pointer}',
      '.card{background:#1e1e32;border:1px solid #3a3a5c;border-radius:8px;padding:16px}',
    ].join('');
    document.head.appendChild(style);
  }

  // ---------------------------------------------------------------------------
  // Compatibility warnings
  // ---------------------------------------------------------------------------
  function showWarning(message, id) {
    if (document.getElementById(id)) return; // already shown
    var banner = document.createElement('div');
    banner.id = id;
    banner.setAttribute('role', 'alert');
    banner.setAttribute('aria-live', 'assertive');
    banner.style.cssText = [
      'position:fixed;top:0;left:0;right:0;z-index:99999',
      'background:#b45309;color:#fff;padding:10px 16px',
      'font-family:sans-serif;font-size:14px;display:flex',
      'align-items:center;justify-content:space-between;gap:12px',
    ].join(';');
    var text = document.createElement('span');
    text.textContent = message;
    var close = document.createElement('button');
    close.textContent = '✕';
    close.setAttribute('aria-label', 'Dismiss warning');
    close.style.cssText = 'background:none;border:none;color:#fff;cursor:pointer;font-size:16px;padding:0 4px';
    close.onclick = function () { banner.remove(); };
    banner.appendChild(text);
    banner.appendChild(close);
    document.body ? document.body.insertBefore(banner, document.body.firstChild)
                  : document.addEventListener('DOMContentLoaded', function () {
                      document.body.insertBefore(banner, document.body.firstChild);
                    });
  }

  if (isIE) {
    showWarning(
      'Internet Explorer is not supported. Please use Chrome, Firefox, Edge, or Safari for the best experience.',
      'compat-ie-warning'
    );
  } else if (!isSupported) {
    showWarning(
      'Your browser (' + browser.name + ' ' + browser.version + ') may not be fully supported. ' +
      'Please update to the latest version for the best experience.',
      'compat-outdated-warning'
    );
  }

  if (!features.webSocket) {
    showWarning(
      'WebSocket is not supported in your browser. Real-time event updates will be unavailable.',
      'compat-ws-warning'
    );
  }

  if (!features.fetch) {
    showWarning(
      'Your browser does not support the Fetch API. Some features may not work correctly.',
      'compat-fetch-warning'
    );
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------
  window.StellarCompat = {
    browser:     browser,
    features:    features,
    isSupported: isSupported,
    isIE:        isIE,
  };

})();
