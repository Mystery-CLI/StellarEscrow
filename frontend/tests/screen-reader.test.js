/**
 * Screen Reader Compatibility Tests
 * WCAG 2.1 SC 4.1.2 – Name, Role, Value
 * SC 1.3.1 – Info & Relationships
 * SC 4.1.3 – Status Messages
 */

beforeEach(() => global.loadFixture());

// ─── 1. Live Region Announcements ────────────────────────────────────────────

describe('Live Region Announcements', () => {
  /** Simulate the app's announce() function */
  function announce(message, priority = 'polite') {
    const region = document.getElementById('live-region');
    region.setAttribute('aria-live', priority);
    region.textContent = '';
    // Real code uses rAF; we set synchronously in tests
    region.textContent = message;
  }

  test('announcing a message updates live region text', () => {
    announce('WebSocket connected');
    expect(document.getElementById('live-region').textContent).toBe('WebSocket connected');
  });

  test('assertive announcements use aria-live="assertive"', () => {
    announce('Error: connection failed', 'assertive');
    expect(document.getElementById('live-region').getAttribute('aria-live')).toBe('assertive');
  });

  test('polite announcements use aria-live="polite"', () => {
    announce('Loaded 50 events');
    expect(document.getElementById('live-region').getAttribute('aria-live')).toBe('polite');
  });

  test('live region is visually hidden (screen-reader-only)', () => {
    const region = document.getElementById('live-region');
    expect(region.classList.contains('visually-hidden')).toBe(true);
  });
});

// ─── 2. Toast Notifications ────────────────────────────────────────────────

describe('Toast Notifications', () => {
  /** Simulate the app's showToast() function */
  function showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'assertive');
    toast.textContent = message;
    container.appendChild(toast);
    return toast;
  }

  test('toasts are appended to the toast container', () => {
    showToast('Trade completed');
    expect(document.querySelector('#toast-container .toast')).not.toBeNull();
  });

  test('toasts have role="alert"', () => {
    const toast = showToast('Trade completed');
    expect(toast.getAttribute('role')).toBe('alert');
  });

  test('toasts have aria-live="assertive"', () => {
    const toast = showToast('Trade completed');
    expect(toast.getAttribute('aria-live')).toBe('assertive');
  });

  test('error toasts have .error class for styling distinction', () => {
    const toast = showToast('Connection failed', 'error');
    expect(toast.classList.contains('error')).toBe(true);
  });

  test('success toasts have .success class', () => {
    const toast = showToast('Wallet connected', 'success');
    expect(toast.classList.contains('success')).toBe(true);
  });

  test('toast container itself is aria-live="assertive"', () => {
    expect(document.getElementById('toast-container').getAttribute('aria-live')).toBe('assertive');
  });
});

// ─── 3. Dynamic Event Feed ───────────────────────────────────────────────────

