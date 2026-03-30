/**
 * Complete Integration Example
 * Shows how to use all advanced filtering features together in a real component
 */

import React, { useMemo } from 'react';
import { useSelector } from 'react-redux';
import { AdvancedSearchFilter, applyAdvancedFilters } from '@components';
import { useAdvancedFilters } from '@state';
import type { RootState } from '@state/types';

/**
 * Example 1: Simple Trade List with Filters
 * Basic usage showing filter + component
 */
export function SimpleTradeListing() {
  const { applyFilters, getAllPresets } = useAdvancedFilters();

  // Get all trades from Redux
  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  // Apply current filters
  const filtered = applyFilters(trades);

  return (
    <div className="trades-section">
      <h2>Trades</h2>

      {/* Filter UI */}
      <AdvancedSearchFilter presets={getAllPresets()} showPresets={true} />

      {/* Results */}
      <div className="trades-list">
        {filtered.length === 0 ? (
          <p>No trades found matching your criteria</p>
        ) : (
          filtered.map((trade) => (
            <div key={trade.id} className="trade-item">
              <h3>{trade.id}</h3>
              <p>Status: {trade.status}</p>
              <p>Amount: {trade.amount}</p>
            </div>
          ))
        )}
      </div>

      {/* Results Summary */}
      <div className="results-summary">
        Found {filtered.length} of {trades.length} trades
      </div>
    </div>
  );
}

/**
 * Example 2: Dashboard with Maintained Filters
 * Keeps filters in Redux state across navigation
 */
export function TradesDashboard() {
  const { applyFilters, getAllPresets, filters, sortConfig } = useAdvancedFilters();

  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  const filtered = useMemo(() => applyFilters(trades), [trades, applyFilters]);

  return (
    <div className="dashboard">
      <div className="sidebar">
        <AdvancedSearchFilter presets={getAllPresets()} />
      </div>

      <div className="content">
        <h1>Trades Dashboard</h1>

        {/* Active Filters Display */}
        {Object.keys(filters).length > 0 && (
          <div className="active-filters">
            <strong>Active Filters:</strong>
            {Object.entries(filters).map(([key, value]) => (
              <span key={key}>
                {key}: {String(value)}
              </span>
            ))}
            {sortConfig.length > 0 && (
              <>
                <strong>Sort:</strong>
                {sortConfig.map((s, i) => (
                  <span key={i}>
                    {s.key} ({s.direction})
                  </span>
                ))}
              </>
            )}
          </div>
        )}

        {/* Trade Cards Grid */}
        <div className="trades-grid">
          {filtered.map((trade) => (
            <TradeCard key={trade.id} trade={trade} />
          ))}
        </div>
      </div>
    </div>
  );
}

/**
 * Example 3: Advanced Preset Management
 * Shows how to create, apply, and manage presets
 */
