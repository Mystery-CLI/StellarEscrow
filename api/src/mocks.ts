import { tradeHandlers } from './mocks/handlers/trades';
import { eventHandlers } from './mocks/handlers/events';
import { blockchainHandlers } from './mocks/handlers/blockchain';

export const handlers = [
  ...tradeHandlers,
  ...eventHandlers,
  ...blockchainHandlers,
];