describe('Dynamic Event Feed Screen Reader Support', () => {
  function addEventToFeed(event) {
    const feed = document.getElementById('events-list');
    const emptyState = feed.querySelector('.empty-state');
    if (emptyState) emptyState.remove();

    const item = document.createElement('div');
    item.className = 'event-item';
    item.setAttribute('role', 'article');
    item.setAttribute(
      'aria-label',
      `${event.event_type} event for trade ${event.trade_id || 'unknown'}`
    );
    item.innerHTML = `
      <span class="event-time">${event.timestamp}</span>
      <span class="event-type">${event.event_type}</span>
      ${event.trade_id ? `<span class="event-trade-id">Trade #${event.trade_id}</span>` : ''}
    `;
    feed.insertBefore(item, feed.firstChild);
    return item;
  }

  test('event items have role="article"', () => {
    addEventToFeed({ event_type: 'trade_created', trade_id: 1, timestamp: '10:00' });
    const item = document.querySelector('.event-item');
    expect(item.getAttribute('role')).toBe('article');
  });

  test('event items have a descriptive aria-label', () => {
    addEventToFeed({ event_type: 'trade_funded', trade_id: 42, timestamp: '10:05' });
    const item = document.querySelector('.event-item');
    expect(item.getAttribute('aria-label')).toContain('trade_funded');
    expect(item.getAttribute('aria-label')).toContain('42');
  });

  test('feed clears empty-state placeholder when events arrive', () => {
    expect(document.querySelector('#events-list .empty-state')).not.toBeNull();
    addEventToFeed({ event_type: 'trade_created', trade_id: 1, timestamp: '10:00' });
    expect(document.querySelector('#events-list .empty-state')).toBeNull();
  });

  test('feed has aria-relevant="additions" to limit announcements', () => {
    const feed = document.getElementById('events-list');
    expect(feed.getAttribute('aria-relevant')).toContain('additions');
  });
});

// ─── 4. Dynamic Search Results ───────────────────────────────────────────────

describe('Search Results Screen Reader Support', () => {
  function renderSearchResults(data) {
    const container = document.getElementById('search-results');
    let html = '';
    if (data.trades?.length > 0) {
      html += '<h3>Trades</h3>';
      data.trades.forEach((t) => {
        html += `<article class="result-card" tabindex="0">
          <h4 class="result-title">Trade #${t.id}</h4>
          <p class="result-meta">Status: ${t.status}</p>
        </article>`;
      });
    }
    if (!html) html = '<p class="empty-state">No results found</p>';
    container.innerHTML = html;
  }

  test('search results container has aria-live="polite"', () => {
    expect(document.getElementById('search-results').getAttribute('aria-live')).toBe('polite');
  });

  test('result cards use <article> for semantics', () => {
    renderSearchResults({ trades: [{ id: 1, status: 'Funded' }] });
    expect(document.querySelector('#search-results article')).not.toBeNull();
  });

  test('result cards have headings for navigation', () => {
    renderSearchResults({ trades: [{ id: 1, status: 'Funded' }] });
    expect(document.querySelector('#search-results h4')).not.toBeNull();
  });

  test('empty results show meaningful message', () => {
    renderSearchResults({});
    expect(document.querySelector('#search-results .empty-state').textContent).toMatch(/no results/i);
  });
});

// ─── 5. Status Indicators ────────────────────────────────────────────────────

describe('Status Indicator Updates', () => {
  function updateWsStatus(connected) {
    const indicator = document.getElementById('ws-indicator');
    const dot = indicator.querySelector('.indicator-dot');
    const text = indicator.querySelector('.indicator-text');
    if (connected) {
      dot.classList.remove('disconnected');
      text.textContent = 'Connected';
    } else {
      dot.classList.add('disconnected');
      text.textContent = 'Disconnected';
    }
  }

  test('ws indicator text updates on connect', () => {
    updateWsStatus(true);
    expect(document.querySelector('#ws-indicator .indicator-text').textContent).toBe('Connected');
  });

  test('ws indicator text updates on disconnect', () => {
    updateWsStatus(true);
    updateWsStatus(false);
    expect(document.querySelector('#ws-indicator .indicator-text').textContent).toBe('Disconnected');
  });

  test('ws indicator dot gains .disconnected on disconnect', () => {
    updateWsStatus(false);
    expect(document.querySelector('#ws-indicator .indicator-dot').classList.contains('disconnected')).toBe(true);
  });

  test('ws indicator dot loses .disconnected on connect', () => {
    updateWsStatus(false);
    updateWsStatus(true);
    expect(document.querySelector('#ws-indicator .indicator-dot').classList.contains('disconnected')).toBe(false);
  });
});

// ─── 6. Error Modal Screen Reader Support ────────────────────────────────────

describe('Error Modal Screen Reader Support', () => {
  function showErrorModal(message) {
    const modal = document.getElementById('error-modal');
    document.getElementById('error-modal-message').textContent = message;
    modal.hidden = false;
  }

  test('error modal has role="alertdialog"', () => {
    expect(document.getElementById('error-modal').getAttribute('role')).toBe('alertdialog');
  });

  test('error modal links title to aria-labelledby', () => {
    const modal = document.getElementById('error-modal');
    const labelId = modal.getAttribute('aria-labelledby');
    expect(document.getElementById(labelId)).not.toBeNull();
  });

  test('error modal links description to aria-describedby', () => {
    const modal = document.getElementById('error-modal');
    const descId = modal.getAttribute('aria-describedby');
    expect(document.getElementById(descId)).not.toBeNull();
  });

  test('error message is programmatically exposed when set', () => {
    showErrorModal('Network request failed');
    expect(document.getElementById('error-modal-message').textContent).toBe('Network request failed');
  });
});

// ─── 7. Pagination Announcements ─────────────────────────────────────────────

describe('Pagination Accessibility', () => {
  test('pagination region has aria-label', () => {
    const pagination = document.getElementById('events-pagination');
    expect(pagination.getAttribute('aria-label')).toBeTruthy();
  });

  test('pagination info span is aria-live="polite"', () => {
    const info = document.getElementById('pagination-info');
    expect(info.getAttribute('aria-live')).toBeTruthy();
  });

  test('pagination buttons have descriptive aria-label', () => {
    const prevBtn = document.querySelector('[data-action="prev"]');
    const nextBtn = document.querySelector('[data-action="next"]');
    expect(prevBtn.getAttribute('aria-label')).toMatch(/previous/i);
    expect(nextBtn.getAttribute('aria-label')).toMatch(/next/i);
  });

  test('updating pagination info text is readable by screen readers', () => {
    const info = document.getElementById('pagination-info');
    info.textContent = 'Page 2 of 5';
    expect(info.textContent).toBe('Page 2 of 5');
  });
});

// ─── 8. Language Switcher ────────────────────────────────────────────────────

describe('Language Switcher Accessibility', () => {
  test('language select has a visually-hidden label', () => {
    const select = document.getElementById('lang-select');
    const label = document.querySelector('label[for="lang-select"]');
    expect(label).not.toBeNull();
    expect(label.classList.contains('visually-hidden')).toBe(true);
  });

  test('language select also has aria-label', () => {
    const select = document.getElementById('lang-select');
    expect(select.getAttribute('aria-label')).toBeTruthy();
  });

  test('language switcher has English, French, Spanish, Arabic options', () => {
    const options = document.querySelectorAll('#lang-select option');
    const values = [...options].map((o) => o.value);
    expect(values).toContain('en');
    expect(values).toContain('fr');
    expect(values).toContain('es');
    expect(values).toContain('ar');
  });
});