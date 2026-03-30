import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { FilterPresetsState } from '../types';

const initialState: FilterPresetsState = {
  presets: {},
  activePresetId: null,
  loading: false,
  error: null,
  persistenceEnabled: true,
};

const filterPresetsSlice = createSlice({
  name: 'filterPresets',
  initialState,
  reducers: {
    // Set all presets
    setPresets: (state, action: PayloadAction<Record<string, any>>) => {
      state.presets = action.payload;
    },

    // Add a new preset
    addPreset: (state, action: PayloadAction<any>) => {
      state.presets[action.payload.id] = action.payload;
    },

    // Update a preset
    updatePreset: (state, action: PayloadAction<any>) => {
      const { id, ...updates } = action.payload;
      if (state.presets[id]) {
        state.presets[id] = {
          ...state.presets[id],
          ...updates,
          updatedAt: new Date().toISOString(),
        };
      }
    },

    // Delete a preset
    deletePreset: (state, action: PayloadAction<string>) => {
      delete state.presets[action.payload];
      if (state.activePresetId === action.payload) {
        state.activePresetId = null;
      }
    },

    // Set active preset
    setActivePreset: (state, action: PayloadAction<string | null>) => {
      state.activePresetId = action.payload;
    },

    // Enable/disable persistence
    setPeristenceEnabled: (state, action: PayloadAction<boolean>) => {
      state.persistenceEnabled = action.payload;
    },

    // Reset to defaults
    resetToDefaults: (state) => {
      state.presets = {};
      state.activePresetId = null;
    },

    // Set loading state
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },

    // Set error
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
  },
});

export const {
  setPresets,
  addPreset,
  updatePreset,
  deletePreset,
  setActivePreset,
  setPeristenceEnabled,
  resetToDefaults,
  setLoading,
  setError,
} = filterPresetsSlice.actions;

export default filterPresetsSlice.reducer;
