/**
 * StellarEscrow - Trade Creation Interface (#238)
 * Form → Validate → Preview → Confirm → Submit
 */
(function () {
    'use strict';

    const API = '/api';

    // Supported currencies (extend as contract adds more SAC tokens)
    const CURRENCIES = [
        { value: 'USDC', label: 'USDC' },
        { value: 'EURC', label: 'EURC' },
        { value: 'XLM',  label: 'XLM'  },
    ];

    const state = {
        step: 'form',   // 'form' | 'preview' | 'submitting' | 'success' | 'error'
        arbitrators: [],
        form: { buyer: '', seller: '', amount: '', currency: 'USDC', arbitrator: '', description: '' },
        preview: null,
        result: null,
    };

    // ── DOM helpers ──────────────────────────────────────────────────────────
    const $ = (sel, ctx = document) => ctx.querySelector(sel);
    function escapeHtml(str) {
        const d = document.createElement('div');
        d.textContent = String(str ?? '');
        return d.innerHTML;
    }
    function formatAmount(val, currency) {
        return `${Number(val).toLocaleString('en-US', { minimumFractionDigits: 2 })} ${currency}`;
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

    // ── Validation ───────────────────────────────────────────────────────────
    const STELLAR_ADDR_RE = /^G[A-Z2-7]{55}$/;

    function validate(form) {
        const errors = {};
        if (!STELLAR_ADDR_RE.test(form.buyer))
            errors.buyer = 'Must be a valid Stellar address (G… 56 chars)';
        if (!STELLAR_ADDR_RE.test(form.seller))
            errors.seller = 'Must be a valid Stellar address (G… 56 chars)';
        if (form.buyer && form.seller && form.buyer === form.seller)
            errors.buyer = errors.seller = 'Buyer and seller must be different addresses';
        const amt = parseFloat(form.amount);
        if (!form.amount || isNaN(amt) || amt <= 0)
            errors.amount = 'Amount must be a positive number';
        if (amt > 1_000_000_000)
            errors.amount = 'Amount exceeds maximum allowed (1,000,000,000)';
        if (!form.arbitrator)
            errors.arbitrator = 'Please select an arbitrator';
        return errors;
    }

    function showErrors(errors) {
        // Clear previous
        document.querySelectorAll('.tc-field-error').forEach(el => { el.textContent = ''; el.hidden = true; });
        document.querySelectorAll('.tc-input-error').forEach(el => el.classList.remove('tc-input-error'));
        Object.entries(errors).forEach(([field, msg]) => {
            const errEl = $(`#tc-err-${field}`);
            const inputEl = $(`#tc-${field}`);
            if (errEl) { errEl.textContent = msg; errEl.hidden = false; }
            if (inputEl) inputEl.classList.add('tc-input-error');
        });
    }

    // ── API ──────────────────────────────────────────────────────────────────
    async function loadArbitrators() {
        try {
            const res = await fetch(`${API}/arbitrators`);
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            const data = await res.json();
            state.arbitrators = Array.isArray(data) ? data : (data.arbitrators ?? []);
        } catch (e) {
            console.warn('Could not load arbitrators from API, using empty list', e);
            state.arbitrators = [];
        }
        populateArbitratorSelect();
    }

    function populateArbitratorSelect() {
        const sel = $('#tc-arbitrator');
        if (!sel) return;
        const current = sel.value;
        sel.innerHTML = '<option value="">— Select arbitrator —</option>' +
            state.arbitrators.map(a => {
                const addr = typeof a === 'string' ? a : (a.address ?? a.id ?? '');
                const label = typeof a === 'string' ? shortAddr(a) : (a.name ?? shortAddr(addr));
                return `<option value="${escapeHtml(addr)}">${escapeHtml(label)}</option>`;
            }).join('');
        if (current) sel.value = current;
    }

    async function submitTrade(form) {
        const walletAddr = getWalletAddress();
        const body = {
            buyer:       form.buyer,
            seller:      form.seller,
            amount:      String(Math.round(parseFloat(form.amount) * 1e7)), // stroops
            currency:    form.currency,
            arbitrator:  form.arbitrator || undefined,
            description: form.description || undefined,
        };

        // If wallet is connected, attempt to sign; otherwise submit unsigned (server signs)
        if (walletAddr && window.StellarWallet?.signTransaction) {
            try {
                const buildRes = await fetch(`${API}/trades/build`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body),
                });
                if (buildRes.ok) {
                    const { xdr } = await buildRes.json();
                    const signed = await window.StellarWallet.signTransaction(xdr);
                    if (signed?.signedXdr) {
                        const submitRes = await fetch(`${API}/trades/submit`, {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ signedXdr: signed.signedXdr }),
                        });
                        if (!submitRes.ok) throw new Error(`HTTP ${submitRes.status}`);
                        return submitRes.json();
                    }
                }
            } catch (e) {
                console.warn('Wallet signing failed, falling back to direct submit', e);
            }
        }

        // Direct submit (server-side signing / testnet)
        const res = await fetch(`${API}/trades`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            const err = await res.json().catch(() => ({}));
            throw new Error(err.message ?? err.error ?? `HTTP ${res.status}`);
        }
        return res.json();
    }

    function getWalletAddress() {
        // Reads from wallet.js state exposed on window, or localStorage fallback
        return window.StellarWallet?.address
            ?? localStorage.getItem('stellar_wallet_address')
            ?? null;
    }

    // ── Render steps ─────────────────────────────────────────────────────────
    function renderForm() {
        const panel = $('#tc-panel');
        if (!panel) return;
        const walletAddr = getWalletAddress();

        panel.innerHTML = `
            <form id="tc-form" class="tc-form" novalidate aria-label="Create trade form">
                <div class="tc-field">
                    <label for="tc-seller" class="tc-label">Seller Address <span aria-hidden="true">*</span></label>
                    <input id="tc-seller" type="text" class="filter-input tc-addr-input"
                        placeholder="G…" maxlength="56" autocomplete="off" spellcheck="false"
                        value="${escapeHtml(state.form.seller || walletAddr || '')}"
                        aria-required="true" aria-describedby="tc-err-seller">
                    <span id="tc-err-seller" class="tc-field-error" role="alert" hidden></span>
                </div>

                <div class="tc-field">
                    <label for="tc-buyer" class="tc-label">Buyer Address <span aria-hidden="true">*</span></label>
                    <input id="tc-buyer" type="text" class="filter-input tc-addr-input"
                        placeholder="G…" maxlength="56" autocomplete="off" spellcheck="false"
                        value="${escapeHtml(state.form.buyer)}"
                        aria-required="true" aria-describedby="tc-err-buyer">
                    <span id="tc-err-buyer" class="tc-field-error" role="alert" hidden></span>
                </div>

                <div class="tc-field tc-amount-row">
                    <div class="tc-amount-group">
                        <label for="tc-amount" class="tc-label">Amount <span aria-hidden="true">*</span></label>
                        <input id="tc-amount" type="number" class="filter-input"
                            placeholder="0.00" min="0.0000001" step="any"
                            value="${escapeHtml(state.form.amount)}"
                            aria-required="true" aria-describedby="tc-err-amount">
                        <span id="tc-err-amount" class="tc-field-error" role="alert" hidden></span>
                    </div>
                    <div class="tc-currency-group">
                        <label for="tc-currency" class="tc-label">Currency</label>
                        <select id="tc-currency" class="filter-select" aria-label="Select currency">
                            ${CURRENCIES.map(c => `<option value="${c.value}"${state.form.currency === c.value ? ' selected' : ''}>${c.label}</option>`).join('')}
                        </select>
                    </div>
                </div>

                <div class="tc-field">
                    <label for="tc-arbitrator" class="tc-label">Arbitrator <span aria-hidden="true">*</span></label>
                    <select id="tc-arbitrator" class="filter-select tc-arb-select"
                        aria-required="true" aria-describedby="tc-err-arbitrator">
                        <option value="">— Select arbitrator —</option>
                    </select>
                    <span id="tc-err-arbitrator" class="tc-field-error" role="alert" hidden></span>
                </div>

                <div class="tc-field">
                    <label for="tc-description" class="tc-label">Description / Reference <span class="tc-optional">(optional)</span></label>
                    <input id="tc-description" type="text" class="filter-input"
                        placeholder="Order #, product name, etc."
                        maxlength="256" value="${escapeHtml(state.form.description)}">
                </div>

                <div class="tc-actions">
                    <button type="submit" class="btn btn-primary">Preview Trade →</button>
                </div>
            </form>
        `;

        populateArbitratorSelect();
        if (state.form.arbitrator) $('#tc-arbitrator').value = state.form.arbitrator;

        $('#tc-form').addEventListener('submit', e => {
            e.preventDefault();
            state.form.seller      = $('#tc-seller').value.trim();
            state.form.buyer       = $('#tc-buyer').value.trim();
            state.form.amount      = $('#tc-amount').value.trim();
            state.form.currency    = $('#tc-currency').value;
            state.form.arbitrator  = $('#tc-arbitrator').value;
            state.form.description = $('#tc-description').value.trim();

            const errors = validate(state.form);
            if (Object.keys(errors).length) { showErrors(errors); return; }

            // Build preview
            const amt = parseFloat(state.form.amount);
            const feeBps = 50; // 0.5% default; server will confirm exact fee
            const fee = (amt * feeBps / 10000).toFixed(7);
            state.preview = { ...state.form, estimatedFee: fee };
            state.step = 'preview';
            renderPreview();
        });
    }

    function renderPreview() {
        const panel = $('#tc-panel');
        if (!panel) return;
        const p = state.preview;
        const arb = state.arbitrators.find(a => {
            const addr = typeof a === 'string' ? a : (a.address ?? a.id ?? '');
            return addr === p.arbitrator;
        });
        const arbLabel = arb
            ? (typeof arb === 'string' ? shortAddr(arb) : (arb.name ?? shortAddr(arb.address ?? arb.id ?? '')))
            : shortAddr(p.arbitrator);

        panel.innerHTML = `
            <div class="tc-preview" role="region" aria-label="Trade preview">
                <p class="tc-preview-note">Review the details below before submitting.</p>
                <dl class="td-detail-grid tc-preview-grid">
                    <dt>Seller</dt>   <dd class="td-addr-full">${escapeHtml(p.seller)}</dd>
                    <dt>Buyer</dt>    <dd class="td-addr-full">${escapeHtml(p.buyer)}</dd>
                    <dt>Amount</dt>   <dd class="td-amount">${escapeHtml(formatAmount(p.amount, p.currency))}</dd>
                    <dt>Est. Fee</dt> <dd class="td-amount tc-fee-note">${escapeHtml(formatAmount(p.estimatedFee, p.currency))} <span class="tc-fee-hint">(~0.5%, confirmed on-chain)</span></dd>
                    <dt>Arbitrator</dt><dd>${escapeHtml(arbLabel)}</dd>
                    ${p.description ? `<dt>Description</dt><dd>${escapeHtml(p.description)}</dd>` : ''}
                </dl>
                <div class="tc-actions">
                    <button id="tc-edit-btn" type="button" class="btn btn-secondary">← Edit</button>
                    <button id="tc-confirm-btn" type="button" class="btn btn-primary">Confirm &amp; Submit</button>
                </div>
            </div>
        `;

        $('#tc-edit-btn').addEventListener('click', () => { state.step = 'form'; renderForm(); });
        $('#tc-confirm-btn').addEventListener('click', handleConfirm);
    }

    async function handleConfirm() {
        state.step = 'submitting';
        const panel = $('#tc-panel');
        if (panel) panel.innerHTML = `
            <div class="tc-state-panel" role="status" aria-live="polite">
                <span class="tc-spinner" aria-hidden="true"></span>
                <p>Submitting trade…</p>
            </div>`;

        try {
            const result = await submitTrade(state.form);
            state.result = result;
            state.step = 'success';
            renderSuccess(result);
        } catch (e) {
            state.step = 'error';
            renderError(e.message);
        }
    }

    function renderSuccess(result) {
        const panel = $('#tc-panel');
        if (!panel) return;
        const tradeId = result.id ?? result.trade_id ?? '—';
        const txHash  = result.txHash ?? result.tx_hash ?? '';

        panel.innerHTML = `
            <div class="tc-state-panel tc-success" role="status" aria-live="polite">
                <span class="tc-success-icon" aria-hidden="true">✓</span>
                <h3 class="tc-state-title">Trade Created!</h3>
                <p>Trade <strong>#${escapeHtml(tradeId)}</strong> has been submitted successfully.</p>
                ${txHash ? `<p class="tc-tx-hash">Tx: <code>${escapeHtml(txHash)}</code></p>` : ''}
                <div class="tc-actions">
                    <button id="tc-new-btn" type="button" class="btn btn-primary">Create Another Trade</button>
                </div>
            </div>
        `;
        showToast(`Trade #${tradeId} created successfully`, 'success');
        $('#tc-new-btn').addEventListener('click', resetForm);
    }

    function renderError(message) {
        const panel = $('#tc-panel');
        if (!panel) return;
        panel.innerHTML = `
            <div class="tc-state-panel tc-error-panel" role="alert" aria-live="assertive">
                <span class="tc-error-icon" aria-hidden="true">✕</span>
                <h3 class="tc-state-title">Submission Failed</h3>
                <p class="tc-error-msg">${escapeHtml(message)}</p>
                <div class="tc-actions">
                    <button id="tc-back-btn" type="button" class="btn btn-secondary">← Back to Preview</button>
                    <button id="tc-retry-btn" type="button" class="btn btn-primary">Retry</button>
                </div>
            </div>
        `;
        $('#tc-back-btn').addEventListener('click', () => { state.step = 'preview'; renderPreview(); });
        $('#tc-retry-btn').addEventListener('click', handleConfirm);
    }

    function resetForm() {
        state.step = 'form';
        state.form = { buyer: '', seller: '', amount: '', currency: 'USDC', arbitrator: '', description: '' };
        state.preview = null;
        state.result = null;
        renderForm();
    }

    // ── Init ─────────────────────────────────────────────────────────────────
    function init() {
        if (!$('#tc-panel')) return;
        loadArbitrators();
        renderForm();

        // Pre-fill seller from connected wallet
        document.addEventListener('wallet:connected', e => {
            if (state.step === 'form' && !state.form.seller) {
                state.form.seller = e.detail?.address ?? '';
                renderForm();
            }
        });

        // Reload arbitrators when section becomes visible
        document.querySelectorAll('a[href="#create-trade"]').forEach(a => {
            a.addEventListener('click', () => {
                if (state.arbitrators.length === 0) loadArbitrators();
            });
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
