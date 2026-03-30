import { Router, Request, Response } from 'express';
import { StellarEventIndexer } from './event-indexer';
import * as pg from 'pg';

/**
 * Stellar Events REST API
 * Query endpoints for historical and real-time event data
 */

export class StellarEventsAPI {
  private router: Router;
  private indexer: StellarEventIndexer;
  private db: pg.Pool;

  constructor(indexer: StellarEventIndexer, db: pg.Pool) {
    this.indexer = indexer;
    this.db = db;
    this.router = Router();
    this.setupRoutes();
  }

  private setupRoutes(): void {
    /**
     * GET /events
     * Query events with filters
     *
     * Query Parameters:
     * - contractId: string (required or via filter)
     * - type: string (trade_created, trade_funded, etc.)
     * - ledgerSeqFrom: number
     * - ledgerSeqTo: number
     * - limit: number (default: 100, max: 1000)
     * - offset: number (default: 0)
     */
    this.router.get('/events', this.getEvents.bind(this));

    /**
     * GET /events/:eventId
     * Get specific event details
     */
    this.router.get('/events/:eventId', this.getEventDetail.bind(this));

    /**
     * GET /contracts/:contractId/events
     * Get events for specific contract
     */
    this.router.get('/contracts/:contractId/events', this.getContractEvents.bind(this));

    /**
     * GET /contracts/:contractId/stats
     * Get event statistics for a contract
     */
    this.router.get('/contracts/:contractId/stats', this.getContractStats.bind(this));

    /**
     * GET /indexer/status
     * Get indexer status and sync progress
     */
    this.router.get('/indexer/status', this.getIndexerStatus.bind(this));

    /**
     * POST /replay
     * Start historical event replay
     */
    this.router.post('/replay', this.startReplay.bind(this));

    /**
     * GET /replay/:replayId
     * Get replay job status
     */
    this.router.get('/replay/:replayId', this.getReplayStatus.bind(this));

    /**
     * GET /events/trade/:tradeId
     * Get all events for a specific trade
     */
    this.router.get('/events/trade/:tradeId', this.getTradeEvents.bind(this));

    /**
     * GET /events/stats
     * Get global event statistics
     */
    this.router.get('/events/stats', this.getGlobalStats.bind(this));
  }

