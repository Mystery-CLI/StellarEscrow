import { EventEmitter } from 'events';
import * as StellarSDK from '@stellar/js-sdk';
import WebSocket from 'ws';
import * as pg from 'pg';
import axios from 'axios';

/**
 * Stellar Event Indexer Service
 *
 * Features:
 * - Real-time event monitoring from Stellar Soroban
 * - XDR parsing and JSON transformation
 * - WebSocket streaming to clients
 * - Historical event replay
 * - Event querying API
 */

// ============================================================================
// TYPES & CONFIGURATION
// ============================================================================

interface StellarEvent {
  id: string;
  type: string; // 'created', 'funded', 'completed', 'disputed', 'resolved'
  contractId: string;
  ledgerSeq: number;
  ledgerClosedAt: Date;
  txHash: string;
  data: Record<string, any>;
  parsedAt: Date;
}

interface EventQuery {
  contractId?: string;
  type?: string;
  ledgerSeqFrom?: number;
  ledgerSeqTo?: number;
  limit?: number;
  offset?: number;
}

interface WebSocketMessage {
  type: 'subscribe' | 'unsubscribe' | 'query';
  contractId?: string;
  eventTypeFilter?: string;
}

// ============================================================================
// STELLAR CONFIGURATION
// ============================================================================

const STELLAR_CONFIG = {
  network:
    process.env.STELLAR_NETWORK || 'TESTNET_SOROBAN',
  rpcUrl:
    process.env.STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org',
  contractId: process.env.STELLAR_ESCROW_CONTRACT_ID || '',
  pollingInterval: parseInt(process.env.POLLING_INTERVAL || '4') * 1000, // 4 seconds
  maxRetries: 3,
};

// ============================================================================
// DATABASE SCHEMA
// ============================================================================

export const STELLAR_EVENTS_SCHEMA = `
-- Indexed Stellar events
CREATE TABLE IF NOT EXISTS stellar_events (
  id SERIAL PRIMARY KEY,
  event_id VARCHAR(255) UNIQUE NOT NULL,
  contract_id VARCHAR(255) NOT NULL,
  event_type VARCHAR(100) NOT NULL,
  ledger_seq BIGINT NOT NULL,
  ledger_closed_at TIMESTAMP NOT NULL,
  tx_hash VARCHAR(255),
  xdr_event TEXT NOT NULL,
  parsed_data JSONB NOT NULL,
  parsed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Event subscriptions for WebSocket clients
CREATE TABLE IF NOT EXISTS event_subscriptions (
  id SERIAL PRIMARY KEY,
  client_id VARCHAR(255) NOT NULL,
  contract_id VARCHAR(255),
  event_type_filter VARCHAR(100),
  subscribed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  last_heartbeat TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexer state tracking
CREATE TABLE IF NOT EXISTS indexer_state (
  id SERIAL PRIMARY KEY,
  contract_id VARCHAR(255) UNIQUE NOT NULL,
  last_ledger_seq BIGINT,
  last_event_id VARCHAR(255),
  last_synced TIMESTAMP,
  sync_status VARCHAR(50) DEFAULT 'syncing',
  error_count INT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Event replay log
CREATE TABLE IF NOT EXISTS event_replay_log (
  id SERIAL PRIMARY KEY,
  replay_id VARCHAR(255) UNIQUE,
  start_ledger BIGINT,
  end_ledger BIGINT,
  events_replayed BIGINT,
  status VARCHAR(50),
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_stellar_events_contract ON stellar_events(contract_id);
CREATE INDEX idx_stellar_events_type ON stellar_events(event_type);
CREATE INDEX idx_stellar_events_ledger ON stellar_events(ledger_seq DESC);
CREATE INDEX idx_stellar_events_timestamp ON stellar_events(parsed_at DESC);
CREATE INDEX idx_indexer_state_contract ON indexer_state(contract_id);
`;

// ============================================================================
// EVENT PARSING & XDR UTILITIES
// ============================================================================

