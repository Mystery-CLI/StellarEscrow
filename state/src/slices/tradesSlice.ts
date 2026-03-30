import { createSlice, createEntityAdapter, PayloadAction } from '@reduxjs/toolkit';
import { Trade, TradesState } from '../types';

export const tradesAdapter = createEntityAdapter<Trade>({
  selectId: (trade) => trade.id,
  sortComparer: (a, b) => a.timestamp.localeCompare(b.timestamp),
});

const initialState: TradesState = tradesAdapter.getInitialState({
  loading: false,
  error: null,
});

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
    addTrade: tradesAdapter.upsertOne,
    updateTrade: (state, action: PayloadAction<Partial<Trade> & { id: string }>) => {
      const { id, ...changes } = action.payload;
      tradesAdapter.updateOne(state, { id, changes });
    },
    setTrades: tradesAdapter.setAll,
    removeTrade: tradesAdapter.removeOne,
  },
});

export const { setLoading, setError, addTrade, updateTrade, setTrades, removeTrade } = tradesSlice.actions;
export default tradesSlice.reducer;
