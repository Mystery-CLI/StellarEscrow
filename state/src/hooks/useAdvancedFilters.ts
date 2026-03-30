import { useCallback, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { setFilters, setSortConfig, setActivePreset, clearFilters } from '../slices/uiSlice';
import {
  setActivePreset as setActivePresetAction,
  addPreset,
  updatePreset,
  deletePreset,
  setPresets,
} from '../slices/filterPresetsSlice';
import { getFilterPresetsManager, applyAdvancedFilters } from '@components';
import type { FilterCriteria, SortConfig, FilterPreset } from '@components';
import type { RootState } from '../types';

/**
 * Hook for managing advanced filters and persistence
 */
export function useAdvancedFilters() {
  const dispatch = useDispatch();
  const filters = useSelector((state: RootState) => state.ui.filters);
  const sortConfig = useSelector((state: RootState) => state.ui.sortConfig || []);
  const activePresetId = useSelector((state: RootState) => state.ui.activePresetId);
  const persistenceEnabled = useSelector(
    (state: RootState) => state.filterPresets?.persistenceEnabled ?? true
  );

  const presetsManager = getFilterPresetsManager();

  // Initialize presets from manager on first load
  useEffect(() => {
    if (!persistenceEnabled) return;

    const allPresets = presetsManager.getAllPresets();
    const presetsRecord = allPresets.reduce(
      (acc, p) => {
        acc[p.id] = p;
        return acc;
      },
      {} as Record<string, FilterPreset>
    );

    dispatch(setPresets(presetsRecord));
  }, [dispatch, persistenceEnabled]);

  /**
   * Apply filters to data
   */
  const applyFilters = useCallback(
    <T extends Record<string, any>>(data: T[]): T[] => {
      return applyAdvancedFilters(data, filters, sortConfig);
    },
    [filters, sortConfig]
  );

  /**
   * Set filters
   */
  const setFiltersCriteria = useCallback(
    (newFilters: Partial<FilterCriteria>) => {
      dispatch(setFilters(newFilters));
      if (activePresetId) {
        dispatch(setActivePreset(undefined));
      }
    },
    [dispatch, activePresetId]
  );

  /**
   * Set sort configuration
   */
  const setSortConfigFn = useCallback(
    (newSortConfig: SortConfig[]) => {
      dispatch(setSortConfig(newSortConfig));
      if (activePresetId) {
        dispatch(setActivePreset(undefined));
      }
    },
    [dispatch, activePresetId]
  );

  /**
   * Clear all filters
   */
  const clearAllFilters = useCallback(() => {
    dispatch(clearFilters());
  }, [dispatch]);

  /**
   * Apply a saved preset
   */
  const applyPreset = useCallback(
    (presetId: string) => {
      const preset = presetsManager.getPreset(presetId);
      if (preset) {
        dispatch(setFilters(preset.filters));
        dispatch(setSortConfig(preset.sortConfig || []));
        dispatch(setActivePreset(presetId));
      }
    },
    [dispatch, presetsManager]
  );

  /**
   * Save current filters as a new preset
   */
  const saveAsPreset = useCallback(
    (name: string, description?: string): FilterPreset | null => {
      if (!persistenceEnabled) {
        console.warn('Filter persistence is disabled');
        return null;
      }

      const preset = presetsManager.createPreset(name, filters, sortConfig, description);
      dispatch(addPreset(preset));
      return preset;
    },
    [dispatch, filters, sortConfig, persistenceEnabled, presetsManager]
  );

  /**
   * Update an existing preset
   */
  const updatePresetFn = useCallback(
    (presetId: string, updates: Partial<Omit<FilterPreset, 'id' | 'createdAt'>>): boolean => {
      if (!persistenceEnabled) {
        console.warn('Filter persistence is disabled');
        return false;
      }

      const updated = presetsManager.updatePreset(presetId, updates);
      if (updated) {
        dispatch(updatePreset(updated));
        return true;
      }
      return false;
    },
    [dispatch, persistenceEnabled, presetsManager]
  );

  /**
   * Delete a preset
   */
  const deletePresetFn = useCallback(
    (presetId: string): boolean => {
      if (!persistenceEnabled) {
        console.warn('Filter persistence is disabled');
        return false;
      }

      const deleted = presetsManager.deletePreset(presetId);
      if (deleted) {
        dispatch(deletePreset(presetId));
        return true;
      }
      return false;
    },
    [dispatch, persistenceEnabled, presetsManager]
  );

  /**
   * Get all presets
   */
  const getAllPresets = useCallback((): FilterPreset[] => {
    return presetsManager.getAllPresets();
  }, [presetsManager]);

  /**
   * Get a specific preset
   */
  const getPreset = useCallback(
    (presetId: string): FilterPreset | undefined => {
      return presetsManager.getPreset(presetId);
    },
    [presetsManager]
  );

  /**
   * Export presets as JSON
   */
  const exportPresets = useCallback((): string => {
    return presetsManager.exportPresets();
  }, [presetsManager]);

  /**
   * Import presets from JSON
   */
  const importPresets = useCallback(
    (jsonData: string): number => {
      const count = presetsManager.importPresets(jsonData);
      const allPresets = presetsManager.getAllPresets();
      const presetsRecord = allPresets.reduce(
        (acc, p) => {
          acc[p.id] = p;
          return acc;
        },
        {} as Record<string, FilterPreset>
      );
      dispatch(setPresets(presetsRecord));
      return count;
    },
    [dispatch, presetsManager]
  );

  return {
    // State
    filters,
    sortConfig,
    activePresetId,
    persistenceEnabled,

    // Filter operations
    applyFilters,
    setFiltersCriteria,
    setSortConfigFn,
    clearAllFilters,

    // Preset operations
    applyPreset,
    saveAsPreset,
    updatePresetFn,
    deletePresetFn,
    getAllPresets,
    getPreset,
    exportPresets,
    importPresets,
  };
}

/**
 * Hook for filtering trade data
 * Commonly used pattern
 */
export function useFilteredTrades(trades: any[]) {
  const { applyFilters } = useAdvancedFilters();
  return applyFilters(trades);
}

export default useAdvancedFilters;
