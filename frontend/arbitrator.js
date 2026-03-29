/**
 * Arbitrator Dashboard
 * Specialized interface for arbitrators to manage disputes and track performance
 */

(function() {
    'use strict';

    const ARBITRATOR_API = '/api/arbitrators';

    const arbitratorState = {
        isArbitrator: false,
        assignedDisputes: [],
        statistics: {
            totalDisputes: 0,
            resolvedDisputes: 0,
            pendingDisputes: 0,
            resolutionRate: 0,
            avgResolutionTime: 0,
            buyerReleaseCount: 0,
            sellerReleaseCount: 0
        },
        performanceMetrics: {
            rating: 0,
            reviewCount: 0,
            joinDate: null,
            totalVolume: 0
        }
    };

    // ============================================
    // Arbitrator Management
    // ============================================

    async function checkArbitratorStatus(address) {
        try {
            const response = await fetch(`${ARBITRATOR_API}/status/${address}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            arbitratorState.isArbitrator = data.is_arbitrator;
            return { success: true, isArbitrator: data.is_arbitrator };
        } catch (error) {
            console.error('Failed to check arbitrator status:', error);
            return { success: false, error: error.message };
        }
    }

    async function getAssignedDisputes() {
        try {
            const response = await fetch(`${ARBITRATOR_API}/disputes`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            arbitratorState.assignedDisputes = data;
            return { success: true, disputes: data };
        } catch (error) {
            console.error('Failed to fetch assigned disputes:', error);
            return { success: false, error: error.message };
        }
    }

    async function getArbitratorStats(address) {
        try {
            const response = await fetch(`${ARBITRATOR_API}/stats/${address}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            arbitratorState.statistics = data;
            return { success: true, stats: data };
        } catch (error) {
            console.error('Failed to fetch arbitrator stats:', error);
            return { success: false, error: error.message };
        }
    }

    async function getPerformanceMetrics(address) {
        try {
            const response = await fetch(`${ARBITRATOR_API}/performance/${address}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            arbitratorState.performanceMetrics = data;
            return { success: true, metrics: data };
        } catch (error) {
            console.error('Failed to fetch performance metrics:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // Resolution Management
    // ============================================

    async function submitResolution(disputeId, resolution, notes) {
        try {
            const response = await fetch(`${ARBITRATOR_API}/disputes/${disputeId}/resolve`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ resolution, notes })
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            dispatchArbitratorEvent('resolved', { disputeId, resolution });
            return { success: true, dispute: data };
        } catch (error) {
            console.error('Failed to submit resolution:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // UI Initialization
    // ============================================

    function initArbitratorDashboard() {
        const dashboard = document.getElementById('arbitrator-dashboard');
        if (!dashboard) return;

        loadDashboardData();
        setupEventListeners();
    }

    async function loadDashboardData() {
        const address = window.StellarWallet?.getAddress();
        if (!address) return;

        // Check arbitrator status
        const statusResult = await checkArbitratorStatus(address);
        if (!statusResult.success || !statusResult.isArbitrator) {
            showArbitratorNotice();
            return;
        }

        // Load disputes
        const disputesResult = await getAssignedDisputes();
        if (disputesResult.success) {
            displayAssignedDisputes(disputesResult.disputes);
        }

        // Load statistics
        const statsResult = await getArbitratorStats(address);
        if (statsResult.success) {
            displayStatistics(statsResult.stats);
        }

        // Load performance metrics
        const metricsResult = await getPerformanceMetrics(address);
        if (metricsResult.success) {
            displayPerformanceMetrics(metricsResult.metrics);
        }
    }

    function setupEventListeners() {
        const resolveButtons = document.querySelectorAll('[data-action="resolve-dispute"]');
        resolveButtons.forEach(btn => {
            btn.addEventListener('click', (e) => {
                const disputeId = e.target.dataset.disputeId;
                openResolutionModal(disputeId);
            });
        });
    }

    function displayAssignedDisputes(disputes) {
        const container = document.getElementById('assigned-disputes');
        if (!container) return;

        if (disputes.length === 0) {
            container.innerHTML = '<p class="empty-state">No assigned disputes</p>';
            return;
        }

        container.innerHTML = disputes.map(d => `
            <div class="dispute-card">
                <div class="dispute-header">
                    <h4>Dispute #${d.id}</h4>
                    <span class="status-badge status-${d.status.toLowerCase()}">${d.status}</span>
                </div>
                
                <div class="dispute-details">
                    <div class="detail-row">
                        <span class="label">Trade ID:</span>
                        <span class="value">#${d.trade_id}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">Raised By:</span>
                        <span class="value">${formatAddress(d.raised_by)}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">Reason:</span>
                        <span class="value">${escapeHtml(d.reason)}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">Created:</span>
                        <span class="value">${formatTimestamp(d.created_at)}</span>
                    </div>
                </div>

                ${d.evidence && d.evidence.length > 0 ? `
                    <div class="evidence-section">
                        <h5>Evidence</h5>
                        <div class="evidence-list">
                            ${d.evidence.map(e => `
                                <a href="${e.url}" class="evidence-link" target="_blank">
                                    📎 ${escapeHtml(e.name)}
                                </a>
                            `).join('')}
                        </div>
                    </div>
                ` : ''}

                ${d.status === 'open' || d.status === 'pending' ? `
                    <div class="dispute-actions">
                        <button 
                            type="button"
                            class="btn btn-primary"
                            data-action="resolve-dispute"
                            data-dispute-id="${d.id}"
                            aria-label="Resolve dispute"
                        >
                            Resolve Dispute
                        </button>
                    </div>
                ` : ''}
            </div>
        `).join('');

        setupEventListeners();
    }

    function displayStatistics(stats) {
        const container = document.getElementById('arbitrator-stats');
        if (!container) return;

        const resolutionRate = stats.totalDisputes > 0 
            ? Math.round((stats.resolvedDisputes / stats.totalDisputes) * 100)
            : 0;

        container.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <h4>Total Disputes</h4>
                    <p class="stat-value">${stats.totalDisputes}</p>
                </div>
                <div class="stat-card">
                    <h4>Resolved</h4>
                    <p class="stat-value">${stats.resolvedDisputes}</p>
                </div>
                <div class="stat-card">
                    <h4>Pending</h4>
                    <p class="stat-value">${stats.pendingDisputes}</p>
                </div>
                <div class="stat-card">
                    <h4>Resolution Rate</h4>
                    <p class="stat-value">${resolutionRate}%</p>
                </div>
                <div class="stat-card">
                    <h4>Avg Resolution Time</h4>
                    <p class="stat-value">${stats.avgResolutionTime} days</p>
                </div>
                <div class="stat-card">
                    <h4>Buyer Releases</h4>
                    <p class="stat-value">${stats.buyerReleaseCount}</p>
                </div>
                <div class="stat-card">
                    <h4>Seller Releases</h4>
                    <p class="stat-value">${stats.sellerReleaseCount}</p>
                </div>
            </div>
        `;
    }

    function displayPerformanceMetrics(metrics) {
        const container = document.getElementById('performance-metrics');
        if (!container) return;

        const stars = '★'.repeat(Math.floor(metrics.rating)) + 
                     '☆'.repeat(5 - Math.floor(metrics.rating));

        container.innerHTML = `
            <div class="metrics-grid">
                <div class="metric-card">
                    <h4>Rating</h4>
                    <div class="rating">
                        <span class="stars">${stars}</span>
                        <span class="rating-value">${metrics.rating.toFixed(1)}/5</span>
                    </div>
                    <p class="metric-detail">${metrics.reviewCount} reviews</p>
                </div>
                <div class="metric-card">
                    <h4>Member Since</h4>
                    <p class="metric-value">${formatDate(metrics.joinDate)}</p>
                </div>
                <div class="metric-card">
                    <h4>Total Volume</h4>
                    <p class="metric-value">$${metrics.totalVolume.toLocaleString()}</p>
                </div>
            </div>
        `;
    }

    function openResolutionModal(disputeId) {
        const modal = document.getElementById('resolution-modal');
        if (!modal) return;

        const dispute = arbitratorState.assignedDisputes.find(d => d.id === parseInt(disputeId));
        if (!dispute) return;

        document.getElementById('resolution-dispute-id').value = disputeId;
        document.getElementById('resolution-dispute-info').innerHTML = `
            <p><strong>Trade #${dispute.trade_id}</strong> - ${escapeHtml(dispute.reason)}</p>
        `;

        modal.hidden = false;
    }

    function closeResolutionModal() {
        const modal = document.getElementById('resolution-modal');
        if (modal) modal.hidden = true;
    }

    function showArbitratorNotice() {
        const dashboard = document.getElementById('arbitrator-dashboard');
        if (!dashboard) return;

        dashboard.innerHTML = `
            <div class="notice notice-info">
                <h3>Arbitrator Access Required</h3>
                <p>You are not registered as an arbitrator. Only registered arbitrators can access this dashboard.</p>
            </div>
        `;
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

    function formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function dispatchArbitratorEvent(eventType, detail) {
        const event = new CustomEvent(`arbitrator:${eventType}`, {
            detail,
            bubbles: true,
            cancelable: true
        });
        document.dispatchEvent(event);
    }

    // ============================================
    // Public API
    // ============================================

    window.ArbitratorDashboard = {
        checkArbitratorStatus,
        getAssignedDisputes,
        getArbitratorStats,
        getPerformanceMetrics,
        submitResolution,
        initArbitratorDashboard,
        getState: () => ({ ...arbitratorState })
    };

    console.log('Arbitrator Dashboard module loaded');
})();
