import WebSocket from 'ws';
import { Server as HTTPServer } from 'http';
import { StellarEventIndexer } from './event-indexer';
import * as pg from 'pg';
import { v4 as uuidv4 } from 'uuid';

/**
 * Stellar Events WebSocket Server
 * Real-time event streaming to frontend clients
 */

interface ClientConnection {
  id: string;
  ws: WebSocket;
  subscriptions: Set<string>;
  connectedAt: Date;
}

interface WebSocketMessage {
  type: 'subscribe' | 'unsubscribe' | 'ping' | 'query';
  contractIds?: string[];
  eventTypeFilter?: string;
  queryOptions?: {
    limit?: number;
    offset?: number;
    ledgerSeqFrom?: number;
    ledgerSeqTo?: number;
  };
}

export class StellarEventsWebSocketServer {
  private wss: WebSocket.Server;
  private clients: Map<string, ClientConnection> = new Map();
  private indexer: StellarEventIndexer;
  private db: pg.Pool;
  private heartbeatInterval: NodeJS.Timer | null = null;
  private maxClients: number = 1000;

  constructor(httpServer: HTTPServer, indexer: StellarEventIndexer, db: pg.Pool) {
    this.indexer = indexer;
    this.db = db;

    // Create WebSocket server
    this.wss = new WebSocket.Server({
      server: httpServer,
      path: '/ws/events',
      perMessageDeflate: false, // Disable compression for performance
      maxPayload: 1024 * 100, // 100KB max message
    });

    this.setupEventHandlers();
    this.startHeartbeat();

    console.log('✓ Stellar Events WebSocket Server initialized');
  }

  /**
   * Setup WebSocket event handlers
   */
  private setupEventHandlers(): void {
    this.wss.on('connection', (ws: WebSocket, req) => {
      this.handleClientConnect(ws, req);
    });

    // Listen to indexer events
    this.indexer.on('event', (event) => {
      this.broadcastEvent(event);
    });

    this.indexer.on('broadcast', ({ clientId, event }) => {
      const client = this.clients.get(clientId);
      if (client) {
        this.sendToClient(client, {
          type: 'event',
          data: event,
          timestamp: new Date().toISOString(),
        });
      }
    });
  }

  /**
   * Handle new client connection
   */
  private handleClientConnect(ws: WebSocket, req: any): void {
    // Check max clients limit
    if (this.clients.size >= this.maxClients) {
      ws.close(4001, 'Server full');
      return;
    }

    const clientId = uuidv4();
    const client: ClientConnection = {
      id: clientId,
      ws,
      subscriptions: new Set(),
      connectedAt: new Date(),
    };

    this.clients.set(clientId, client);
    console.log(`📱 Client connected: ${clientId} (total: ${this.clients.size})`);

    // Send welcome message
    this.sendToClient(client, {
      type: 'welcome',
      clientId,
      message: 'Connected to Stellar Events stream',
      timestamp: new Date().toISOString(),
    });

    // Setup client message handlers
    ws.on('message', (data) => this.handleClientMessage(client, data));
    ws.on('close', () => this.handleClientDisconnect(client));
    ws.on('error', (error) => {
      console.error(`Client error (${clientId}):`, error.message);
    });

    // Send initial indexer status
    this.sendToClient(client, {
      type: 'indexer_status',
      status: this.indexer.getStatus(),
    });
  }

  /**
   * Handle client disconnection
   */
  private handleClientDisconnect(client: ClientConnection): void {
    this.clients.delete(client.id);
    this.indexer.removeSubscriber(client.id);
    console.log(`📱 Client disconnected: ${client.id} (total: ${this.clients.size})`);
  }

  /**
   * Handle incoming messages from clients
   */
  private async handleClientMessage(client: ClientConnection, data: WebSocket.Data): Promise<void> {
    try {
      const message = JSON.parse(data.toString()) as WebSocketMessage;

      switch (message.type) {
        case 'subscribe':
          this.handleSubscribe(client, message.contractIds || []);
          break;

        case 'unsubscribe':
          this.handleUnsubscribe(client, message.contractIds || []);
          break;

        case 'query':
          await this.handleQuery(client, message.queryOptions);
          break;

        case 'ping':
          this.sendToClient(client, { type: 'pong', timestamp: new Date().toISOString() });
          break;

        default:
          this.sendError(client, `Unknown message type: ${message.type}`);
      }
    } catch (error) {
      console.error('Error handling client message:', error);
      this.sendError(client, 'Invalid message format');
    }
  }

