/**
 * Evidence Storage Service (Issue #124)
 * Integrates with POST /disputes/:id/evidence and signed download URL endpoints.
 */

(function () {
    'use strict';

    const INDEXER_BASE = window.INDEXER_BASE_URL || '';

    // Allowed MIME types and max size must match the backend StorageService
    const ALLOWED_MIMES = ['image/jpeg', 'image/png', 'image/webp', 'application/pdf'];
    const MAX_SIZE_BYTES = 20 * 1024 * 1024; // 20 MB

    /**
     * Client-side validation before upload.
     * @param {File} file
     * @returns {{ valid: boolean, error?: string }}
     */
    function validateEvidenceFile(file) {
        if (file.size > MAX_SIZE_BYTES) {
            return { valid: false, error: `File "${file.name}" exceeds the 20 MB limit.` };
        }
        if (!ALLOWED_MIMES.includes(file.type)) {
            return {
                valid: false,
                error: `File type "${file.type}" is not allowed. Use JPEG, PNG, WebP, or PDF.`,
            };
        }
        return { valid: true };
    }

    /**
     * Upload a single evidence file for a dispute.
     * @param {number|string} disputeId  - trade_id acting as dispute identifier
     * @param {File}          file
     * @param {string}        uploader   - Stellar address of the uploader
     * @param {string}        [description]
     * @returns {Promise<{ success: boolean, evidence?: object, error?: string }>}
     */
    async function uploadEvidence(disputeId, file, uploader, description = '') {
        const validation = validateEvidenceFile(file);
        if (!validation.valid) {
            return { success: false, error: validation.error };
        }

        const formData = new FormData();
        formData.append('file', file, file.name);
        formData.append('uploader', uploader);
        if (description.trim()) {
            formData.append('description', description.trim());
        }

        try {
            const res = await fetch(`${INDEXER_BASE}/disputes/${disputeId}/evidence`, {
                method: 'POST',
                body: formData,
            });

            if (!res.ok) {
                const body = await res.json().catch(() => ({}));
                return { success: false, error: body.error || `Upload failed (HTTP ${res.status})` };
            }

            const data = await res.json();
            return { success: true, evidence: data };
        } catch (err) {
            return { success: false, error: err.message };
        }
    }

    /**
     * Fetch all evidence records for a dispute.
     * @param {number|string} disputeId
     * @param {string}        requester - Stellar address of the viewer
     * @returns {Promise<{ success: boolean, evidence?: object[], error?: string }>}
     */
    async function listEvidence(disputeId, requester) {
        try {
            const res = await fetch(
                `${INDEXER_BASE}/disputes/${disputeId}/evidence?requester=${encodeURIComponent(requester)}`
            );
            if (!res.ok) {
                const body = await res.json().catch(() => ({}));
                return { success: false, error: body.error || `Fetch failed (HTTP ${res.status})` };
            }
            const data = await res.json();
            return { success: true, evidence: data.evidence || [] };
        } catch (err) {
            return { success: false, error: err.message };
        }
    }

    /**
     * Obtain a short-lived signed download URL for a specific evidence item.
     * @param {number|string} disputeId
     * @param {string}        evidenceId - UUID of the evidence record
     * @param {string}        requester
     * @returns {Promise<{ success: boolean, downloadUrl?: string, expiresAt?: string, error?: string }>}
     */
    async function getDownloadUrl(disputeId, evidenceId, requester) {
        try {
            const res = await fetch(
                `${INDEXER_BASE}/disputes/${disputeId}/evidence/${evidenceId}/download-url?requester=${encodeURIComponent(requester)}`
            );
            if (!res.ok) {
                const body = await res.json().catch(() => ({}));
                return { success: false, error: body.error || `Token request failed (HTTP ${res.status})` };
            }
            const data = await res.json();
            return {
                success: true,
                downloadUrl: `${INDEXER_BASE}${data.download_url}`,
                expiresAt: data.expires_at,
            };
        } catch (err) {
            return { success: false, error: err.message };
        }
    }

    // -------------------------------------------------------------------------
    // UI helpers
    // -------------------------------------------------------------------------

    /**
     * Render the evidence list into a container element.
     * Each item gets a "Download" button that fetches a signed URL on demand.
     *
     * @param {HTMLElement}   container
     * @param {object[]}      evidenceItems  - from listEvidence()
     * @param {number|string} disputeId
     * @param {string}        requester
     */
    function renderEvidenceList(container, evidenceItems, disputeId, requester) {
        if (!container) return;

        if (!evidenceItems || evidenceItems.length === 0) {
            container.innerHTML = '<p class="empty-state">No evidence uploaded yet.</p>';
            return;
        }

        container.innerHTML = evidenceItems
            .map(
                (item) => `
            <div class="evidence-item" data-evidence-id="${item.id}">
                <div class="evidence-meta">
                    <span class="evidence-name">${escapeHtml(item.file?.original_name || 'File')}</span>
                    <span class="evidence-uploader">by ${formatAddress(item.uploader)}</span>
                    ${item.description ? `<span class="evidence-desc">${escapeHtml(item.description)}</span>` : ''}
                    <span class="evidence-date">${formatTimestamp(item.created_at)}</span>
                </div>
                <button
                    type="button"
                    class="btn btn-sm btn-secondary evidence-download-btn"
                    data-evidence-id="${item.id}"
                    aria-label="Download evidence file"
                >
                    Download
                </button>
            </div>`
            )
            .join('');

        container.querySelectorAll('.evidence-download-btn').forEach((btn) => {
            btn.addEventListener('click', async () => {
                btn.disabled = true;
                btn.textContent = 'Getting link…';
                const result = await getDownloadUrl(disputeId, btn.dataset.evidenceId, requester);
                if (result.success) {
                    // Open in new tab — browser handles the download
                    window.open(result.downloadUrl, '_blank', 'noopener');
                } else {
                    alert(`Could not get download link: ${result.error}`);
                }
                btn.disabled = false;
                btn.textContent = 'Download';
            });
        });
    }

    // -------------------------------------------------------------------------
    // Private helpers (duplicated from disputes.js to keep module self-contained)
    // -------------------------------------------------------------------------

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = String(text);
        return div.innerHTML;
    }

    function formatAddress(addr) {
        if (!addr || addr.length < 12) return addr || '';
        return `${addr.slice(0, 6)}…${addr.slice(-6)}`;
    }

    function formatTimestamp(ts) {
        return new Date(ts).toLocaleString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        });
    }

    // -------------------------------------------------------------------------
    // Public API
    // -------------------------------------------------------------------------

    window.EvidenceService = {
        validateEvidenceFile,
        uploadEvidence,
        listEvidence,
        getDownloadUrl,
        renderEvidenceList,
        ALLOWED_MIMES,
        MAX_SIZE_BYTES,
    };

    console.log('Evidence Service module loaded');
})();
