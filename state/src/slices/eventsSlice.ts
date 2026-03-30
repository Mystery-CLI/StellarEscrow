import { createSlice, createEntityAdapter, PayloadAction } from '@reduxjs/toolkit';
import { Event, EventsState } from '../types';

export const eventsAdapter = createEntityAdapter<Event>({
  selectId: (event) => event.id,
  sortComparer: (a, b) => b.timestamp.localeCompare(a.timestamp),
});

const initialState: EventsState = eventsAdapter.getInitialState({
  loading: false,
  error: null,
});

const eventsSlice = createSlice({
  name: 'events',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    addEvent: eventsAdapter.upsertOne,
    setEvents: eventsAdapter.setAll,
    clearEvents: (state) => {
      eventsAdapter.removeAll(state);
    },
  },
});

export const { setLoading, setError, addEvent, setEvents, clearEvents } = eventsSlice.actions;
export default eventsSlice.reducer;
