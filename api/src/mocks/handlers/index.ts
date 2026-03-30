import { tradeHandlers } from './trades';
import { eventHandlers } from './events';
import { blockchainHandlers } from './blockchain';

export const handlers = [
  ...tradeHandlers,
  ...eventHandlers,
  ...blockchainHandlers,
];