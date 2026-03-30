/**
 * StellarEscrow A11y Test Setup
 * Provides shared HTML fixture and utilities for all test suites
 */

// Mock localStorage
const localStorageMock = (() => {
  let store = {};
  return {
    getItem: (k) => store[k] ?? null,
    setItem: (k, v) => { store[k] = String(v); },
    removeItem: (k) => { delete store[k]; },
    clear: () => { store = {}; },
  };
})();
Object.defineProperty(global, 'localStorage', { value: localStorageMock });

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock IntersectionObserver
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// Shared HTML fixture derived from the real index.html
global.STELLAR_HTML = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="description" content="StellarEscrow - Accessible decentralized escrow dashboard">
  <meta name="theme-color" content="#1a1a2e">
  <title>StellarEscrow Dashboard</title>
</head>
<body>
  <a href="#main-content" class="skip-link">Skip to main content</a>
  <div id="live-region" class="visually-hidden" aria-live="polite" aria-atomic="true"></div>
  <div id="offline-indicator" role="status" aria-live="polite" hidden>Offline</div>

  <button id="contrast-toggle" class="contrast-toggle" type="button"
    aria-pressed="false" aria-label="Toggle high contrast mode">
    <span aria-hidden="true">◐</span>
    <span class="toggle-text">High Contrast</span>
  </button>

  <header role="banner">
    <nav role="navigation" aria-label="Main navigation">
      <div class="nav-container">
        <h1 class="logo">
          <a href="/" aria-label="StellarEscrow Home">
            <span aria-hidden="true">◇</span>
            <span>StellarEscrow</span>
          </a>
        </h1>
        <button id="mobile-menu-btn" type="button"
          aria-expanded="false" aria-controls="main-menu"
          aria-label="Toggle navigation menu">
          <span aria-hidden="true"></span>
        </button>
        <ul id="main-menu" class="nav-menu" role="menubar">
          <li role="none"><a href="#dashboard" role="menuitem" class="nav-link active"
            aria-current="page">Dashboard</a></li>
          <li role="none"><a href="#events" role="menuitem" class="nav-link">Events</a></li>
          <li role="none"><a href="#search" role="menuitem" class="nav-link">Search</a></li>
          <li role="none"><a href="#disputes" role="menuitem" class="nav-link">Disputes</a></li>
          <li role="none"><a href="#help" role="menuitem" class="nav-link">Help Center</a></li>
        </ul>
        <div class="lang-switcher">
          <label for="lang-select" class="visually-hidden">Language</label>
          <select id="lang-select" aria-label="Select language">
            <option value="en">English</option>
            <option value="fr">Français</option>
            <option value="es">Español</option>
            <option value="ar">العربية</option>
          </select>
        </div>
        <div class="notification-controls">
          <button id="notification-center-btn" type="button" class="btn btn-icon"
            aria-label="Notification center" aria-expanded="false"
            aria-controls="notification-center">
            🔔
            <span id="notification-badge" class="notification-badge" hidden>0</span>
          </button>
        </div>
        <div id="wallet-container" class="wallet-container" role="region" aria-label="Wallet connection">
          <button id="wallet-connect-btn" class="btn btn-wallet" type="button"
            aria-label="Connect Stellar wallet" aria-expanded="false"
            aria-controls="wallet-menu">Connect Wallet</button>
          <div id="wallet-menu" class="wallet-menu" hidden role="menu" aria-label="Wallet options">
            <button type="button" class="wallet-option" data-provider="freighter"
              role="menuitem" aria-label="Connect with Freighter wallet">Freighter</button>
            <button type="button" class="wallet-option" data-provider="albedo"
              role="menuitem" aria-label="Connect with Albedo wallet">Albedo</button>
          </div>
          <div id="wallet-connected" class="wallet-connected" hidden>
            <div class="wallet-info">
              <span id="wallet-provider-badge" class="wallet-provider-badge"></span>
              <span id="wallet-address-display" class="wallet-address-display"></span>
            </div>
            <button id="wallet-menu-toggle" type="button" class="wallet-menu-toggle"
              aria-label="Wallet menu" aria-expanded="false" aria-controls="wallet-dropdown">⋮</button>
            <div id="wallet-dropdown" class="wallet-dropdown" hidden role="menu">
              <button type="button" class="wallet-dropdown-item" id="disconnect-wallet-btn"
                role="menuitem">Disconnect</button>
            </div>
          </div>
        </div>
      </div>
    </nav>
  </header>

  <main id="main-content" role="main">
    <section id="dashboard" aria-labelledby="dashboard-heading" class="section">
      <div class="container">
        <h2 id="dashboard-heading" class="section-title">Dashboard</h2>
        <div class="status-grid" role="region" aria-label="System status overview">
          <article class="status-card" aria-labelledby="health-status">
            <h3 id="health-status" class="card-title">System Health</h3>
            <div id="health-indicator" class="status-indicator" role="status" aria-live="polite">
              <span class="indicator-dot" aria-hidden="true"></span>
              <span class="indicator-text">Checking...</span>
            </div>
          </article>
          <article class="status-card" aria-labelledby="ws-status">
            <h3 id="ws-status" class="card-title">WebSocket</h3>
            <div id="ws-indicator" class="status-indicator" role="status" aria-live="polite">
              <span class="indicator-dot disconnected" aria-hidden="true"></span>
              <span class="indicator-text">Disconnected</span>
            </div>
          </article>
        </div>
        <div class="events-feed" role="region" aria-label="Real-time events feed">
          <div class="feed-header">
            <h3>Real-time Events</h3>
            <button id="clear-feed-btn" type="button" class="btn btn-secondary"
              aria-label="Clear events feed">Clear Feed</button>
          </div>
          <div id="events-list" class="events-list" role="log"
            aria-live="polite" aria-label="Events log" aria-relevant="additions" tabindex="0">
            <p class="empty-state">Connecting to event stream...</p>
          </div>
        </div>
      </div>
    </section>

    <section id="events" aria-labelledby="events-heading" class="section">
      <div class="container">
        <h2 id="events-heading" class="section-title">Event Explorer</h2>
        <form id="events-filter-form" class="filter-form" role="search" aria-label="Events filter form">
          <div class="filter-group">
            <label for="event-type-filter" class="filter-label">Event Type</label>
            <select id="event-type-filter" class="filter-select" aria-describedby="event-type-help">
              <option value="">All Events</option>
              <option value="trade_created">Trade Created</option>
              <option value="trade_funded">Trade Funded</option>
            </select>
            <span id="event-type-help" class="visually-hidden">Filter events by type</span>
          </div>
          <div class="filter-group">
            <label for="trade-id-filter" class="filter-label">Trade ID</label>
            <input type="number" id="trade-id-filter" class="filter-input"
              placeholder="Enter trade ID" min="0" aria-describedby="trade-id-desc">
            <span id="trade-id-desc" class="visually-hidden">Enter a specific trade ID</span>
          </div>
          <button type="submit" class="btn btn-primary" aria-label="Apply filters to events">Apply Filters</button>
        </form>
        <div class="table-container" role="region" aria-label="Events table">
          <table id="events-table" class="data-table" aria-describedby="events-table-caption">
            <caption id="events-table-caption" class="visually-hidden">
              Contract events with timestamp, type, trade ID, and details
            </caption>
            <thead>
              <tr>
                <th scope="col">Time</th>
                <th scope="col">Type</th>
                <th scope="col">Trade ID</th>
                <th scope="col">Details</th>
              </tr>
            </thead>
            <tbody id="events-table-body">
              <tr><td colspan="4" class="empty-state">No events loaded.</td></tr>
            </tbody>
          </table>
        </div>
        <nav id="events-pagination" class="pagination" aria-label="Events pagination">
          <button type="button" class="btn btn-secondary pagination-btn"
            data-action="prev" aria-label="Previous page of events">← Previous</button>
          <span id="pagination-info" class="pagination-info" aria-live="polite">Page 1</span>
          <button type="button" class="btn btn-secondary pagination-btn"
            data-action="next" aria-label="Next page of events">Next →</button>
        </nav>
      </div>
    </section>

    <section id="search" aria-labelledby="search-heading" class="section">
      <div class="container">
        <h2 id="search-heading" class="section-title">Global Search</h2>
        <form id="search-form" class="search-form" role="search" aria-label="Global search">
          <div class="search-input-group">
            <label for="search-input" class="visually-hidden">Search trades, users, and arbitrators</label>
            <input type="search" id="search-input" class="search-input"
              placeholder="Search..." aria-describedby="search-help" autocomplete="off">
            <button type="submit" class="btn btn-primary search-btn" aria-label="Submit search">Search</button>
          </div>
          <span id="search-help" class="visually-hidden">Enter keywords to search</span>
        </form>
        <div id="search-results" class="search-results" role="region"
          aria-label="Search results" aria-live="polite">
          <p class="empty-state">Enter a search term to find results.</p>
        </div>
      </div>
    </section>

    <section id="disputes" aria-labelledby="disputes-heading" class="section">
      <div class="container">
        <h2 id="disputes-heading" class="section-title">Dispute Management</h2>
        <button id="raise-dispute-btn" type="button" class="btn btn-primary"
          aria-label="Raise a new dispute">Raise Dispute</button>
        <div id="disputes-list" class="disputes-list" role="region" aria-label="Disputes list">
          <p class="empty-state">No disputes found</p>
        </div>
      </div>
    </section>

    <div id="dispute-modal" class="modal" hidden role="dialog"
      aria-labelledby="dispute-modal-title" aria-modal="true">
      <div class="modal-content">
        <div class="modal-header">
          <h3 id="dispute-modal-title">Raise Dispute</h3>
          <button id="dispute-modal-close" type="button" class="btn btn-close"
            aria-label="Close dialog">✕</button>
        </div>
        <form id="dispute-form" class="dispute-form">
          <div class="form-group">
            <label for="dispute-trade-id" class="form-label">Trade ID *</label>
            <input type="number" id="dispute-trade-id" class="form-input"
              required min="1" aria-describedby="trade-id-help">
            <span id="trade-id-help" class="form-help">Enter the trade ID to dispute</span>
          </div>
          <div class="form-group">
            <label for="dispute-reason" class="form-label">Reason *</label>
            <textarea id="dispute-reason" class="form-textarea" required rows="4"
              aria-describedby="reason-help"></textarea>
            <span id="reason-help" class="form-help">Provide details about the dispute</span>
          </div>
          <div class="form-group">
            <label for="evidence-input" class="form-label">Upload Evidence</label>
            <input type="file" id="evidence-input" class="form-input" multiple
              accept=".jpg,.jpeg,.png,.pdf,.txt" aria-describedby="evidence-help">
            <span id="evidence-help" class="form-help">Supported: JPG, PNG, PDF, TXT</span>
          </div>
          <div class="form-actions">
            <button type="submit" class="btn btn-primary" aria-label="Submit dispute">Submit</button>
            <button type="button" class="btn btn-secondary" id="dispute-cancel-btn">Cancel</button>
          </div>
        </form>
      </div>
    </div>

    <section id="help" aria-labelledby="help-heading" class="section">
      <div class="container">
        <h2 id="help-heading" class="section-title">Help Center</h2>
        <nav id="help-nav" class="help-nav" aria-label="Help center sections">
          <button type="button" class="help-nav-btn active" data-target="faqs" aria-pressed="true">FAQs</button>
          <button type="button" class="help-nav-btn" data-target="tutorials" aria-pressed="false">Tutorials</button>
        </nav>
        <div id="faqs-content" class="help-content active" role="region" aria-label="Frequently Asked Questions">
          <div class="faq-list" role="list">
            <details class="faq-item">
              <summary class="faq-question">What is StellarEscrow?</summary>
              <div class="faq-answer"><p>A decentralized escrow platform.</p></div>
            </details>
          </div>
        </div>
        <div id="tutorials-content" class="help-content" role="region" aria-label="Tutorials" hidden>
          <div class="tutorial-list" role="list"></div>
        </div>
      </div>
    </section>

    <div id="notification-center" class="notification-center" hidden
      role="region" aria-label="Notification center">
      <div class="notification-center-header">
        <h3>Notifications</h3>
        <button type="button" class="btn btn-close" id="notification-center-close"
          aria-label="Close notification center">✕</button>
      </div>
      <div class="notification-center-list">
        <p class="empty-state">No notifications</p>
      </div>
    </div>

    <div id="toast-container" class="toast-container" role="alert"
      aria-live="assertive" aria-label="Notifications"></div>

    <div id="error-modal" class="error-modal" role="alertdialog" aria-modal="true"
      aria-labelledby="error-modal-title" aria-describedby="error-modal-message" hidden>
      <div class="error-modal-content">
        <h2 id="error-modal-title" class="error-modal-title">Something went wrong</h2>
        <p id="error-modal-message" class="error-modal-message"></p>
        <div class="error-modal-actions">
          <button id="error-modal-close" type="button" class="btn btn-ghost">Dismiss</button>
        </div>
      </div>
    </div>
  </main>

  <footer role="contentinfo">
    <div class="container">
      <div class="footer-content">
        <p class="copyright">StellarEscrow &copy; 2024. Built with accessibility in mind.</p>
        <nav class="footer-links" aria-label="Footer navigation">
          <a href="#help" aria-label="Help Center">Help Center</a>
          <a href="https://stellar.org" target="_blank" rel="noopener noreferrer"
            aria-label="Visit Stellar website (opens in new tab)">Stellar</a>
        </nav>
      </div>
    </div>
  </footer>
</body>
</html>`;

/** Load the HTML fixture into jsdom */
global.loadFixture = () => {
  document.open();
  document.write(global.STELLAR_HTML);
  document.close();
};