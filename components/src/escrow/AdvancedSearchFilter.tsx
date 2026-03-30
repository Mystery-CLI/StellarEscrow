import React, { useState, useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { setFilters, setSortConfig, setActivePreset, clearFilters } from '../../state/slices/uiSlice';
import { getFilterPresetsManager } from '@components';
import type { FilterCriteria, SortConfig, FilterPreset } from '@components';
import type { RootState } from '../../state/types';
import './AdvancedSearchFilter.css';

export interface AdvancedSearchFilterProps {
  onApplyFilters?: (filters: FilterCriteria, sortConfig: SortConfig[]) => void;
  presets?: FilterPreset[];
  showPresets?: boolean;
}

/**
 * AdvancedSearchFilter Component
 * Provides UI for applying multi-criteria filters and sorting to trades
 */
export const AdvancedSearchFilter: React.FC<AdvancedSearchFilterProps> = ({
  onApplyFilters,
  presets: externalPresets,
  showPresets = true,
}) => {
  const dispatch = useDispatch();
  const currentFilters = useSelector((state: RootState) => state.ui.filters);
  const currentSort = useSelector((state: RootState) => state.ui.sortConfig);
  const activePresetId = useSelector((state: RootState) => state.ui.activePresetId);

  const [filters, setLocalFilters] = useState<Partial<FilterCriteria>>(currentFilters);
  const [sortConfig, setLocalSortConfig] = useState<SortConfig[]>(currentSort || []);
  const [presets, setPresets] = useState<FilterPreset[]>(
    externalPresets || getFilterPresetsManager().getAllPresets()
  );
  const [isExpanded, setIsExpanded] = useState(false);

  // String filters
  const handleStatusChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      status: value || undefined,
    }));
  }, []);

  const handleSellerChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      seller: value || undefined,
    }));
  }, []);

  const handleBuyerChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      buyer: value || undefined,
    }));
  }, []);

  // Range filters
  const handleMinAmountChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      minAmount: value ? parseFloat(value) : undefined,
    }));
  }, []);

  const handleMaxAmountChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      maxAmount: value ? parseFloat(value) : undefined,
    }));
  }, []);

  const handleStartDateChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      startDate: value || undefined,
    }));
  }, []);

  const handleEndDateChange = useCallback((value: string) => {
    setLocalFilters((prev) => ({
      ...prev,
      endDate: value || undefined,
    }));
  }, []);

  // Boolean filters
  const handleIsVerifiedChange = useCallback((value: boolean | 'all') => {
    setLocalFilters((prev) => ({
      ...prev,
      isVerified: value === 'all' ? undefined : value,
    }));
  }, []);

  const handleIsDisputedChange = useCallback((value: boolean | 'all') => {
    setLocalFilters((prev) => ({
      ...prev,
      isDisputed: value === 'all' ? undefined : value,
    }));
  }, []);

  // Sorting
  const handleAddSort = useCallback(() => {
    setLocalSortConfig((prev) => [...prev, { key: 'timestamp', direction: 'desc' }]);
  }, []);

  const handleRemoveSort = useCallback((index: number) => {
    setLocalSortConfig((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleSortKeyChange = useCallback((index: number, key: string) => {
    setLocalSortConfig((prev) => {
      const updated = [...prev];
      updated[index].key = key;
      return updated;
    });
  }, []);

  const handleSortDirectionChange = useCallback((index: number, direction: 'asc' | 'desc') => {
    setLocalSortConfig((prev) => {
      const updated = [...prev];
      updated[index].direction = direction;
      return updated;
    });
  }, []);

  // Apply filters
  const handleApplyFilters = useCallback(() => {
    dispatch(setFilters(filters));
    dispatch(setSortConfig(sortConfig));
    dispatch(setActivePreset(undefined));

    if (onApplyFilters) {
      onApplyFilters(filters, sortConfig);
    }
  }, [filters, sortConfig, dispatch, onApplyFilters]);

  // Clear filters
  const handleClearFilters = useCallback(() => {
    setLocalFilters({});
    setLocalSortConfig([]);
    dispatch(clearFilters());
  }, [dispatch]);

  // Apply preset
  const handleApplyPreset = useCallback(
    (preset: FilterPreset) => {
      setLocalFilters(preset.filters);
      setLocalSortConfig(preset.sortConfig || []);
      dispatch(setActivePreset(preset.id));

      if (onApplyFilters) {
        onApplyFilters(preset.filters, preset.sortConfig || []);
      }
    },
    [dispatch, onApplyFilters]
  );

  // Save current filters as preset
  const handleSaveAsPreset = useCallback(() => {
    const presetName = prompt('Preset name:');
    if (presetName) {
      const manager = getFilterPresetsManager();
      const preset = manager.createPreset(presetName, filters, sortConfig);
      setPresets(manager.getAllPresets());
    }
  }, [filters, sortConfig]);

  return (
    <div className="advanced-search-filter">
      <div className="asf-header">
        <h3>Advanced Search & Filter</h3>
        <button
          className="asf-toggle-btn"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-expanded={isExpanded}
        >
          {isExpanded ? '▼ Collapse' : '▶ Expand'}
        </button>
      </div>

      {isExpanded && (
        <div className="asf-content">
          {/* Presets Section */}
          {showPresets && presets.length > 0 && (
            <div className="asf-section asf-presets">
              <h4>Saved Presets</h4>
              <div className="asf-presets-list">
                {presets.map((preset) => (
                  <button
                    key={preset.id}
                    className={`asf-preset-btn ${activePresetId === preset.id ? 'active' : ''}`}
                    onClick={() => handleApplyPreset(preset)}
                    title={preset.description}
                  >
                    {preset.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* String Filters Section */}
          <div className="asf-section asf-string-filters">
            <h4>Status & Participants</h4>

            <div className="asf-field">
              <label htmlFor="asf-status">Status</label>
              <select
                id="asf-status"
                value={filters.status || ''}
                onChange={(e) => handleStatusChange(e.target.value)}
              >
                <option value="">All Statuses</option>
                <option value="created">Created</option>
                <option value="funded">Funded</option>
                <option value="completed">Completed</option>
                <option value="disputed">Disputed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>

            <div className="asf-field">
              <label htmlFor="asf-seller">Seller</label>
              <input
                id="asf-seller"
                type="text"
                placeholder="Seller name..."
                value={filters.seller || ''}
                onChange={(e) => handleSellerChange(e.target.value)}
              />
            </div>

            <div className="asf-field">
              <label htmlFor="asf-buyer">Buyer</label>
              <input
                id="asf-buyer"
                type="text"
                placeholder="Buyer name..."
                value={filters.buyer || ''}
                onChange={(e) => handleBuyerChange(e.target.value)}
              />
            </div>
          </div>

          {/* Range Filters Section */}
          <div className="asf-section asf-range-filters">
            <h4>Amount & Date Range</h4>

            <div className="asf-field-row">
              <div className="asf-field">
                <label htmlFor="asf-min-amount">Min Amount</label>
                <input
                  id="asf-min-amount"
                  type="number"
                  min="0"
                  step="100"
                  placeholder="0"
                  value={filters.minAmount || ''}
                  onChange={(e) => handleMinAmountChange(e.target.value)}
                />
              </div>

              <div className="asf-field">
                <label htmlFor="asf-max-amount">Max Amount</label>
                <input
                  id="asf-max-amount"
                  type="number"
                  min="0"
                  step="100"
                  placeholder="No limit"
                  value={filters.maxAmount || ''}
                  onChange={(e) => handleMaxAmountChange(e.target.value)}
                />
              </div>
            </div>

            <div className="asf-field-row">
              <div className="asf-field">
                <label htmlFor="asf-start-date">Start Date</label>
                <input
                  id="asf-start-date"
                  type="date"
                  value={typeof filters.startDate === 'string' ? filters.startDate : ''}
                  onChange={(e) => handleStartDateChange(e.target.value)}
                />
              </div>

              <div className="asf-field">
                <label htmlFor="asf-end-date">End Date</label>
                <input
                  id="asf-end-date"
                  type="date"
                  value={typeof filters.endDate === 'string' ? filters.endDate : ''}
                  onChange={(e) => handleEndDateChange(e.target.value)}
                />
              </div>
            </div>
          </div>

          {/* Boolean Filters Section */}
          <div className="asf-section asf-boolean-filters">
            <h4>Verification & Status</h4>

            <div className="asf-field">
              <label htmlFor="asf-verified">Verified</label>
              <select
                id="asf-verified"
                value={filters.isVerified === undefined ? 'all' : filters.isVerified ? 'true' : 'false'}
                onChange={(e) =>
                  handleIsVerifiedChange(e.target.value === 'all' ? 'all' : e.target.value === 'true')
                }
              >
                <option value="all">All</option>
                <option value="true">Verified Only</option>
                <option value="false">Unverified Only</option>
              </select>
            </div>

            <div className="asf-field">
              <label htmlFor="asf-disputed">Disputed</label>
              <select
                id="asf-disputed"
                value={filters.isDisputed === undefined ? 'all' : filters.isDisputed ? 'true' : 'false'}
                onChange={(e) =>
                  handleIsDisputedChange(e.target.value === 'all' ? 'all' : e.target.value === 'true')
                }
              >
                <option value="all">All</option>
                <option value="true">Disputed Only</option>
                <option value="false">Non-Disputed Only</option>
              </select>
            </div>
          </div>

          {/* Sorting Section */}
          <div className="asf-section asf-sorting">
            <h4>Sort Order</h4>
            {sortConfig.length === 0 ? (
              <p className="asf-no-sorts">No sort criteria applied</p>
            ) : (
              <div className="asf-sorts-list">
                {sortConfig.map((sort, index) => (
                  <div key={index} className="asf-sort-item">
                    <select
                      value={sort.key}
                      onChange={(e) => handleSortKeyChange(index, e.target.value)}
                    >
                      <option value="timestamp">Date Created</option>
                      <option value="amount">Amount</option>
                      <option value="status">Status</option>
                      <option value="seller">Seller</option>
                      <option value="buyer">Buyer</option>
                      <option value="isVerified">Verified</option>
                    </select>

                    <select
                      value={sort.direction}
                      onChange={(e) =>
                        handleSortDirectionChange(index, e.target.value as 'asc' | 'desc')
                      }
                    >
                      <option value="asc">Ascending (A-Z, 0-9)</option>
                      <option value="desc">Descending (Z-A, 9-0)</option>
                    </select>

                    <button
                      className="asf-remove-sort-btn"
                      onClick={() => handleRemoveSort(index)}
                      title="Remove this sort"
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
            )}
            <button className="asf-add-sort-btn" onClick={handleAddSort}>
              + Add Sort Criteria
            </button>
          </div>

          {/* Action Buttons */}
          <div className="asf-actions">
            <button className="asf-btn asf-btn-primary" onClick={handleApplyFilters}>
              Apply Filters
            </button>
            <button className="asf-btn asf-btn-secondary" onClick={handleClearFilters}>
              Clear All
            </button>
            <button className="asf-btn asf-btn-tertiary" onClick={handleSaveAsPreset}>
              Save as Preset
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default AdvancedSearchFilter;
