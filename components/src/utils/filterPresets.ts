/**
 * Filter Presets Utility
 * Manages saved filter configurations for quick access
 */

import { FilterCriteria, SortConfig } from './filters';

/**
 * Represents a saved filter preset
 */
export interface FilterPreset {
  id: string;
  name: string;
  description?: string;
  filters: Partial<FilterCriteria>;
  sortConfig: SortConfig[];
  createdAt: string;
  updatedAt: string;
  isDefault?: boolean;
}

/**
 * Default filter presets for StellarEscrow trades
 */
const DEFAULT_PRESETS: FilterPreset[] = [
  {
    id: 'completed-trades',
    name: 'Completed Trades',
    description: 'All successfully completed trades',
    filters: { status: 'completed', isVerified: true },
    sortConfig: [{ key: 'timestamp', direction: 'desc' }],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
  {
    id: 'pending-trades',
    name: 'Pending Trades',
    description: 'Trades waiting for completion',
    filters: { status: 'funded' },
    sortConfig: [{ key: 'timestamp', direction: 'asc' }],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
  {
    id: 'disputed-trades',
    name: 'Disputed Trades',
    description: 'Trades in dispute requiring arbitration',
    filters: { status: 'disputed' },
    sortConfig: [{ key: 'timestamp', direction: 'desc' }],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
  {
    id: 'high-value-trades',
    name: 'High Value Trades',
    description: 'Trades with amounts above 5000',
    filters: { minAmount: 5000 },
    sortConfig: [{ key: 'amount', direction: 'desc' }],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
  {
    id: 'verified-safe-trades',
    name: 'Verified & Safe Trades',
    description: 'Verified trades that are not disputed',
    filters: { isVerified: true, isDisputed: false },
    sortConfig: [
      { key: 'isVerified', direction: 'desc' },
      { key: 'timestamp', direction: 'desc' },
    ],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
  {
    id: 'recent-trades',
    name: 'Recent Trades',
    description: 'Most recently created trades',
    filters: {},
    sortConfig: [{ key: 'timestamp', direction: 'desc' }],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    isDefault: true,
  },
];

const PRESETS_STORAGE_KEY = 'stellar-escrow-filter-presets';

/**
 * Filter Presets Manager
 * Handles CRUD operations for filter presets with localStorage persistence
 */
export class FilterPresetsManager {
  private presets: Map<string, FilterPreset>;
  private storageEnabled: boolean;

  constructor() {
    this.presets = new Map();
    this.storageEnabled = this.isLocalStorageAvailable();

    // Initialize with default presets
    DEFAULT_PRESETS.forEach((preset) => {
      this.presets.set(preset.id, { ...preset });
    });

    // Load custom presets from storage
    this.loadFromStorage();
  }

  /**
   * Check if localStorage is available
   */
  private isLocalStorageAvailable(): boolean {
    try {
      const test = '__storage_test__';
      localStorage.setItem(test, test);
      localStorage.removeItem(test);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Load presets from localStorage
   */
  private loadFromStorage(): void {
    if (!this.storageEnabled) return;

    try {
      const stored = localStorage.getItem(PRESETS_STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored) as FilterPreset[];
        parsed.forEach((preset) => {
          this.presets.set(preset.id, preset);
        });
      }
    } catch (error) {
      console.warn('Failed to load filter presets from storage:', error);
    }
  }

  /**
   * Save presets to localStorage
   */
  private saveToStorage(): void {
    if (!this.storageEnabled) return;

    try {
      const customPresets = Array.from(this.presets.values()).filter((p) => !p.isDefault);
      localStorage.setItem(PRESETS_STORAGE_KEY, JSON.stringify(customPresets));
    } catch (error) {
      console.warn('Failed to save filter presets to storage:', error);
    }
  }

  /**
   * Get all presets
   */
  getAllPresets(): FilterPreset[] {
    return Array.from(this.presets.values()).sort((a, b) => {
      // Default presets first, then by name
      if (a.isDefault !== b.isDefault) {
        return a.isDefault ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });
  }

  /**
   * Get preset by ID
   */
  getPreset(id: string): FilterPreset | undefined {
    return this.presets.get(id);
  }

  /**
   * Create a new preset
   */
  createPreset(
    name: string,
    filters: Partial<FilterCriteria>,
    sortConfig: SortConfig[] = [],
    description?: string
  ): FilterPreset {
    const id = this.generateId();
    const now = new Date().toISOString();

    const preset: FilterPreset = {
      id,
      name,
      description,
      filters,
      sortConfig,
      createdAt: now,
      updatedAt: now,
      isDefault: false,
    };

    this.presets.set(id, preset);
    this.saveToStorage();

    return preset;
  }

  /**
   * Update an existing preset
   */
  updatePreset(
    id: string,
    updates: Partial<Omit<FilterPreset, 'id' | 'createdAt'>>
  ): FilterPreset | undefined {
    const preset = this.presets.get(id);
    if (!preset) return undefined;

    const updated: FilterPreset = {
      ...preset,
      ...updates,
      id: preset.id, // Ensure ID doesn't change
      createdAt: preset.createdAt, // Preserve creation date
      updatedAt: new Date().toISOString(),
    };

    this.presets.set(id, updated);
    this.saveToStorage();

    return updated;
  }

  /**
   * Delete a preset
   */
  deletePreset(id: string): boolean {
    const preset = this.presets.get(id);
    if (!preset) return false;

    // Don't allow deleting default presets
    if (preset.isDefault) {
      console.warn(`Cannot delete default preset: ${id}`);
      return false;
    }

    this.presets.delete(id);
    this.saveToStorage();

    return true;
  }

  /**
   * Reset to default presets only
   */
  resetToDefaults(): void {
    this.presets.clear();
    DEFAULT_PRESETS.forEach((preset) => {
      this.presets.set(preset.id, { ...preset });
    });
    this.saveToStorage();
  }

  /**
   * Export presets as JSON
   */
  exportPresets(): string {
    const customPresets = Array.from(this.presets.values()).filter((p) => !p.isDefault);
    return JSON.stringify(customPresets, null, 2);
  }

  /**
   * Import presets from JSON
   */
  importPresets(jsonData: string): number {
    try {
      const imported = JSON.parse(jsonData) as FilterPreset[];
      let count = 0;

      imported.forEach((preset) => {
        // Validate required fields
        if (preset.id && preset.name && preset.filters) {
          this.presets.set(preset.id, {
            ...preset,
            updatedAt: new Date().toISOString(),
          });
          count++;
        }
      });

      this.saveToStorage();
      return count;
    } catch (error) {
      console.error('Failed to import presets:', error);
      return 0;
    }
  }

  /**
   * Generate unique ID for new presets
   */
  private generateId(): string {
    return `preset-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Search presets by name
   */
  searchPresets(query: string): FilterPreset[] {
    const q = query.toLowerCase();
    return Array.from(this.presets.values()).filter(
      (p) => p.name.toLowerCase().includes(q) || p.description?.toLowerCase().includes(q)
    );
  }

  /**
   * Get presets by filter criteria match
   */
  getPresetsByFilters(filters: Partial<FilterCriteria>): FilterPreset[] {
    return Array.from(this.presets.values()).filter((preset) => {
      // Check if all specified filters match
      return Object.entries(filters).every(([key, value]) => preset.filters[key as keyof FilterCriteria] === value);
    });
  }
}

/**
 * Singleton instance
 */
let instance: FilterPresetsManager | null = null;

/**
 * Get or create the filter presets manager instance
 */
export function getFilterPresetsManager(): FilterPresetsManager {
  if (!instance) {
    instance = new FilterPresetsManager();
  }
  return instance;
}

/**
 * Reset the singleton (useful for testing)
 */
export function resetFilterPresetsManager(): void {
  instance = null;
}
