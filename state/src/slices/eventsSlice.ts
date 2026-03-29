import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Event, EventsState } from '../types';

const initialState: EventsState = {
  byId: {},
  allIds: [],
  loading: false,
  error: null,
};

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
    addEvent: (state, action: PayloadAction<Event>) => {
      const event = action.payload;
      state.byId[event.id] = event;
      if (!state.allIds.includes(event.id)) {
        state.allIds.unshift(event.id);
      }
    },
    setEvents: (state, action: PayloadAction<Event[]>) => {
      state.byId = {};
      state.allIds = [];
      action.payload.forEach((event) => {
        state.byId[event.id] = event;
        state.allIds.push(event.id);
      });
    },
    clearEvents: (state) => {
      state.byId = {};
      state.allIds = [];
    },
  },
});

export const { setLoading, setError, addEvent, setEvents, clearEvents } = eventsSlice.actions;
export default eventsSlice.reducer;
