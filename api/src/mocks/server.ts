import { setupServer } from 'msw/node';
import { handlers } from './handlers';
import { recordCall } from './monitor';

console.log('[MSW] handler count:', handlers.length);

export const server = setupServer(...handlers);

server.events.on('request:match', (req) => {
  recordCall(req.method, req.url.pathname);
});