class XDRParser {
  /**
   * Parse Stellar event XDR and convert to JSON
   */
  static parseEventXDR(xdrEvent: string, eventType: string): Record<string, any> {
    try {
      // Parse XDR into Stellar data structure
      const parsed = StellarSDK.xdr.TransactionEnvelope.fromXDR(
        xdrEvent,
        'base64'
      );

      // Extract relevant fields based on event type
      const eventData = {
        eventType,
        xdrHash: this.hashXDR(xdrEvent),
        timestamp: new Date().toISOString(),
      };

      // Event-specific parsing
      switch (eventType) {
        case 'trade_created':
          return {
            ...eventData,
            tradeId: this.extractField(parsed, 'trade_id'),
            sellerId: this.extractField(parsed, 'seller'),
            amount: this.extractField(parsed, 'amount'),
            asset: this.extractField(parsed, 'asset'),
          };

        case 'trade_funded':
          return {
            ...eventData,
            tradeId: this.extractField(parsed, 'trade_id'),
            buyerId: this.extractField(parsed, 'buyer'),
            fundedAmount: this.extractField(parsed, 'funded_amount'),
          };

        case 'trade_completed':
          return {
            ...eventData,
            tradeId: this.extractField(parsed, 'trade_id'),
            releasedTo: this.extractField(parsed, 'released_to'),
            releasedAmount: this.extractField(parsed, 'released_amount'),
          };

        case 'dispute_raised':
          return {
            ...eventData,
            tradeId: this.extractField(parsed, 'trade_id'),
            initiator: this.extractField(parsed, 'initiator'),
            reason: this.extractField(parsed, 'dispute_reason'),
          };

        case 'dispute_resolved':
          return {
            ...eventData,
            tradeId: this.extractField(parsed, 'trade_id'),
            winner: this.extractField(parsed, 'resolution_winner'),
            arbitratorDecision: this.extractField(parsed, 'decision_reason'),
          };

        default:
          return eventData;
      }
    } catch (error) {
      console.error(`Failed to parse XDR for ${eventType}:`, error);
      return {
        eventType,
        parseError: (error as Error).message,
        rawXdr: xdrEvent.substring(0, 100),
      };
    }
  }

  /**
   * Extract field from Stellar object
   */
  private static extractField(obj: any, fieldName: string): any {
    try {
      return obj[fieldName] || null;
    } catch {
      return null;
    }
  }

  /**
   * Hash XDR for deduplication
   */
  private static hashXDR(xdr: string): string {
    const crypto = require('crypto');
    return crypto.createHash('sha256').update(xdr).digest('hex');
  }
}

// ============================================================================
// STELLAR EVENT INDEXER SERVICE
// ============================================================================

export class StellarEventIndexer extends EventEmitter {
  private db: pg.Pool;
  private rpcClient: any;
  private isRunning: boolean = false;
  private lastLedgerSeq: number = 0;
  private contractId: string;
  private wsServer?: WebSocket.Server;
  private subscribedClients: Map<string, Set<string>> = new Map(); // clientId -> contractIds

  constructor(dbPool: pg.Pool, contractId: string) {
    super();
    this.db = dbPool;
    this.contractId = contractId || STELLAR_CONFIG.contractId;
    this.rpcClient = new StellarSDK.SorobanDataBuilder().server(STELLAR_CONFIG.rpcUrl);
    this.initializeDatabase();
  }

  /**
   * Initialize database tables
   */
  private async initializeDatabase(): Promise<void> {
    try {
      await this.db.query(STELLAR_EVENTS_SCHEMA);
      console.log('✓ Stellar events tables initialized');

      // Load last indexed ledger
      const result = await this.db.query(
        `SELECT last_ledger_seq FROM indexer_state WHERE contract_id = $1`,
        [this.contractId]
      );

      if (result.rows.length > 0) {
        this.lastLedgerSeq = result.rows[0].last_ledger_seq || 0;
        console.log(`✓ Last indexed ledger: ${this.lastLedgerSeq}`);
      }
    } catch (error) {
      console.error('Failed to initialize Stellar events tables:', error);
    }
  }

  /**
   * Start real-time event monitoring
   */
  async startIndexing(startLedger?: number): Promise<void> {
    if (this.isRunning) {
      console.log('Indexer already running');
      return;
    }

    this.isRunning = true;
    console.log('🚀 Starting Stellar Event Indexer...');

    if (startLedger) {
      this.lastLedgerSeq = startLedger;
    }

    // Start polling loop
    this.pollForEvents();

    // Emit ready event
    this.emit('ready');
  }

  /**
   * Stop indexing
   */
  stopIndexing(): void {
    this.isRunning = false;
    console.log('⏸️  Stellar Event Indexer stopped');
  }

