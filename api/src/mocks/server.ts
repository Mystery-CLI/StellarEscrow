import { setupServer } from 'msw/node';
import { handlers } from '../mocks';
import { recordCall } from './monitor';

export const server = setupServer(...handlers);

server.events.on('request:match', (req) => {
  recordCall(req.method, req.url.pathname);
});
