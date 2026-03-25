import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Trade, TradesState } from '../types';

const initialState: TradesState = {
  byId: {},
  allIds: [],
  loading: false,
  error: null,
};

const tradesSlice = createSlice({
  name: 'trades',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    addTrade: (state, action: PayloadAction<Trade>) => {
      const trade = action.payload;
      state.byId[trade.id] = trade;
      if (!state.allIds.includes(trade.id)) {
        state.allIds.push(trade.id);
      }
    },
    updateTrade: (state, action: PayloadAction<Partial<Trade> & { id: string }>) => {
      const { id, ...updates } = action.payload;
      if (state.byId[id]) {
        state.byId[id] = { ...state.byId[id], ...updates };
      }
    },
    setTrades: (state, action: PayloadAction<Trade[]>) => {
      state.byId = {};
      state.allIds = [];
      action.payload.forEach((trade) => {
        state.byId[trade.id] = trade;
        state.allIds.push(trade.id);
      });
    },
    removeTrade: (state, action: PayloadAction<string>) => {
      delete state.byId[action.payload];
      state.allIds = state.allIds.filter((id) => id !== action.payload);
    },
  },
});

export const { setLoading, setError, addTrade, updateTrade, setTrades, removeTrade } = tradesSlice.actions;
export default tradesSlice.reducer;
