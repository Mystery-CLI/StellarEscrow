/**
 * Dispute Management Interface
 * Handles raising disputes, viewing details, and tracking resolution
 */

(function() {
    'use strict';

    const DISPUTE_API = '/api/disputes';
    const MAX_EVIDENCE_SIZE = 5 * 1024 * 1024; // 5MB

    const disputeState = {
        disputes: [],
        currentDispute: null,
        evidence: [],
        filters: {
            status: 'all',
            tradeId: null
        }
    };

    // ============================================
    // Dispute Management
    // ============================================

    async function raiseDispute(tradeId, reason, evidence = []) {
        try {
            const formData = new FormData();
            formData.append('trade_id', tradeId);
            formData.append('reason', reason);
            
            evidence.forEach((file, index) => {
                formData.append(`evidence_${index}`, file);
            });

            const response = await fetch(`${DISPUTE_API}/raise`, {
                method: 'POST',
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            dispatchDisputeEvent('raised', { disputeId: data.id, tradeId });
            return { success: true, disputeId: data.id };
        } catch (error) {
            console.error('Failed to raise dispute:', error);
            return { success: false, error: error.message };
        }
    }

    async function getDisputeDetails(disputeId) {
        try {
            const response = await fetch(`${DISPUTE_API}/${disputeId}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            disputeState.currentDispute = data;
            return { success: true, dispute: data };
        } catch (error) {
            console.error('Failed to fetch dispute:', error);
            return { success: false, error: error.message };
        }
    }

    async function getDisputesByTrade(tradeId) {
        try {
            const response = await fetch(`${DISPUTE_API}?trade_id=${tradeId}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            return { success: true, disputes: data };
        } catch (error) {
            console.error('Failed to fetch disputes:', error);
            return { success: false, error: error.message };
        }
    }

    async function resolveDispute(disputeId, resolution, notes = '') {
        try {
            const response = await fetch(`${DISPUTE_API}/${disputeId}/resolve`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ resolution, notes })
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            dispatchDisputeEvent('resolved', { disputeId, resolution });
            return { success: true, dispute: data };
        } catch (error) {
            console.error('Failed to resolve dispute:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // Evidence Management
    // ============================================

    function validateEvidence(file) {
        if (file.size > MAX_EVIDENCE_SIZE) {
            return { valid: false, error: 'File exceeds 5MB limit' };
        }
        
        const allowed = ['image/jpeg', 'image/png', 'application/pdf', 'text/plain'];
        if (!allowed.includes(file.type)) {
            return { valid: false, error: 'File type not allowed' };
        }
        
        return { valid: true };
    }

    function addEvidence(file) {
        const validation = validateEvidence(file);
        if (!validation.valid) {
            return { success: false, error: validation.error };
        }

        disputeState.evidence.push({
            id: Date.now(),
            file,
            name: file.name,
            size: file.size,
            type: file.type,
            uploadedAt: new Date().toISOString()
        });

        return { success: true, evidenceId: disputeState.evidence.length - 1 };
    }

    function removeEvidence(evidenceId) {
        disputeState.evidence = disputeState.evidence.filter(e => e.id !== evidenceId);
    }

    // ============================================
    // UI Initialization
    // ============================================

    function initDisputeUI() {
        const raiseBtn = document.getElementById('raise-dispute-btn');
        const form = document.getElementById('dispute-form');
        const evidenceInput = document.getElementById('evidence-input');
        const evidenceList = document.getElementById('evidence-list');

        if (!raiseBtn) return;

        // Open dispute form
        raiseBtn.addEventListener('click', () => {
            const modal = document.getElementById('dispute-modal');
            if (modal) modal.hidden = false;
        });

        // Handle form submission
        if (form) {
            form.addEventListener('submit', async (e) => {
                e.preventDefault();
                
                const tradeId = document.getElementById('dispute-trade-id').value;
                const reason = document.getElementById('dispute-reason').value;
                
                if (!tradeId || !reason) {
                    showToast('Please fill in all required fields', 'error');
                    return;
                }

                const result = await raiseDispute(tradeId, reason, 
                    disputeState.evidence.map(e => e.file));
                
                if (result.success) {
                    showToast('Dispute raised successfully', 'success');
                    form.reset();
                    disputeState.evidence = [];
                    updateEvidenceList();
                    closeDisputeModal();
                } else {
                    showToast(`Error: ${result.error}`, 'error');
                }
            });
        }

        // Handle evidence upload
        if (evidenceInput) {
            evidenceInput.addEventListener('change', (e) => {
                Array.from(e.target.files).forEach(file => {
                    const result = addEvidence(file);
                    if (!result.success) {
                        showToast(result.error, 'error');
                    }
                });
                updateEvidenceList();
                e.target.value = '';
            });
        }

        // Close modal
        const closeBtn = document.getElementById('dispute-modal-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', closeDisputeModal);
        }
    }

    function updateEvidenceList() {
        const list = document.getElementById('evidence-list');
        if (!list) return;

        if (disputeState.evidence.length === 0) {
            list.innerHTML = '<p class="empty-state">No evidence uploaded</p>';
            return;
        }

        list.innerHTML = disputeState.evidence.map(e => `
            <div class="evidence-item">
                <div class="evidence-info">
                    <span class="evidence-name">${escapeHtml(e.name)}</span>
                    <span class="evidence-size">${(e.size / 1024).toFixed(2)} KB</span>
                </div>
                <button 
                    type="button"
                    class="btn btn-sm btn-danger"
                    data-evidence-id="${e.id}"
                    aria-label="Remove evidence"
                >
                    Remove
                </button>
            </div>
        `).join('');

        // Add remove handlers
        list.querySelectorAll('[data-evidence-id]').forEach(btn => {
            btn.addEventListener('click', () => {
                removeEvidence(parseInt(btn.dataset.evidenceId));
                updateEvidenceList();
            });
        });
    }

    function displayDisputeDetails(dispute) {
        const container = document.getElementById('dispute-details');
        if (!container) return;

        const statusClass = `status-${dispute.status.toLowerCase()}`;
        const resolutionText = dispute.resolution ? 
            `Released to ${dispute.resolution.toLowerCase()}` : 'Pending';

        container.innerHTML = `
            <div class="dispute-card">
                <div class="dispute-header">
                    <h3>Dispute #${dispute.id}</h3>
                    <span class="dispute-status ${statusClass}">${dispute.status}</span>
                </div>
                
                <div class="dispute-info">
                    <div class="info-row">
                        <label>Trade ID:</label>
                        <span>${dispute.trade_id}</span>
                    </div>
                    <div class="info-row">
                        <label>Raised By:</label>
                        <span>${formatAddress(dispute.raised_by)}</span>
                    </div>
                    <div class="info-row">
                        <label>Reason:</label>
                        <span>${escapeHtml(dispute.reason)}</span>
                    </div>
                    <div class="info-row">
                        <label>Created:</label>
                        <span>${formatTimestamp(dispute.created_at)}</span>
                    </div>
                </div>

                ${dispute.arbitrator ? `
                    <div class="arbitrator-section">
                        <h4>Arbitrator Assignment</h4>
                        <div class="arbitrator-info">
                            <span class="arbitrator-address">${formatAddress(dispute.arbitrator)}</span>
                            <span class="arbitrator-badge">Assigned</span>
                        </div>
                    </div>
                ` : ''}

                ${dispute.evidence && dispute.evidence.length > 0 ? `
                    <div class="evidence-section">
                        <h4>Evidence</h4>
                        <div class="evidence-files">
                            ${dispute.evidence.map(e => `
                                <a href="${e.url}" class="evidence-link" target="_blank" rel="noopener">
                                    📎 ${escapeHtml(e.name)}
                                </a>
                            `).join('')}
                        </div>
                    </div>
                ` : ''}

                ${dispute.resolution ? `
                    <div class="resolution-section">
                        <h4>Resolution</h4>
                        <div class="resolution-info">
                            <p><strong>Decision:</strong> ${resolutionText}</p>
                            ${dispute.resolution_notes ? `
                                <p><strong>Notes:</strong> ${escapeHtml(dispute.resolution_notes)}</p>
                            ` : ''}
                            <p><strong>Resolved:</strong> ${formatTimestamp(dispute.resolved_at)}</p>
                        </div>
                    </div>
                ` : ''}
            </div>
        `;
    }

    function displayDisputesList(disputes) {
        const container = document.getElementById('disputes-list');
        if (!container) return;

        if (disputes.length === 0) {
            container.innerHTML = '<p class="empty-state">No disputes found</p>';
            return;
        }

        container.innerHTML = disputes.map(d => `
            <div class="dispute-row" data-dispute-id="${d.id}">
                <div class="dispute-col">
                    <span class="dispute-id">Dispute #${d.id}</span>
                    <span class="dispute-trade">Trade #${d.trade_id}</span>
                </div>
                <div class="dispute-col">
                    <span class="dispute-status status-${d.status.toLowerCase()}">${d.status}</span>
                </div>
                <div class="dispute-col">
                    <span class="dispute-date">${formatTimestamp(d.created_at)}</span>
                </div>
                <div class="dispute-col">
                    <button 
                        type="button"
                        class="btn btn-sm btn-primary"
                        data-dispute-id="${d.id}"
                        aria-label="View dispute details"
                    >
                        View
                    </button>
                </div>
            </div>
        `).join('');

        // Add view handlers
        container.querySelectorAll('[data-dispute-id]').forEach(row => {
            const btn = row.querySelector('button');
            if (btn) {
                btn.addEventListener('click', async () => {
                    const result = await getDisputeDetails(row.dataset.disputeId);
                    if (result.success) {
                        displayDisputeDetails(result.dispute);
                    }
                });
            }
        });
    }

    function closeDisputeModal() {
        const modal = document.getElementById('dispute-modal');
        if (modal) modal.hidden = true;
    }

    // ============================================
    // Utilities
    // ============================================

    function formatAddress(address) {
        if (!address) return '';
        return `${address.substring(0, 6)}...${address.substring(address.length - 6)}`;
    }

    function formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.setAttribute('role', 'alert');
        toast.textContent = message;
        container.appendChild(toast);

        setTimeout(() => {
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 300);
        }, 5000);
    }

    function dispatchDisputeEvent(eventType, detail) {
        const event = new CustomEvent(`dispute:${eventType}`, {
            detail,
            bubbles: true,
            cancelable: true
        });
        document.dispatchEvent(event);
    }

    // ============================================
    // Public API
    // ============================================

    window.DisputeManager = {
        raiseDispute,
        getDisputeDetails,
        getDisputesByTrade,
        resolveDispute,
        addEvidence,
        removeEvidence,
        displayDisputeDetails,
        displayDisputesList,
        initDisputeUI,
        getState: () => ({ ...disputeState })
    };

    console.log('Dispute Manager module loaded');
})();
