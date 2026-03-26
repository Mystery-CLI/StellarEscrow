/**
 * StellarEscrow - Trade Dashboard
 * Displays active trades, trade history, quick actions, and real-time status updates.
 */

(function () {
    'use strict';

    // ============================================
    // Config & State
    // ============================================
    const API = '/api';
    const POLL_INTERVAL_MS = 10_000;

    const state = {
        trades: [],
        history: [],
        filter: { status: '', search: '', sort: 'created_desc' },
        historyFilter: { status: '', search: '', sort: 'created_desc' },
        activePage: 1,
        historyPage: 1,
        pageSize: 10,
        pollTimer: null,
        wsSocket: null,
    };

    // ============================================
    // DOM helpers
    // ============================================
    const $ = (sel, ctx = document) => ctx.querySelector(sel);
    const $$ = (sel, ctx = document) => [...ctx.querySelectorAll(sel)];

    function escapeHtml(str) {
        const d = document.createElement('div');
        d.textContent = String(str ?? '');
        return d.innerHTML;
    }

    function formatAmount(val) {
        return Number(val / 1e7).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 7 });
    }

    function formatDate(ts) {
        if (!ts) return '—';
        return new Date(ts).toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    }

    function shortAddr(addr) {
        if (!addr || addr.length < 10) return addr ?? '—';
        return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
    }

    function showToast(msg, type = 'info') {
        const c = $('#toast-container');
        if (!c) return;
        const t = document.createElement('div');
        t.className = `toast ${type}`;
        t.setAttribute('role', 'alert');
        t.textContent = msg;
        c.appendChild(t);
        setTimeout(() => { t.style.opacity = '0'; setTimeout(() => t.remove(), 300); }, 4000);
    }

    // ============================================
    // Status badge
    // ============================================
    const STATUS_LABELS = {
        Created: 'Created',
        Funded: 'Funded',
        Completed: 'Completed',
        Disputed: 'Disputed',
        Cancelled: 'Cancelled',
    };

    function statusBadge(status) {
        const label = STATUS_LABELS[status] ?? status;
        return `<span class="td-status-badge td-status-${(status ?? '').toLowerCase()}" aria-label="Status: ${escapeHtml(label)}">${escapeHtml(label)}</span>`;
    }

    // ============================================
    // Quick actions per status
    // ============================================
    function quickActions(trade) {
        const actions = [];
        switch (trade.status) {
            case 'Created':
                actions.push({ label: 'Fund', action: 'fund', style: 'primary' });
                actions.push({ label: 'Cancel', action: 'cancel', style: 'danger' });
                break;
            case 'Funded':
                actions.push({ label: 'Complete', action: 'complete', style: 'primary' });
                actions.push({ label: 'Dispute', action: 'dispute', style: 'warning' });
                break;
            case 'Completed':
                actions.push({ label: 'Confirm Receipt', action: 'confirm', style: 'primary' });
                actions.push({ label: 'Dispute', action: 'dispute', style: 'warning' });
                break;
            case 'Disputed':
                actions.push({ label: 'View Dispute', action: 'view-dispute', style: 'secondary' });
                break;
            default:
                break;
        }
        return actions
            .map(a => `<button class="btn btn-sm td-action-btn td-action-${a.style}" data-trade-id="${escapeHtml(trade.id)}" data-action="${a.action}" aria-label="${a.label} trade ${escapeHtml(trade.id)}">${a.label}</button>`)
            .join('');
    }

    // ============================================
    // API calls
    // ============================================
    async function apiFetch(path) {
        const res = await fetch(`${API}${path}`);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
    }

    async function loadActiveTrades() {
        try {
            const { status, search, sort } = state.filter;
            const params = new URLSearchParams({ limit: state.pageSize, offset: (state.activePage - 1) * state.pageSize });
            if (status) params.set('status', status);
            if (search) params.set('q', search);
            if (sort) params.set('sort', sort);
            // Only active (non-terminal) statuses
            if (!status) params.set('active_only', 'true');
            const data = await apiFetch(`/trades?${params}`);
            state.trades = Array.isArray(data) ? data : (data.trades ?? []);
            renderActiveTrades();
        } catch (e) {
            console.error('Failed to load active trades', e);
            renderActiveTradesError();
        }
    }

    async function loadTradeHistory() {
        try {
            const { status, search, sort } = state.historyFilter;
            const params = new URLSearchParams({ limit: state.pageSize, offset: (state.historyPage - 1) * state.pageSize });
            if (status) params.set('status', status);
            if (search) params.set('q', search);
            if (sort) params.set('sort', sort);
            const data = await apiFetch(`/trades/history?${params}`);
            state.history = Array.isArray(data) ? data : (data.trades ?? []);
            renderHistory();
        } catch (e) {
            console.error('Failed to load trade history', e);
            renderHistoryError();
        }
    }

    async function performAction(tradeId, action) {
        const endpoints = {
            fund: `/trades/${tradeId}/fund`,
            cancel: `/trades/${tradeId}/cancel`,
            complete: `/trades/${tradeId}/complete`,
            confirm: `/trades/${tradeId}/confirm`,
            dispute: `/trades/${tradeId}/dispute`,
        };

        if (action === 'view-dispute') {
            window.location.hash = '#disputes';
            return;
        }

        const endpoint = endpoints[action];
        if (!endpoint) return;

        try {
            const res = await fetch(`${API}${endpoint}`, { method: 'POST', headers: { 'Content-Type': 'application/json' } });
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            showToast(`Action "${action}" submitted for trade #${tradeId}`, 'success');
            await loadActiveTrades();
            await loadTradeHistory();
        } catch (e) {
            showToast(`Failed to perform action: ${e.message}`, 'error');
        }
    }

    // ============================================
    // Render: Active Trades
    // ============================================
    function renderActiveTrades() {
        const tbody = $('#td-active-tbody');
        const empty = $('#td-active-empty');
        if (!tbody) return;

        const trades = applyClientFilter(state.trades, state.filter);

        if (trades.length === 0) {
            tbody.innerHTML = '';
            if (empty) empty.hidden = false;
            return;
        }
        if (empty) empty.hidden = true;

        tbody.innerHTML = trades.map(t => `
            <tr>
                <td data-label="ID"><a href="#" class="td-trade-link" data-trade-id="${escapeHtml(t.id)}">#${escapeHtml(t.id)}</a></td>
                <td data-label="Status">${statusBadge(t.status)}</td>
                <td data-label="Amount" class="td-amount">${escapeHtml(formatAmount(t.amount))} USDC</td>
                <td data-label="Buyer" class="td-addr" title="${escapeHtml(t.buyer ?? '')}">${escapeHtml(shortAddr(t.buyer))}</td>
                <td data-label="Seller" class="td-addr" title="${escapeHtml(t.seller ?? '')}">${escapeHtml(shortAddr(t.seller))}</td>
                <td data-label="Created">${escapeHtml(formatDate(t.created_at))}</td>
                <td data-label="Actions" class="td-actions">${quickActions(t)}</td>
            </tr>
        `).join('');
    }

    function renderActiveTradesError() {
        const tbody = $('#td-active-tbody');
        if (tbody) tbody.innerHTML = `<tr><td colspan="7" class="td-error-state">Failed to load trades. <button class="btn btn-sm btn-ghost" id="td-retry-active">Retry</button></td></tr>`;
        $('#td-retry-active')?.addEventListener('click', loadActiveTrades);
    }

    // ============================================
    // Render: History
    // ============================================
    function renderHistory() {
        const tbody = $('#td-history-tbody');
        const empty = $('#td-history-empty');
        if (!tbody) return;

        const trades = applyClientFilter(state.history, state.historyFilter);

        if (trades.length === 0) {
            tbody.innerHTML = '';
            if (empty) empty.hidden = false;
            return;
        }
        if (empty) empty.hidden = true;

        tbody.innerHTML = trades.map(t => `
            <tr>
                <td data-label="ID"><a href="#" class="td-trade-link" data-trade-id="${escapeHtml(t.id)}">#${escapeHtml(t.id)}</a></td>
                <td data-label="Status">${statusBadge(t.status)}</td>
                <td data-label="Amount" class="td-amount">${escapeHtml(formatAmount(t.amount))} USDC</td>
                <td data-label="Buyer" class="td-addr" title="${escapeHtml(t.buyer ?? '')}">${escapeHtml(shortAddr(t.buyer))}</td>
                <td data-label="Seller" class="td-addr" title="${escapeHtml(t.seller ?? '')}">${escapeHtml(shortAddr(t.seller))}</td>
                <td data-label="Completed">${escapeHtml(formatDate(t.updated_at))}</td>
            </tr>
        `).join('');
    }

    function renderHistoryError() {
        const tbody = $('#td-history-tbody');
        if (tbody) tbody.innerHTML = `<tr><td colspan="6" class="td-error-state">Failed to load history. <button class="btn btn-sm btn-ghost" id="td-retry-history">Retry</button></td></tr>`;
        $('#td-retry-history')?.addEventListener('click', loadTradeHistory);
    }

    // ============================================
    // Client-side filter/sort (fallback when API doesn't support all params)
    // ============================================
    function applyClientFilter(trades, filter) {
        let result = [...trades];
        if (filter.status) result = result.filter(t => t.status === filter.status);
        if (filter.search) {
            const q = filter.search.toLowerCase();
            result = result.filter(t =>
                String(t.id).includes(q) ||
                (t.buyer ?? '').toLowerCase().includes(q) ||
                (t.seller ?? '').toLowerCase().includes(q)
            );
        }
        const [field, dir] = (filter.sort ?? 'created_desc').split('_');
        result.sort((a, b) => {
            let av, bv;
            if (field === 'amount') { av = Number(a.amount); bv = Number(b.amount); }
            else { av = new Date(a.created_at ?? 0).getTime(); bv = new Date(b.created_at ?? 0).getTime(); }
            return dir === 'asc' ? av - bv : bv - av;
        });
        return result;
    }

    // ============================================
    // Real-time updates via WebSocket
    // ============================================
    function connectDashboardWs() {
        const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
        const url = `${protocol}//${location.host}/api/ws`;
        try {
            state.wsSocket = new WebSocket(url);
            state.wsSocket.addEventListener('message', (ev) => {
                try {
                    const msg = JSON.parse(ev.data);
                    const tradeEvents = ['trade_created', 'trade_funded', 'trade_completed', 'trade_confirmed', 'trade_cancelled', 'dispute_raised', 'dispute_resolved'];
                    if (tradeEvents.includes(msg.event_type)) {
                        loadActiveTrades();
                        loadTradeHistory();
                        updateLiveIndicator(true);
                    }
                } catch (_) { /* ignore parse errors */ }
            });
            state.wsSocket.addEventListener('open', () => updateLiveIndicator(true));
            state.wsSocket.addEventListener('close', () => {
                updateLiveIndicator(false);
                // Fall back to polling
                startPolling();
            });
            state.wsSocket.addEventListener('error', () => updateLiveIndicator(false));
        } catch (_) {
            startPolling();
        }
    }

    function startPolling() {
        if (state.pollTimer) return;
        state.pollTimer = setInterval(() => {
            loadActiveTrades();
            loadTradeHistory();
        }, POLL_INTERVAL_MS);
    }

    function updateLiveIndicator(live) {
        const dot = $('#td-live-dot');
        const text = $('#td-live-text');
        if (!dot || !text) return;
        dot.className = `indicator-dot${live ? '' : ' disconnected'}`;
        text.textContent = live ? 'Live' : 'Polling';
    }

    // ============================================
    // Event delegation for action buttons & links
    // ============================================
    function bindEvents() {
        // Active trades filter form
        $('#td-active-filter-form')?.addEventListener('submit', (e) => {
            e.preventDefault();
            state.filter.status = $('#td-active-status-filter')?.value ?? '';
            state.filter.sort = $('#td-active-sort')?.value ?? 'created_desc';
            state.activePage = 1;
            renderActiveTrades();
        });

        // Active trades search (debounced)
        let searchTimer;
        $('#td-active-search')?.addEventListener('input', (e) => {
            clearTimeout(searchTimer);
            searchTimer = setTimeout(() => {
                state.filter.search = e.target.value.trim();
                state.activePage = 1;
                renderActiveTrades();
            }, 300);
        });

        // History filter form
        $('#td-history-filter-form')?.addEventListener('submit', (e) => {
            e.preventDefault();
            state.historyFilter.status = $('#td-history-status-filter')?.value ?? '';
            state.historyFilter.sort = $('#td-history-sort')?.value ?? 'created_desc';
            state.historyPage = 1;
            renderHistory();
        });

        // History search (debounced)
        let historySearchTimer;
        $('#td-history-search')?.addEventListener('input', (e) => {
            clearTimeout(historySearchTimer);
            historySearchTimer = setTimeout(() => {
                state.historyFilter.search = e.target.value.trim();
                state.historyPage = 1;
                renderHistory();
            }, 300);
        });

        // Action buttons (event delegation)
        document.addEventListener('click', (e) => {
            const btn = e.target.closest('.td-action-btn');
            if (btn) {
                const { tradeId, action } = btn.dataset;
                if (tradeId && action) performAction(tradeId, action);
            }

            // Trade detail link
            const link = e.target.closest('.td-trade-link');
            if (link) {
                e.preventDefault();
                const id = link.dataset.tradeId;
                if (id) openTradeDetail(id);
            }
        });

        // Pagination
        $('#td-active-prev')?.addEventListener('click', () => { if (state.activePage > 1) { state.activePage--; loadActiveTrades(); } });
        $('#td-active-next')?.addEventListener('click', () => { state.activePage++; loadActiveTrades(); });
        $('#td-history-prev')?.addEventListener('click', () => { if (state.historyPage > 1) { state.historyPage--; loadTradeHistory(); } });
        $('#td-history-next')?.addEventListener('click', () => { state.historyPage++; loadTradeHistory(); });

        // Trade detail modal close
        $('#td-detail-close')?.addEventListener('click', closeTradeDetail);
        $('#td-detail-modal')?.addEventListener('click', (e) => { if (e.target === e.currentTarget) closeTradeDetail(); });
        document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeTradeDetail(); });
    }

    // ============================================
    // Trade Detail Modal
    // ============================================
    async function openTradeDetail(tradeId) {
        const modal = $('#td-detail-modal');
        const body = $('#td-detail-body');
        if (!modal || !body) return;

        body.innerHTML = '<p class="td-loading">Loading trade details…</p>';
        modal.hidden = false;
        modal.focus();

        try {
            const trade = await apiFetch(`/trades/${tradeId}`);
            body.innerHTML = renderTradeDetailHtml(trade);
        } catch (e) {
            body.innerHTML = `<p class="td-error-state">Failed to load trade #${escapeHtml(tradeId)}</p>`;
        }
    }

    function closeTradeDetail() {
        const modal = $('#td-detail-modal');
        if (modal) modal.hidden = true;
    }

    function renderTradeDetailHtml(t) {
        return `
            <dl class="td-detail-grid">
                <dt>Trade ID</dt><dd>#${escapeHtml(t.id)}</dd>
                <dt>Status</dt><dd>${statusBadge(t.status)}</dd>
                <dt>Amount</dt><dd>${escapeHtml(formatAmount(t.amount))} USDC</dd>
                <dt>Buyer</dt><dd class="td-addr-full">${escapeHtml(t.buyer ?? '—')}</dd>
                <dt>Seller</dt><dd class="td-addr-full">${escapeHtml(t.seller ?? '—')}</dd>
                <dt>Arbitrator</dt><dd class="td-addr-full">${escapeHtml(t.arbitrator ?? 'None')}</dd>
                <dt>Created</dt><dd>${escapeHtml(formatDate(t.created_at))}</dd>
                <dt>Updated</dt><dd>${escapeHtml(formatDate(t.updated_at))}</dd>
            </dl>
            <div class="td-detail-actions">${quickActions(t)}</div>
        `;
    }

    // ============================================
    // Init
    // ============================================
    function init() {
        bindEvents();
        loadActiveTrades();
        loadTradeHistory();
        connectDashboardWs();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
