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
    var isBrave = !!(navigator.brave && typeof navigator.brave.isBrave === 'function');
    if (isBrave)              return { name: 'Brave',   version: (ua.match(/Chrome\/([\d.]+)/) || [])[1] };
    if (/Edg\//.test(ua))     return { name: 'Edge',    version: (ua.match(/Edg\/([\d.]+)/)    || [])[1] };
    if (/OPR\//.test(ua))     return { name: 'Opera',   version: (ua.match(/OPR\/([\d.]+)/)    || [])[1] };
    if (/Chrome\//.test(ua))  return { name: 'Chrome',  version: (ua.match(/Chrome\/([\d.]+)/) || [])[1] };
    if (/Firefox\//.test(ua)) return { name: 'Firefox', version: (ua.match(/Firefox\/([\d.]+)/)|| [])[1] };
    if (/Safari\//.test(ua))  return { name: 'Safari',  version: (ua.match(/Version\/([\d.]+)/)|| [])[1] };
    if (/Trident\//.test(ua)) return { name: 'IE',      version: (ua.match(/rv:([\d.]+)/)      || [])[1] };
    return { name: 'Unknown', version: '0' };
  })();

  // Minimum supported versions
  var MIN_VERSIONS = { Chrome: 90, Firefox: 88, Safari: 14, Edge: 90, Opera: 76, Brave: 90 };

  var majorVersion = parseInt((browser.version || '0').split('.')[0], 10);
  var isSupported   = browser.name !== 'IE' &&
                      (!(browser.name in MIN_VERSIONS) || majorVersion >= MIN_VERSIONS[browser.name]);
  var isIE          = browser.name === 'IE';
  var isSafari      = browser.name === 'Safari';
  var isMobileSafari = /iPhone|iPad|iPod/.test(ua);

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
    abortController:  typeof AbortController !== 'undefined',
    promiseAllSettled:typeof Promise.allSettled !== 'undefined',
    stringReplaceAll: typeof String.prototype.replaceAll !== 'undefined',
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

  // Promise.allSettled
  if (!features.promiseAllSettled) {
    Promise.allSettled = function (promises) {
      return Promise.all(promises.map(function (p) {
        return Promise.resolve(p).then(
          function (val) { return { status: 'fulfilled', value: val }; },
          function (err) { return { status: 'rejected', reason: err }; }
        );
      }));
    };
  }

  // AbortController (basic polyfill)
  if (!features.abortController) {
    function AbortSignal() { this.aborted = false; this.onabort = null; }
    function AbortController() { this.signal = new AbortSignal(); }
    AbortController.prototype.abort = function () {
      this.signal.aborted = true;
      if (typeof this.signal.onabort === 'function') this.signal.onabort();
    };
    window.AbortController = AbortController;
    window.AbortSignal = AbortSignal;
  }

  // String.prototype.replaceAll
  if (!features.stringReplaceAll) {
    String.prototype.replaceAll = function (search, replacement) {
      if (search instanceof RegExp && !search.global) {
        throw new TypeError('replaceAll must be called with a global RegExp');
      }
      return this.split(search).join(replacement);
    };
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

  if (!features.localStorage) {
    showWarning(
      'LocalStorage is disabled or not supported. This might be due to Private Browsing mode. ' +
      'Settings and session data will not be saved.',
      'compat-storage-warning'
    );
  }

  // ---------------------------------------------------------------------------
  // Browser-specific fixes
  // ---------------------------------------------------------------------------

  // Safari: 100vh height fix for mobile
  if (isMobileSafari) {
    var setVH = function () {
      var vh = window.innerHeight * 0.01;
      document.documentElement.style.setProperty('--vh', vh + 'px');
    };
    window.addEventListener('resize', setVH);
    setVH();
  }

  // Chrome / Edge / Brave: custom scrollbar styling if supported
  var isChromium = /Chrome/.test(browser.name) || /Edge/.test(browser.name) || /Brave/.test(browser.name);
  if (isChromium) {
    var scrollStyle = document.createElement('style');
    scrollStyle.textContent = [
      '::-webkit-scrollbar{width:8px;height:8px}',
      '::-webkit-scrollbar-track{background:#1a1a2e}',
      '::-webkit-scrollbar-thumb{background:#4b4b7c;border-radius:4px}',
      '::-webkit-scrollbar-thumb:hover{background:#6366f1}',
    ].join('');
    document.head.appendChild(scrollStyle);
  }

  // Safari: passive touch-event listeners to avoid scroll-blocking warnings
  // and fix 300ms tap delay on older iOS Safari (< 13).
  if (isSafari || isMobileSafari) {
    // Ensure touch-action is set so iOS Safari doesn't delay click events
    var safariStyle = document.createElement('style');
    safariStyle.textContent = 'a,button,[role="button"]{touch-action:manipulation}';
    document.head.appendChild(safariStyle);
  }

  // iOS Safari: prevent elastic over-scroll on fixed elements (common layout bug)
  if (isMobileSafari) {
    document.addEventListener('touchmove', function (e) {
      var target = e.target;
      if (target && target.closest && target.closest('.modal, .wallet-menu, .wallet-dropdown')) {
        e.stopPropagation();
      }
    }, { passive: true });
  }

  // Firefox: :focus-visible polyfill — add .focus-visible class via JS when
  // keyboard navigation is detected (Firefox < 85 lacks :focus-visible support).
  var firefoxMajor = browser.name === 'Firefox' ? parseInt((browser.version || '0').split('.')[0], 10) : 999;
  if (firefoxMajor < 85) {
    var usingKeyboard = false;
    document.addEventListener('keydown', function () { usingKeyboard = true; }, true);
    document.addEventListener('mousedown', function () { usingKeyboard = false; }, true);
    document.addEventListener('focusin', function (e) {
      if (usingKeyboard && e.target && e.target.classList) {
        e.target.classList.add('focus-visible');
      }
    }, true);
    document.addEventListener('focusout', function (e) {
      if (e.target && e.target.classList) {
        e.target.classList.remove('focus-visible');
      }
    }, true);
  }

  // Safari / WebKit: smooth-scroll polyfill for anchor navigation
  // (Safari < 15.4 does not support CSS scroll-behavior: smooth natively)
  var safariMajor = isSafari ? parseInt((browser.version || '0').split('.')[0], 10) : 999;
  if (safariMajor < 15) {
    document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
      anchor.addEventListener('click', function (e) {
        var href = anchor.getAttribute('href');
        if (!href || href === '#') return;
        var target = document.querySelector(href);
        if (target) {
          e.preventDefault();
          target.scrollIntoView({ behavior: 'smooth', block: 'start' });
          // Update focus for accessibility
          if (!target.hasAttribute('tabindex')) {
            target.setAttribute('tabindex', '-1');
          }
          target.focus({ preventScroll: true });
        }
      });
    });
  }

  // Edge (Chromium) / Chrome: ensure dialog element is supported, add basic
  // polyfill for <dialog> if missing (used by some modal patterns).
  if (typeof HTMLDialogElement === 'undefined') {
    var dialogStyle = document.createElement('style');
    dialogStyle.textContent = 'dialog{display:none;position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);z-index:10000;background:#fff;padding:1em;border:1px solid #ccc}dialog[open]{display:block}';
    document.head.appendChild(dialogStyle);
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------
  window.StellarCompat = {
    browser:     browser,
    features:    features,
    isSupported: isSupported,
    isIE:        isIE,
    fixes: {
      safariTouchAction:    isSafari || isMobileSafari,
      iosTouchMove:         isMobileSafari,
      safariVHFix:          isMobileSafari,
      chromiumScrollbars:   isChromium,
      firefoxFocusVisible:  firefoxMajor < 85,
      safariSmoothScroll:   safariMajor < 15,
      dialogPolyfill:       typeof HTMLDialogElement === 'undefined',
      abortController:      !features.abortController,
      promiseAllSettled:    !features.promiseAllSettled,
      stringReplaceAll:     !features.stringReplaceAll,
    },
  };

})();