  /**
   * Subscribe client to contract events
   */
  private handleSubscribe(client: ClientConnection, contractIds: string[]): void {
    if (contractIds.length === 0) {
      return this.sendError(client, 'No contract IDs provided');
    }

    // Add to indexer subscriptions
    this.indexer.addSubscriber(client.id, contractIds);

    // Track locally
    contractIds.forEach((id) => client.subscriptions.add(id));

    this.sendToClient(client, {
      type: 'subscribed',
      contractIds,
      message: `Subscribed to ${contractIds.length} contract(s)`,
    });

    console.log(`✓ Client ${client.id} subscribed to: ${contractIds.join(', ')}`);
  }

  /**
   * Unsubscribe client from contract events
   */
  private handleUnsubscribe(client: ClientConnection, contractIds: string[]): void {
    contractIds.forEach((id) => client.subscriptions.delete(id));

    this.sendToClient(client, {
      type: 'unsubscribed',
      contractIds,
      message: `Unsubscribed from ${contractIds.length} contract(s)`,
    });

    // If no more subscriptions, remove from indexer
    if (client.subscriptions.size === 0) {
      this.indexer.removeSubscriber(client.id);
    }
  }

  /**
   * Handle historical event query
   */
  private async handleQuery(client: ClientConnection, options?: any): Promise<void> {
    try {
      const query = {
        contractId: options?.contractId,
        type: options?.type,
        ledgerSeqFrom: options?.ledgerSeqFrom,
        ledgerSeqTo: options?.ledgerSeqTo,
        limit: options?.limit || 100,
        offset: options?.offset || 0,
      };

      const events = await this.indexer.queryEvents(query);

      this.sendToClient(client, {
        type: 'query_response',
        events,
        count: events.length,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      this.sendError(client, `Query failed: ${(error as Error).message}`);
    }
  }

  /**
   * Broadcast event to all subscribers
   */
  private broadcastEvent(event: any): void {
    const payload = {
      type: 'event',
      data: event,
      timestamp: new Date().toISOString(),
    };

    let sentCount = 0;

    for (const [clientId, client] of this.clients) {
      // Only send if client subscribed to this contract
      if (client.subscriptions.has(event.contractId)) {
        this.sendToClient(client, payload);
        sentCount++;
      }
    }

    if (sentCount > 0) {
      console.log(`📡 Event broadcast to ${sentCount} clients`);
    }
  }

  /**
   * Send message to client
   */
  private sendToClient(client: ClientConnection, message: any): void {
    try {
      if (client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(JSON.stringify(message));
      }
    } catch (error) {
      console.error(`Failed to send to client ${client.id}:`, error);
    }
  }

  /**
   * Send error to client
   */
  private sendError(client: ClientConnection, errorMessage: string): void {
    this.sendToClient(client, {
      type: 'error',
      message: errorMessage,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Heartbeat to detect and cleanup dead connections
   */
  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      for (const [clientId, client] of this.clients) {
        if (client.ws.readyState === WebSocket.OPEN) {
          // Send heartbeat
          this.sendToClient(client, {
            type: 'heartbeat',
            timestamp: new Date().toISOString(),
          });
        } else {
          // Remove dead connection
          this.clients.delete(clientId);
          console.log(`💀 Removed dead connection: ${clientId}`);
        }
      }
    }, 30000); // Every 30 seconds
  }

  /**
   * Get server statistics
   */
  getStats(): {
    connectedClients: number;
    maxClients: number;
    subscriptions: Record<string, number>;
  } {
    const subscriptions: Record<string, number> = {};

    for (const client of this.clients.values()) {
      for (const contractId of client.subscriptions) {
        subscriptions[contractId] = (subscriptions[contractId] || 0) + 1;
      }
    }

    return {
      connectedClients: this.clients.size,
      maxClients: this.maxClients,
      subscriptions,
    };
  }

  /**
   * Close server
   */
  close(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }

    for (const client of this.clients.values()) {
      client.ws.close(1000, 'Server closing');
    }

    this.wss.close();
    console.log('✓ WebSocket server closed');
  }
}

export default StellarEventsWebSocketServer;
