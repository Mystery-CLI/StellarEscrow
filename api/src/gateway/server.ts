#!/usr/bin/env node
/**
 * API Gateway entrypoint.
 * Start with: node dist/gateway/server.js
 * Or via package.json: "gateway": "ts-node src/gateway/server.ts"
 */

import { ApiGateway } from './gateway';
import { loadGatewayConfig } from './config';

const config = loadGatewayConfig();
const gateway = new ApiGateway(config);
const server = gateway.listen(config.port);

process.on('SIGTERM', () => {
  console.log('[gateway] SIGTERM received — shutting down');
  server.close(() => process.exit(0));
});

process.on('SIGINT', () => {
  server.close(() => process.exit(0));
});