  /**
   * GET /events
   * Query events with multiple filters
   */
  private async getEvents(req: Request, res: Response): Promise<void> {
    try {
      const {
        contractId,
        type,
        ledgerSeqFrom,
        ledgerSeqTo,
        limit = 100,
        offset = 0,
      } = req.query;

      // Validate limit
      const parsedLimit = Math.min(parseInt(limit as string) || 100, 1000);
      const parsedOffset = parseInt(offset as string) || 0;

      const query = {
        contractId: contractId as string,
        type: type as string,
        ledgerSeqFrom: ledgerSeqFrom ? parseInt(ledgerSeqFrom as string) : undefined,
        ledgerSeqTo: ledgerSeqTo ? parseInt(ledgerSeqTo as string) : undefined,
        limit: parsedLimit,
        offset: parsedOffset,
      };

      const events = await this.indexer.queryEvents(query);

      // Get total count
      const countResult = await this.db.query(
        `SELECT COUNT(*) as total FROM stellar_events WHERE contract_id = $1`,
        [contractId]
      );

      res.json({
        success: true,
        events,
        count: events.length,
        total: parseInt(countResult.rows[0].total),
        limit: parsedLimit,
        offset: parsedOffset,
        hasMore: parsedOffset + parsedLimit < parseInt(countResult.rows[0].total),
      });
    } catch (error) {
      console.error('Error querying events:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to query events',
      });
    }
  }

  /**
   * GET /events/:eventId
   * Get specific event
   */
  private async getEventDetail(req: Request, res: Response): Promise<void> {
    try {
      const { eventId } = req.params;

      const result = await this.db.query(
        `SELECT * FROM stellar_events WHERE event_id = $1`,
        [eventId]
      );

      if (result.rows.length === 0) {
        return res.status(404).json({
          success: false,
          error: 'Event not found',
        });
      }

      const row = result.rows[0];
      res.json({
        success: true,
        event: {
          id: row.event_id,
          type: row.event_type,
          contractId: row.contract_id,
          ledgerSeq: row.ledger_seq,
          ledgerClosedAt: row.ledger_closed_at,
          txHash: row.tx_hash,
          data: row.parsed_data,
          parsedAt: row.parsed_at,
        },
      });
    } catch (error) {
      console.error('Error fetching event:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch event',
      });
    }
  }

  /**
   * GET /contracts/:contractId/events
   * Get all events for contract
   */
  private async getContractEvents(req: Request, res: Response): Promise<void> {
    try {
      const { contractId } = req.params;
      const { type, limit = 100, offset = 0 } = req.query;

      const parsedLimit = Math.min(parseInt(limit as string) || 100, 1000);
      const parsedOffset = parseInt(offset as string) || 0;

      let sql = `SELECT * FROM stellar_events WHERE contract_id = $1`;
      const params: any[] = [contractId];

      if (type) {
        sql += ` AND event_type = $2`;
        params.push(type);
      }

      sql += ` ORDER BY ledger_seq DESC LIMIT $${params.length + 1} OFFSET $${params.length + 2}`;
      params.push(parsedLimit, parsedOffset);

      const result = await this.db.query(sql, params);

      // Get total count
      const countResult = await this.db.query(
        `SELECT COUNT(*) as total FROM stellar_events WHERE contract_id = $1`,
        [contractId]
      );

      res.json({
        success: true,
        contractId,
        events: result.rows.map((row) => ({
          id: row.event_id,
          type: row.event_type,
          ledgerSeq: row.ledger_seq,
          ledgerClosedAt: row.ledger_closed_at,
          data: row.parsed_data,
        })),
        count: result.rows.length,
        total: parseInt(countResult.rows[0].total),
        limit: parsedLimit,
        offset: parsedOffset,
      });
    } catch (error) {
      console.error('Error fetching contract events:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch contract events',
      });
    }
  }

  /**
   * GET /contracts/:contractId/stats
   * Get event statistics
   */
  private async getContractStats(req: Request, res: Response): Promise<void> {
    try {
      const { contractId } = req.params;

      const result = await this.db.query(
        `SELECT 
          COUNT(*) as total_events,
          COUNT(DISTINCT event_type) as unique_event_types,
          MIN(ledger_seq) as first_ledger,
          MAX(ledger_seq) as last_ledger,
          MAX(parsed_at) as last_event_time
        FROM stellar_events
        WHERE contract_id = $1`,
        [contractId]
      );

      const eventTypeResult = await this.db.query(
        `SELECT event_type, COUNT(*) as count
        FROM stellar_events
        WHERE contract_id = $1
        GROUP BY event_type
        ORDER BY count DESC`,
        [contractId]
      );

      const stats = result.rows[0];
      res.json({
        success: true,
        contractId,
        statistics: {
          totalEvents: parseInt(stats.total_events),
          uniqueEventTypes: parseInt(stats.unique_event_types),
          firstLedger: stats.first_ledger,
          lastLedger: stats.last_ledger,
          lastEventTime: stats.last_event_time,
        },
        eventTypeBreakdown: eventTypeResult.rows.map((row) => ({
          type: row.event_type,
          count: parseInt(row.count),
        })),
      });
    } catch (error) {
      console.error('Error fetching contract stats:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch statistics',
      });
    }
  }

  /**
   * GET /indexer/status
   * Get indexer sync status
   */
  private async getIndexerStatus(req: Request, res: Response): Promise<void> {
    try {
      const status = this.indexer.getStatus();

      const result = await this.db.query(
        `SELECT * FROM indexer_state WHERE contract_id = $1`,
        [status.contractId]
      );

      const state = result.rows[0] || {};

      res.json({
        success: true,
        status: {
          isRunning: status.isRunning,
          lastIndexedLedger: status.lastLedger,
          contractId: status.contractId,
          ...state,
        },
      });
    } catch (error) {
      console.error('Error fetching indexer status:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch indexer status',
      });
    }
  }

  /**
   * POST /replay
   * Start historical event replay
   */
  private async startReplay(req: Request, res: Response): Promise<void> {
    try {
      const { startLedger, endLedger } = req.body;

      if (!startLedger || !endLedger) {
        return res.status(400).json({
          success: false,
          error: 'startLedger and endLedger required',
        });
      }

      const replayId = await this.indexer.replayEvents(startLedger, endLedger);

      res.json({
        success: true,
        replayId,
        message: `Replay started from ledger ${startLedger} to ${endLedger}`,
      });
    } catch (error) {
      console.error('Error starting replay:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to start replay',
      });
    }
  }

  /**
   * GET /replay/:replayId
   * Get replay job status
   */
  private async getReplayStatus(req: Request, res: Response): Promise<void> {
    try {
      const { replayId } = req.params;

      const result = await this.db.query(
        `SELECT * FROM event_replay_log WHERE replay_id = $1`,
        [replayId]
      );

      if (result.rows.length === 0) {
        return res.status(404).json({
          success: false,
          error: 'Replay not found',
        });
      }

      res.json({
        success: true,
        replay: result.rows[0],
      });
    } catch (error) {
      console.error('Error fetching replay status:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch replay status',
      });
    }
  }

  /**
   * GET /events/trade/:tradeId
   * Get all events related to a trade
   */
  private async getTradeEvents(req: Request, res: Response): Promise<void> {
    try {
      const { tradeId } = req.params;

      // Search for trade events in parsed_data
      const result = await this.db.query(
        `SELECT * FROM stellar_events 
        WHERE parsed_data->>'tradeId' = $1 OR parsed_data->>'trade_id' = $1
        ORDER BY ledger_seq DESC`,
        [tradeId]
      );

      res.json({
        success: true,
        tradeId,
        events: result.rows.map((row) => ({
          id: row.event_id,
          type: row.event_type,
          ledgerSeq: row.ledger_seq,
          data: row.parsed_data,
          parsedAt: row.parsed_at,
        })),
        count: result.rows.length,
      });
    } catch (error) {
      console.error('Error fetching trade events:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch trade events',
      });
    }
  }

  /**
   * GET /events/stats
   * Get global statistics
   */
  private async getGlobalStats(req: Request, res: Response): Promise<void> {
    try {
      const result = await this.db.query(
        `SELECT 
          COUNT(*) as total_events,
          COUNT(DISTINCT contract_id) as contract_count,
          COUNT(DISTINCT event_type) as event_type_count,
          MAX(ledger_seq) as max_ledger,
          MAX(parsed_at) as latest_event_time
        FROM stellar_events`
      );

      const stats = result.rows[0];

      res.json({
        success: true,
        statistics: {
          totalEvents: parseInt(stats.total_events),
          contractCount: parseInt(stats.contract_count),
          eventTypeCount: parseInt(stats.event_type_count),
          maxLedger: stats.max_ledger,
          latestEventTime: stats.latest_event_time,
        },
      });
    } catch (error) {
      console.error('Error fetching global stats:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to fetch statistics',
      });
    }
  }

  getRouter(): Router {
    return this.router;
  }
}

export default StellarEventsAPI;
