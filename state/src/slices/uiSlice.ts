import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { UIState } from '../types';

const initialState: UIState = {
  selectedTradeId: null,
  filters: {},
  pagination: {
    page: 1,
    pageSize: 50,
  },
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
    setPagination: (state, action: PayloadAction<UIState['pagination']>) => {
      state.pagination = action.payload;
    },
    resetUI: (state) => {
      state.selectedTradeId = null;
      state.filters = {};
      state.pagination = { page: 1, pageSize: 50 };
    },
  },
});

export const { selectTrade, setFilters, setPagination, resetUI } = uiSlice.actions;
export default uiSlice.reducer;
