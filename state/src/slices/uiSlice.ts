import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { UIState } from '../types';

const initialState: UIState = {
  selectedTradeId: null,
  filters: {},
  sortConfig: [],
  pagination: {
    page: 1,
    pageSize: 50,
  },
  activePresetId: undefined,
};

const uiSlice = createSlice({
  name: 'ui',
  initialState,
  reducers: {
    selectTrade: (state, action: PayloadAction<string | null>) => {
      state.selectedTradeId = action.payload;
    },
    setFilters: (state, action: PayloadAction<UIState['filters']>) => {
      state.filters = action.payload;
    },
    setSortConfig: (state, action: PayloadAction<UIState['sortConfig']>) => {
      state.sortConfig = action.payload;
    },
    setPagination: (state, action: PayloadAction<UIState['pagination']>) => {
      state.pagination = action.payload;
    },
    setActivePreset: (state, action: PayloadAction<string | undefined>) => {
      state.activePresetId = action.payload;
    },
    applyPreset: (
      state,
      action: PayloadAction<{ filters: UIState['filters']; sortConfig: UIState['sortConfig'] }>
    ) => {
      state.filters = action.payload.filters;
      state.sortConfig = action.payload.sortConfig;
    },
    clearFilters: (state) => {
      state.filters = {};
      state.sortConfig = [];
      state.activePresetId = undefined;
    },
    resetUI: (state) => {
      state.selectedTradeId = null;
      state.filters = {};
      state.sortConfig = [];
      state.pagination = { page: 1, pageSize: 50 };
      state.activePresetId = undefined;
    },
  },
});

export const {
  selectTrade,
  setFilters,
  setSortConfig,
  setPagination,
  setActivePreset,
  applyPreset,
  clearFilters,
  resetUI,
} = uiSlice.actions;
export default uiSlice.reducer;
