import { configureStore, combineReducers, ThunkAction, Action, Middleware } from '@reduxjs/toolkit';
import { persistStore, persistReducer, FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER } from 'redux-persist';
import storage from 'redux-persist/lib/storage';
import logger from 'redux-logger';
import tradesReducer from './slices/tradesSlice';
import eventsReducer from './slices/eventsSlice';
import uiReducer from './slices/uiSlice';
import localeReducer from './slices/localeSlice';
import { escrowApi } from './api/escrowApi';

const persistConfig = {
  key: 'root',
  storage,
  whitelist: ['trades', 'ui', 'locale'],
  blacklist: ['events', escrowApi.reducerPath],
};

const rootReducer = combineReducers({
  trades: tradesReducer,
  events: eventsReducer,
  ui: uiReducer,
  locale: localeReducer,
  [escrowApi.reducerPath]: escrowApi.reducer,
});

const appReducer = (state: ReturnType<typeof rootReducer> | undefined, action: Action<string>) => {
  if (action.type === 'RESET_STATE') {
    return rootReducer(undefined, action);
  }
  return rootReducer(state, action);
};

const persistedReducer = persistReducer(persistConfig, appReducer);

export const store = configureStore({
  reducer: persistedReducer,
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER],
      },
    }).concat(escrowApi.middleware).concat(logger as Middleware),
  devTools: process.env.NODE_ENV !== 'production',
});

export const persistor = persistStore(store);

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
export type AppThunk<ReturnType = void> = ThunkAction<ReturnType, RootState, unknown, Action<string>>;