  /**
   * Poll Stellar RPC for new events
   */
  private async pollForEvents(): Promise<void> {
    while (this.isRunning) {
      try {
        // Fetch new events from Stellar
        const events = await this.fetchStellarEvents(this.lastLedgerSeq + 1);

        if (events && events.length > 0) {
          console.log(`📦 Fetched ${events.length} new events`);

          for (const event of events) {
            await this.processEvent(event);
          }

          // Update last indexed ledger
          this.lastLedgerSeq = Math.max(
            ...events.map((e) => e.ledger_sequence)
          );

          await this.updateIndexerState(this.lastLedgerSeq);

          // Broadcast to WebSocket subscribers
          this.broadcastToSubscribers(events);
        }

        // Wait before next poll
        await this.delay(STELLAR_CONFIG.pollingInterval);
      } catch (error) {
        console.error('Error polling for events:', error);
        await this.delay(STELLAR_CONFIG.pollingInterval * 2); // Backoff
      }
    }
  }

  /**
   * Fetch events from Stellar RPC
   */
  private async fetchStellarEvents(fromLedger: number): Promise<any[]> {
    try {
      // Call Stellar RPC getEvents endpoint
      const response = await axios.post(STELLAR_CONFIG.rpcUrl, {
        jsonrpc: '2.0',
        id: 1,
        method: 'getEvents',
        params: {
          startLedger: fromLedger,
          filters: [
            {
              type: 'contract',
              contractIds: [this.contractId],
              topics: [
                ['AAAADwAAAAZjcmVhdGVkAA=='], // 'created' event topic
                ['AAAADwAAAAZmdW5kZWQAAA=='], // 'funded' event topic
                ['AAAAEAAAABBjb21wbGV0ZWQAAA=='], // 'completed' event topic
              ],
            },
          ],
          limit: 100,
        },
      });

      return response.data.result?.events || [];
    } catch (error) {
      console.error('Failed to fetch Stellar events:', error);
      return [];
    }
  }

  /**
   * Process individual event
   */
  private async processEvent(event: any): Promise<void> {
    try {
      const xdr = event.xdr || '';
      const eventType = this.determineEventType(event);
      const parsedData = XDRParser.parseEventXDR(xdr, eventType);

      const stellarEvent: StellarEvent = {
        id: `${event.id}`,
        type: eventType,
        contractId: this.contractId,
        ledgerSeq: event.ledger_sequence,
        ledgerClosedAt: new Date(event.ledger_close_time * 1000),
        txHash: event.pagingToken || '',
        data: parsedData,
        parsedAt: new Date(),
      };

      // Store in database
      await this.storeEvent(stellarEvent);

      // Emit for real-time processing
      this.emit('event', stellarEvent);
    } catch (error) {
      console.error('Error processing event:', error);
    }
  }

  /**
   * Determine event type from Stellar event
   */
  private determineEventType(event: any): string {
    const topics = event.topic || [];

    if (topics.some((t: any) => t.includes('created'))) return 'trade_created';
    if (topics.some((t: any) => t.includes('funded'))) return 'trade_funded';
    if (topics.some((t: any) => t.includes('completed'))) return 'trade_completed';
    if (topics.some((t: any) => t.includes('dispute'))) return 'dispute_raised';
    if (topics.some((t: any) => t.includes('resolved'))) return 'dispute_resolved';

    return 'unknown';
  }

  /**
   * Store event in database
   */
  private async storeEvent(event: StellarEvent): Promise<void> {
    try {
      await this.db.query(
        `INSERT INTO stellar_events 
        (event_id, contract_id, event_type, ledger_seq, ledger_closed_at, tx_hash, xdr_event, parsed_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (event_id) DO NOTHING`,
        [
          event.id,
          event.contractId,
          event.type,
          event.ledgerSeq,
          event.ledgerClosedAt,
          event.txHash,
          JSON.stringify(event.data), // Store full XDR
          JSON.stringify(event.data), // Parsed JSON
        ]
      );

      console.log(`✓ Event stored: ${event.id} (${event.type})`);
    } catch (error) {
      console.error('Failed to store event:', error);
    }
  }

