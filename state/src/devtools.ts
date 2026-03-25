import { store } from './store';

export const enableReduxDevTools = () => {
  if (typeof window !== 'undefined' && (window as any).__REDUX_DEVTOOLS_EXTENSION__) {
    console.log('Redux DevTools enabled');
  }
};

export const getStateSnapshot = () => {
  return store.getState();
};

export const subscribeToStateChanges = (callback: (state: any) => void) => {
  return store.subscribe(() => {
    callback(store.getState());
  });
};

export const logStateTree = () => {
  const state = store.getState();
  console.group('Redux State Tree');
  console.log('Trades:', state.trades);
  console.log('Events:', state.events);
  console.log('UI:', state.ui);
  console.groupEnd();
};