export function PresetManager() {
  const {
    filters,
    sortConfig,
    activePresetId,
    getAllPresets,
    applyPreset,
    saveAsPreset,
    deletePresetFn,
  } = useAdvancedFilters();

  const [presetName, setPresetName] = React.useState('');
  const [presetDesc, setPresetDesc] = React.useState('');
  const presets = getAllPresets();

  const handleSave = () => {
    if (presetName.trim()) {
      saveAsPreset(presetName, presetDesc);
      setPresetName('');
      setPresetDesc('');
    }
  };

  const handleDelete = (presetId: string) => {
    if (confirm(`Delete this preset?`)) {
      deletePresetFn(presetId);
    }
  };

  return (
    <div className="preset-manager">
      <h2>Filter Presets</h2>

      {/* Save Current as Preset */}
      <div className="save-preset section">
        <h3>Save Current Filters</h3>
        <input
          type="text"
          placeholder="Preset name"
          value={presetName}
          onChange={(e) => setPresetName(e.target.value)}
        />
        <textarea
          placeholder="Description (optional)"
          value={presetDesc}
          onChange={(e) => setPresetDesc(e.target.value)}
        />
        <button onClick={handleSave}>Save as Preset</button>
      </div>

      {/* Existing Presets */}
      <div className="presets-list section">
        <h3>Available Presets</h3>
        {presets.length === 0 ? (
          <p>No presets available</p>
        ) : (
          <div className="preset-items">
            {presets.map((preset) => (
              <div
                key={preset.id}
                className={`preset-item ${activePresetId === preset.id ? 'active' : ''}`}
              >
                <div className="preset-info">
                  <strong>{preset.name}</strong>
                  {preset.description && <p>{preset.description}</p>}
                </div>
                <div className="preset-actions">
                  <button
                    className="apply-btn"
                    onClick={() => applyPreset(preset.id)}
                  >
                    Apply
                  </button>
                  {!preset.isDefault && (
                    <button
                      className="delete-btn"
                      onClick={() => handleDelete(preset.id)}
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Example 4: Advanced Data Table with Filtering
 * Use filtering with pagination and export
 */
export function TradesDataTable() {
  const { applyFilters, filters, sortConfig } = useAdvancedFilters();
  const [page, setPage] = React.useState(1);
  const pageSize = 20;

  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  const filtered = useMemo(() => applyFilters(trades), [trades, applyFilters]);

  // Pagination
  const totalPages = Math.ceil(filtered.length / pageSize);
  const start = (page - 1) * pageSize;
  const paged = filtered.slice(start, start + pageSize);

  const handleExport = () => {
    const csv = [
      ['ID', 'Seller', 'Buyer', 'Amount', 'Status', 'Timestamp'],
      ...filtered.map((t) => [t.id, t.seller, t.buyer, t.amount, t.status, t.timestamp]),
    ]
      .map((row) => row.join(','))
      .join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `trades-export-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
  };

  return (
    <div className="data-table-section">
      <div className="table-header">
        <h2>All Trades</h2>
        <div className="table-actions">
          <button onClick={handleExport}>Export CSV</button>
          <span className="record-count">
            {filtered.length} results
            {filters && Object.keys(filters).length > 0 && ` (filtered)`}
          </span>
        </div>
      </div>

      {/* Filters Summary */}
      {Object.keys(filters).length > 0 && (
        <div className="filters-applied">
          <strong>Filters active:</strong>
          <code>{JSON.stringify(filters, null, 2)}</code>
        </div>
      )}

      {/* Table */}
      <table className="trades-table">
        <thead>
          <tr>
            <th>Trade ID</th>
            <th>Seller</th>
            <th>Buyer</th>
            <th>Amount</th>
            <th>Status</th>
            <th>Date</th>
          </tr>
        </thead>
        <tbody>
          {paged.map((trade) => (
            <tr key={trade.id}>
              <td>{trade.id}</td>
              <td>{trade.seller}</td>
              <td>{trade.buyer}</td>
              <td>{trade.amount}</td>
              <td>{trade.status}</td>
              <td>{new Date(trade.timestamp).toLocaleDateString()}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* Pagination */}
      <div className="pagination">
        <button disabled={page === 1} onClick={() => setPage(1)}>
          First
        </button>
        <button disabled={page === 1} onClick={() => setPage(page - 1)}>
          Prev
        </button>
        <span>
          Page {page} of {totalPages}
        </span>
        <button disabled={page === totalPages} onClick={() => setPage(page + 1)}>
          Next
        </button>
        <button disabled={page === totalPages} onClick={() => setPage(totalPages)}>
          Last
        </button>
      </div>
    </div>
  );
}

/**
 * Example 5: Programmatic Filtering
 * Direct use of applyAdvancedFilters function
 */
export function ProgrammaticFiltering() {
  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  // Complex programmatic filter
  const complexFiltered = React.useMemo(() => {
    return applyAdvancedFilters(
      trades,
      {
        status: 'completed',
        minAmount: 1000,
        maxAmount: 50000,
        isVerified: true,
        isDisputed: false,
        startDate: '2024-01-01',
        endDate: new Date().toISOString().split('T')[0],
      },
      [
        { key: 'amount', direction: 'desc' },
        { key: 'timestamp', direction: 'desc' },
      ]
    );
  }, [trades]);

  return (
    <div>
      <h2>High-Value Verified Trades (Programmatic)</h2>
      <p>
        Found {complexFiltered.length} trades matching: completed, verified, not disputed,
        $1,000-$50,000, from Jan 2024 to today, sorted by amount desc
      </p>
      {complexFiltered.map((trade) => (
        <div key={trade.id} className="trade-summary">
          {trade.id}: ${trade.amount} ({trade.status})
        </div>
      ))}
    </div>
  );
}

/**
 * Example 6: Reusable Filter Panel Component
 */
interface FilterPanelProps {
  onFiltersApplied?: (count: number) => void;
}

export function FilterPanel({ onFiltersApplied }: FilterPanelProps) {
  const { applyFilters, getAllPresets } = useAdvancedFilters();
  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  const handleApplyFilters = () => {
    const filtered = applyFilters(trades);
    onFiltersApplied?.(filtered.length);
  };

  return (
    <div className="filter-panel">
      <AdvancedSearchFilter
        presets={getAllPresets()}
        showPresets={true}
        onApplyFilters={handleApplyFilters}
      />
    </div>
  );
}

/**
 * Example 7: Minimal Setup - Just the Essentials
 */
export function MinimalExample() {
  const { applyFilters } = useAdvancedFilters();
  const trades = useSelector((state: RootState) =>
    state.trades.allIds.map((id) => state.trades.byId[id])
  );

  return (
    <>
      <AdvancedSearchFilter showPresets={true} />
      {applyFilters(trades).map((trade) => (
        <div key={trade.id}>{trade.id}</div>
      ))}
    </>
  );
}

// Helper component (these are assumed to exist)
function TradeCard({ trade }: { trade: any }) {
  return (
    <div className="trade-card">
      <h4>{trade.id}</h4>
      <p>{trade.seller} → {trade.buyer}</p>
      <p>Amount: {trade.amount}</p>
      <p>Status: {trade.status}</p>
    </div>
  );
}

export default {
  SimpleTradeListing,
  TradesDashboard,
  PresetManager,
  TradesDataTable,
  ProgrammaticFiltering,
  FilterPanel,
  MinimalExample,
};
