import { configureStore, ThunkAction, Action } from '@reduxjs/toolkit';
import {
  persistStore,
  persistReducer,
  FLUSH,
  REHYDRATE,
  PAUSE,
  PERSIST,
  PURGE,
  REGISTER,
} from 'redux-persist';
import storage from 'redux-persist/lib/storage';
import logger from 'redux-logger';

import tradesReducer from './slices/tradesSlice';
import eventsReducer from './slices/eventsSlice';
import uiReducer from './slices/uiSlice';
import localeReducer from './slices/localeSlice';
import filterPresetsReducer from './slices/filterPresetsSlice';
import { escrowApi } from './api/escrowApi';

// =========================
// ROOT REDUCER
// =========================

const rootReducer = {
  trades: tradesReducer,
  events: eventsReducer,
  ui: uiReducer,
  locale: localeReducer,
  [escrowApi.reducerPath]: escrowApi.reducer,
};

// =========================
// PERSIST CONFIG
// =========================

const persistConfig = {
  key: 'root',
  storage,
  whitelist: ['trades', 'ui', 'locale'],
  blacklist: ['events', escrowApi.reducerPath],
};

const persistedReducer = persistReducer(persistConfig, (state: any, action: any) => {
  if (action.type === 'RESET_STATE') {
    return undefined;
  }

  return {
    trades: tradesReducer(state?.trades, action),
    events: eventsReducer(state?.events, action),
    ui: uiReducer(state?.ui, action),
    locale: localeReducer(state?.locale, action),
    [escrowApi.reducerPath]: escrowApi.reducer(undefined, action),
  };
});

// =========================
// STORE
// =========================

export const store = configureStore({
  reducer: persistedReducer,

  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [
          FLUSH,
          REHYDRATE,
          PAUSE,
          PERSIST,
          PURGE,
          REGISTER,
        ],
      },
    })
      .concat(escrowApi.middleware)
      .concat(logger as any), // ✅ fix logger type issue

  devTools: process.env.NODE_ENV !== 'production',
});

// =========================
// PERSISTOR
// =========================

export const persistor = persistStore(store);

// =========================
// TYPES
// =========================

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

export type AppThunk<ReturnType = void> = ThunkAction<
  ReturnType,
  RootState,
  unknown,
  Action<string>
>;