  /**
   * Update indexer state
   */
  private async updateIndexerState(ledgerSeq: number): Promise<void> {
    try {
      await this.db.query(
        `INSERT INTO indexer_state (contract_id, last_ledger_seq, last_synced, sync_status)
        VALUES ($1, $2, NOW(), 'synced')
        ON CONFLICT (contract_id) DO UPDATE SET
          last_ledger_seq = $2,
          last_synced = NOW(),
          error_count = 0`,
        [this.contractId, ledgerSeq]
      );
    } catch (error) {
      console.error('Failed to update indexer state:', error);
    }
  }

  /**
   * Broadcast events to WebSocket subscribers
   */
  private broadcastToSubscribers(events: any[]): void {
    for (const [clientId, contractIds] of this.subscribedClients) {
      for (const event of events) {
        if (contractIds.has(event.contractId)) {
          // Send to client via WebSocket
          this.emit('broadcast', {
            clientId,
            event,
          });
        }
      }
    }
  }

  /**
   * Query historical events
   */
  async queryEvents(query: EventQuery): Promise<StellarEvent[]> {
    try {
      let sql = `SELECT * FROM stellar_events WHERE 1=1`;
      const params: any[] = [];

      if (query.contractId) {
        sql += ` AND contract_id = $${params.length + 1}`;
        params.push(query.contractId);
      }

      if (query.type) {
        sql += ` AND event_type = $${params.length + 1}`;
        params.push(query.type);
      }

      if (query.ledgerSeqFrom) {
        sql += ` AND ledger_seq >= $${params.length + 1}`;
        params.push(query.ledgerSeqFrom);
      }

      if (query.ledgerSeqTo) {
        sql += ` AND ledger_seq <= $${params.length + 1}`;
        params.push(query.ledgerSeqTo);
      }

      sql += ` ORDER BY ledger_seq DESC`;

      if (query.limit) {
        sql += ` LIMIT $${params.length + 1}`;
        params.push(query.limit);
      }

      if (query.offset) {
        sql += ` OFFSET $${params.length + 1}`;
        params.push(query.offset);
      }

      const result = await this.db.query(sql, params);

      return result.rows.map((row) => ({
        id: row.event_id,
        type: row.event_type,
        contractId: row.contract_id,
        ledgerSeq: row.ledger_seq,
        ledgerClosedAt: row.ledger_closed_at,
        txHash: row.tx_hash,
        data: row.parsed_data,
        parsedAt: row.parsed_at,
      }));
    } catch (error) {
      console.error('Error querying events:', error);
      return [];
    }
  }

  /**
   * Replay historical events
   */
  async replayEvents(startLedger: number, endLedger: number): Promise<string> {
    const replayId = `replay_${Date.now()}`;

    try {
      await this.db.query(
        `INSERT INTO event_replay_log (replay_id, start_ledger, end_ledger, status, started_at)
        VALUES ($1, $2, $3, 'in_progress', NOW())`,
        [replayId, startLedger, endLedger]
      );

      let eventsReplayed = 0;

      for (let ledger = startLedger; ledger <= endLedger; ledger += 100) {
        const events = await this.fetchStellarEvents(ledger);

        for (const event of events) {
          await this.processEvent(event);
          eventsReplayed++;
        }
      }

      // Mark replay completed
      await this.db.query(
        `UPDATE event_replay_log SET 
         status = 'completed', 
         events_replayed = $1, 
         completed_at = NOW()
        WHERE replay_id = $2`,
        [eventsReplayed, replayId]
      );

      console.log(`✓ Replay completed: ${eventsReplayed} events processed`);
      return replayId;
    } catch (error) {
      console.error('Replay failed:', error);

      await this.db.query(
        `UPDATE event_replay_log SET status = 'failed' WHERE replay_id = $1`,
        [replayId]
      );

      throw error;
    }
  }

  /**
   * WebSocket subscription management
   */
  addSubscriber(clientId: string, contractIds: string[]): void {
    this.subscribedClients.set(clientId, new Set(contractIds));
    console.log(`✓ Client ${clientId} subscribed to ${contractIds.length} contracts`);
  }

  removeSubscriber(clientId: string): void {
    this.subscribedClients.delete(clientId);
    console.log(`✓ Client ${clientId} unsubscribed`);
  }

  /**
   * Utility: delay
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  getStatus(): { isRunning: boolean; lastLedger: number; contractId: string } {
    return {
      isRunning: this.isRunning,
      lastLedger: this.lastLedgerSeq,
      contractId: this.contractId,
    };
  }
}

export default StellarEventIndexer;
