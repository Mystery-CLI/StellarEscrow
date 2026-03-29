/**
 * StellarEscrow - Transaction History (#247)
 * Full history view: filtering, sorting, detail modal, CSV + PDF export.
 */
(function () {
    'use strict';

    const API = '/api';

    const state = {
        records: [],
        filter: { status: '', dateFrom: '', dateTo: '', sort: 'date_desc', search: '' },
        page: 1,
        pageSize: 20,
        total: 0,
    };

    // ── DOM helpers ──────────────────────────────────────────────────────────
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
        return new Date(ts).toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
    }

    function shortAddr(addr) {
        if (!addr || addr.length < 10) return addr ?? '—';
        return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
    }

    function statusBadge(status) {
        const label = status ?? '—';
        return `<span class="td-status-badge td-status-${label.toLowerCase()}" aria-label="Status: ${escapeHtml(label)}">${escapeHtml(label)}</span>`;
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

    // ── API ──────────────────────────────────────────────────────────────────
    async function apiFetch(path) {
        const res = await fetch(`${API}${path}`);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
    }

    async function loadHistory() {
        const tbody = $('#hx-tbody');
        if (tbody) tbody.innerHTML = `<tr><td colspan="6" class="td-loading">Loading…</td></tr>`;

        try {
            const { status, dateFrom, dateTo, sort, search } = state.filter;
            const params = new URLSearchParams({
                limit: state.pageSize,
                offset: (state.page - 1) * state.pageSize,
            });
            if (status)   params.set('status', status);
            if (dateFrom) params.set('from', new Date(dateFrom).toISOString());
            if (dateTo)   params.set('to', new Date(dateTo).toISOString());
            if (sort)     params.set('sort', sort);
            if (search)   params.set('q', search);

            const data = await apiFetch(`/trades/history?${params}`);
            state.records = Array.isArray(data) ? data : (data.trades ?? []);
            state.total   = data.total ?? state.records.length;
            render();
        } catch (e) {
            console.error('Failed to load history', e);
            if (tbody) tbody.innerHTML = `<tr><td colspan="6" class="td-error-state">Failed to load. <button class="btn btn-sm btn-ghost" id="hx-retry">Retry</button></td></tr>`;
            $('#hx-retry')?.addEventListener('click', loadHistory);
        }
    }

    async function loadTradeDetail(tradeId) {
        try {
            const trade = await apiFetch(`/trades/${tradeId}`);
            showDetailModal(trade);
        } catch (e) {
            showToast(`Could not load trade #${tradeId}: ${e.message}`, 'error');
        }
    }

    // ── Render ───────────────────────────────────────────────────────────────
    function render() {
        const tbody = $('#hx-tbody');
        const empty = $('#hx-empty');
        if (!tbody) return;

        const rows = applyClientSort(state.records, state.filter);

        if (rows.length === 0) {
            tbody.innerHTML = '';
            if (empty) empty.hidden = false;
            updatePagination(0);
            return;
        }
        if (empty) empty.hidden = true;

        tbody.innerHTML = rows.map(t => `
            <tr class="hx-row" data-trade-id="${escapeHtml(t.id)}" tabindex="0" role="button" aria-label="View details for trade ${escapeHtml(t.id)}">
                <td data-label="Date">${escapeHtml(formatDate(t.created_at ?? t.updated_at))}</td>
                <td data-label="ID"><span class="hx-trade-id">#${escapeHtml(t.id)}</span></td>
                <td data-label="Status">${statusBadge(t.status)}</td>
                <td data-label="Amount" class="td-amount">${escapeHtml(formatAmount(t.amount))} USDC</td>
                <td data-label="Counterparty" class="td-addr" title="${escapeHtml(t.buyer ?? '')} / ${escapeHtml(t.seller ?? '')}">
                    B: ${escapeHtml(shortAddr(t.buyer))} / S: ${escapeHtml(shortAddr(t.seller))}
                </td>
                <td data-label="Arbitrator" class="td-addr">${t.arbitrator ? escapeHtml(shortAddr(t.arbitrator)) : '—'}</td>
            </tr>
        `).join('');

        // Row click / keyboard → detail
        $$('.hx-row', tbody).forEach(row => {
            const open = () => loadTradeDetail(row.dataset.tradeId);
            row.addEventListener('click', open);
            row.addEventListener('keydown', e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); open(); } });
        });

        updatePagination(state.total);
    }

    function updatePagination(total) {
        const info = $('#hx-page-info');
        const prev = $('#hx-prev');
        const next = $('#hx-next');
        const totalPages = Math.max(1, Math.ceil(total / state.pageSize));
        if (info) info.textContent = `Page ${state.page} of ${totalPages} (${total} records)`;
        if (prev) prev.disabled = state.page <= 1;
        if (next) next.disabled = state.page >= totalPages;
    }

    // ── Detail Modal ─────────────────────────────────────────────────────────
    function showDetailModal(t) {
        const modal = $('#hx-detail-modal');
        const body  = $('#hx-detail-body');
        if (!modal || !body) return;

        const arb = t.arbitrator
            ? `<span class="td-addr-full">${escapeHtml(t.arbitrator)}</span>`
            : '—';

        const timeline = buildTimeline(t);

        body.innerHTML = `
            <dl class="td-detail-grid">
                <dt>Trade ID</dt>   <dd>#${escapeHtml(t.id)}</dd>
                <dt>Status</dt>     <dd>${statusBadge(t.status)}</dd>
                <dt>Amount</dt>     <dd class="td-amount">${escapeHtml(formatAmount(t.amount))} USDC</dd>
                <dt>Fee</dt>        <dd>${escapeHtml(formatAmount(t.fee ?? 0))} USDC</dd>
                <dt>Buyer</dt>      <dd class="td-addr-full">${escapeHtml(t.buyer ?? '—')}</dd>
                <dt>Seller</dt>     <dd class="td-addr-full">${escapeHtml(t.seller ?? '—')}</dd>
                <dt>Arbitrator</dt> <dd>${arb}</dd>
                <dt>Created</dt>    <dd>${escapeHtml(formatDate(t.created_at))}</dd>
                <dt>Updated</dt>    <dd>${escapeHtml(formatDate(t.updated_at))}</dd>
                ${t.expiry_time ? `<dt>Expires</dt><dd>${escapeHtml(formatDate(t.expiry_time))}</dd>` : ''}
            </dl>
            <h4 class="hx-timeline-heading">Event Timeline</h4>
            <ol class="hx-timeline" aria-label="Trade event timeline">${timeline}</ol>
        `;

        modal.hidden = false;
        modal.focus();
    }

    function buildTimeline(t) {
        // Use events array if API returns it, otherwise synthesise from status
        const events = Array.isArray(t.events) ? t.events : synthesiseEvents(t);
        if (!events.length) return '<li class="hx-timeline-item">No events recorded.</li>';
        return events.map(ev => `
            <li class="hx-timeline-item">
                <span class="hx-timeline-dot" aria-hidden="true"></span>
                <span class="hx-timeline-label">${escapeHtml(ev.type ?? ev.event_type ?? '—')}</span>
                <span class="hx-timeline-time">${escapeHtml(formatDate(ev.timestamp ?? ev.created_at))}</span>
            </li>
        `).join('');
    }

    function synthesiseEvents(t) {
        const evs = [{ type: 'Created', timestamp: t.created_at }];
        const map = { Funded: 'Funded', Completed: 'Completed', Disputed: 'Disputed', Cancelled: 'Cancelled' };
        if (t.status && map[t.status]) evs.push({ type: map[t.status], timestamp: t.updated_at });
        return evs;
    }

    // ── Client-side sort (supplement server sort) ────────────────────────────
    function applyClientSort(records, filter) {
        const [field, dir] = (filter.sort ?? 'date_desc').split('_');
        return [...records].sort((a, b) => {
            let av, bv;
            if (field === 'amount') {
                av = Number(a.amount); bv = Number(b.amount);
            } else if (field === 'status') {
                av = a.status ?? ''; bv = b.status ?? '';
                return dir === 'asc' ? av.localeCompare(bv) : bv.localeCompare(av);
            } else {
                av = new Date(a.created_at ?? 0).getTime();
                bv = new Date(b.created_at ?? 0).getTime();
            }
            return dir === 'asc' ? av - bv : bv - av;
        });
    }

    // ── Export ───────────────────────────────────────────────────────────────
    function exportCSV() {
        const rows = applyClientSort(state.records, state.filter);
        const header = 'ID,Date,Status,Amount (USDC),Fee (USDC),Buyer,Seller,Arbitrator\n';
        const body = rows.map(t => [
            t.id,
            formatDate(t.created_at ?? t.updated_at),
            t.status,
            formatAmount(t.amount),
            formatAmount(t.fee ?? 0),
            t.buyer ?? '',
            t.seller ?? '',
            t.arbitrator ?? '',
        ].map(v => `"${String(v).replace(/"/g, '""')}"`).join(',')).join('\n');

        download('transaction-history.csv', 'text/csv', header + body);
        showToast('CSV exported', 'success');
    }

    function exportPDF() {
        const rows = applyClientSort(state.records, state.filter);
        const tableRows = rows.map(t => `
            <tr>
                <td>#${escapeHtml(t.id)}</td>
                <td>${escapeHtml(formatDate(t.created_at ?? t.updated_at))}</td>
                <td>${escapeHtml(t.status)}</td>
                <td>${escapeHtml(formatAmount(t.amount))}</td>
                <td>${escapeHtml(formatAmount(t.fee ?? 0))}</td>
                <td style="font-size:9px">${escapeHtml(t.buyer ?? '')}</td>
                <td style="font-size:9px">${escapeHtml(t.seller ?? '')}</td>
            </tr>`).join('');

        const html = `<!DOCTYPE html><html><head><meta charset="UTF-8">
            <title>Transaction History</title>
            <style>
                body { font-family: sans-serif; font-size: 11px; margin: 24px; }
                h1 { font-size: 16px; margin-bottom: 12px; }
                table { border-collapse: collapse; width: 100%; }
                th, td { border: 1px solid #ccc; padding: 4px 8px; text-align: left; }
                th { background: #f0f0f0; }
            </style></head><body>
            <h1>StellarEscrow — Transaction History</h1>
            <p>Exported: ${new Date().toLocaleString()}</p>
            <table>
                <thead><tr><th>ID</th><th>Date</th><th>Status</th><th>Amount</th><th>Fee</th><th>Buyer</th><th>Seller</th></tr></thead>
                <tbody>${tableRows}</tbody>
            </table></body></html>`;

        const win = window.open('', '_blank');
        if (!win) { showToast('Allow pop-ups to export PDF', 'error'); return; }
        win.document.write(html);
        win.document.close();
        win.focus();
        win.print();
        showToast('PDF print dialog opened', 'success');
    }

    function download(filename, mime, content) {
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob([content], { type: mime }));
        a.download = filename;
        a.click();
        URL.revokeObjectURL(a.href);
    }

    // ── Event wiring ─────────────────────────────────────────────────────────
    function init() {
        // Filter form
        $('#hx-filter-form')?.addEventListener('submit', e => {
            e.preventDefault();
            state.filter.status   = $('#hx-status-filter')?.value ?? '';
            state.filter.dateFrom = $('#hx-date-from')?.value ?? '';
            state.filter.dateTo   = $('#hx-date-to')?.value ?? '';
            state.filter.sort     = $('#hx-sort')?.value ?? 'date_desc';
            state.filter.search   = $('#hx-search')?.value ?? '';
            state.page = 1;
            loadHistory();
        });

        $('#hx-filter-form')?.addEventListener('reset', () => {
            state.filter = { status: '', dateFrom: '', dateTo: '', sort: 'date_desc', search: '' };
            state.page = 1;
            loadHistory();
        });

        // Pagination
        $('#hx-prev')?.addEventListener('click', () => { if (state.page > 1) { state.page--; loadHistory(); } });
        $('#hx-next')?.addEventListener('click', () => { state.page++; loadHistory(); });

        // Export
        $('#hx-export-csv')?.addEventListener('click', exportCSV);
        $('#hx-export-pdf')?.addEventListener('click', exportPDF);

        // Detail modal close
        const closeModal = () => { const m = $('#hx-detail-modal'); if (m) m.hidden = true; };
        $('#hx-detail-close')?.addEventListener('click', closeModal);
        $('#hx-detail-modal')?.addEventListener('keydown', e => { if (e.key === 'Escape') closeModal(); });
        $('#hx-detail-modal')?.addEventListener('click', e => { if (e.target === e.currentTarget) closeModal(); });

        // Nav link activation
        document.querySelectorAll('a[href="#history"]').forEach(a => {
            a.addEventListener('click', () => {
                if (state.records.length === 0) loadHistory();
            });
        });

        // Initial load if section is visible on page load
        if (location.hash === '#history') loadHistory();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
