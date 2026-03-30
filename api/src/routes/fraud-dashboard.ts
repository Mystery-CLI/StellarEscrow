import { Router, Request, Response } from 'express';
import * as pg from 'pg';
import axios from 'axios';

/**
 * Fraud Dashboard APIs & Schema
 * Provides analytics and monitoring endpoints for fraud detection
 */

interface FraudDashboardStats {
  totalTrades: number;
  flaggedTrades: number;
  criticalRiskTrades: number;
  highRiskTrades: number;
  approvalRate: number;
  falsePositiveRate: number;
  avgRiskScore: number;
}

interface FraudTimeSeries {
  date: Date;
  totalTrades: number;
  flaggedTrades: number;
  criticalTrades: number;
  manualReviews: number;
}

// ============================================================================
// FRAUD DASHBOARD DATABASE SCHEMA
// ============================================================================

export const FRAUD_DASHBOARD_SCHEMA = `
-- Aggregate fraud metrics by day
CREATE TABLE IF NOT EXISTS fraud_daily_metrics (
  id SERIAL PRIMARY KEY,
  date DATE NOT NULL UNIQUE,
  total_trades BIGINT DEFAULT 0,
  flagged_trades BIGINT DEFAULT 0,
  critical_risk BIGINT DEFAULT 0,
  high_risk BIGINT DEFAULT 0,
  medium_risk BIGINT DEFAULT 0,
  low_risk BIGINT DEFAULT 0,
  manual_reviews BIGINT DEFAULT 0,
  approved_count BIGINT DEFAULT 0,
  blocked_count BIGINT DEFAULT 0,
  avg_risk_score DECIMAL(3, 2) DEFAULT 0,
  false_positives BIGINT DEFAULT 0,
  false_negatives BIGINT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Fraud alerts queue for manual review
CREATE TABLE IF NOT EXISTS fraud_alerts_queue (
  id SERIAL PRIMARY KEY,
  alert_id VARCHAR(255) UNIQUE NOT NULL,
  trade_id VARCHAR(255) NOT NULL,
  seller_id VARCHAR(255),
  buyer_id VARCHAR(255),
  amount_usd DECIMAL(18, 2),
  risk_score DECIMAL(3, 2),
  risk_level VARCHAR(50),
  status VARCHAR(50) DEFAULT 'pending',  -- pending, reviewed, approved, blocked
  reviewer_id VARCHAR(255),
  review_notes TEXT,
  reviewed_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Model performance tracking
CREATE TABLE IF NOT EXISTS ml_model_performance (
  id SERIAL PRIMARY KEY,
  model_id VARCHAR(255),
  model_version VARCHAR(50),
  date DATE,
  true_positives BIGINT,
  true_negatives BIGINT,
  false_positives BIGINT,
  false_negatives BIGINT,
  precision DECIMAL(5, 4),
  recall DECIMAL(5, 4),
  f1_score DECIMAL(5, 4),
  roc_auc DECIMAL(5, 4),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- User fraud history
CREATE TABLE IF NOT EXISTS user_fraud_history (
  id SERIAL PRIMARY KEY,
  user_id VARCHAR(255) NOT NULL,
  total_trades BIGINT DEFAULT 0,
  flagged_count BIGINT DEFAULT 0,
  average_risk_score DECIMAL(3, 2),
  is_high_risk BOOLEAN DEFAULT false,
  last_flagged_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_fraud_daily_metrics_date ON fraud_daily_metrics(date DESC);
CREATE INDEX idx_fraud_alerts_status ON fraud_alerts_queue(status);
CREATE INDEX idx_fraud_alerts_created ON fraud_alerts_queue(created_at DESC);
CREATE INDEX idx_user_fraud_history_user_id ON user_fraud_history(user_id);
`;

// ============================================================================
// FRAUD DASHBOARD SERVICE
// ============================================================================

export class FraudDashboardService {
  private db: pg.Pool;
  private router: Router;

  constructor(dbPool: pg.Pool) {
    this.db = dbPool;
    this.router = Router();
    this.setupRoutes();
  }

  private setupRoutes(): void {
    // Dashboard overview
    this.router.get('/stats', this.getDashboardStats.bind(this));
    this.router.get('/time-series', this.getTimeSeries.bind(this));
    this.router.get('/pending-reviews', this.getPendingReviews.bind(this));
    this.router.get('/alerts/:alertId', this.getAlertDetail.bind(this));
    this.router.post('/alerts/:alertId/review', this.reviewAlert.bind(this));
    this.router.get('/user-risk/:userId', this.getUserRiskProfile.bind(this));
    this.router.get('/model-performance', this.getModelPerformance.bind(this));
  }

  /**
   * GET /fraud/stats
   * Dashboard overview statistics
   */
  private async getDashboardStats(req: Request, res: Response): Promise<void> {
    try {
      // Get today's stats
      const result = await this.db.query(
        `SELECT 
          total_trades,
          flagged_trades,
          critical_risk + high_risk as high_risk_count,
          avg_risk_score,
          approveals,
          blocked_count
        FROM fraud_daily_metrics
        WHERE date = CURRENT_DATE`
      );

      const stats = result.rows[0] || {
        total_trades: 0,
        flagged_trades: 0,
        high_risk_count: 0,
        avg_risk_score: 0,
      };

      const response: FraudDashboardStats = {
        totalTrades: stats.total_trades,
        flaggedTrades: stats.flagged_trades,
        criticalRiskTrades: 0,
        highRiskTrades: stats.high_risk_count,
        approvalRate: stats.total_trades > 0 ? (stats.total_trades - stats.flagged_trades) / stats.total_trades : 0,
        falsePositiveRate: 0, // Would calculate from feedback
        avgRiskScore: stats.avg_risk_score,
      };

      res.json(response);
    } catch (error) {
      console.error('Error fetching dashboard stats:', error);
      res.status(500).json({ error: 'Failed to fetch stats' });
    }
  }

  /**
   * GET /fraud/time-series
   * Historical fraud metrics over time
   */
  private async getTimeSeries(req: Request, res: Response): Promise<void> {
    try {
      const days = parseInt(req.query.days as string) || 30;

      const result = await this.db.query(
        `SELECT 
          date,
          total_trades,
          flagged_trades,
          critical_risk,
          manual_reviews
        FROM fraud_daily_metrics
        WHERE date >= CURRENT_DATE - INTERVAL '${days} days'
        ORDER BY date DESC`
      );

      res.json({
        period_days: days,
        data: result.rows,
      });
    } catch (error) {
      console.error('Error fetching time series:', error);
      res.status(500).json({ error: 'Failed to fetch time series' });
    }
  }

  /**
   * GET /fraud/pending-reviews
   * Get queue of alerts pending manual review
   */
  private async getPendingReviews(req: Request, res: Response): Promise<void> {
    try {
      const limit = parseInt(req.query.limit as string) || 50;
      const offset = parseInt(req.query.offset as string) || 0;

      const result = await this.db.query(
        `SELECT 
          alert_id,
          trade_id,
          seller_id,
          buyer_id,
          amount_usd,
          risk_score,
          risk_level,
          created_at
        FROM fraud_alerts_queue
        WHERE status = 'pending'
        ORDER BY risk_score DESC, created_at DESC
        LIMIT $1 OFFSET $2`,
        [limit, offset]
      );

      const countResult = await this.db.query(
        `SELECT COUNT(*) FROM fraud_alerts_queue WHERE status = 'pending'`
      );

      res.json({
        total_pending: parseInt(countResult.rows[0].count),
        limit,
        offset,
        alerts: result.rows,
      });
    } catch (error) {
      console.error('Error fetching pending reviews:', error);
      res.status(500).json({ error: 'Failed to fetch pending reviews' });
    }
  }

  /**
   * GET /fraud/alerts/:alertId
   * Get detailed alert information
   */
  private async getAlertDetail(req: Request, res: Response): Promise<void> {
    try {
      const { alertId } = req.params;

      const result = await this.db.query(
        `SELECT * FROM fraud_alerts_queue WHERE alert_id = $1`,
        [alertId]
      );

      if (result.rows.length === 0) {
        return res.status(404).json({ error: 'Alert not found' });
      }

      res.json(result.rows[0]);
    } catch (error) {
      console.error('Error fetching alert detail:', error);
      res.status(500).json({ error: 'Failed to fetch alert' });
    }
  }

  /**
   * POST /fraud/alerts/:alertId/review
   * Submit manual review decision
   */
  private async reviewAlert(req: Request, res: Response): Promise<void> {
    try {
      const { alertId } = req.params;
      const { decision, notes, reviewerId } = req.body;

      if (!['approved', 'blocked'].includes(decision)) {
        return res.status(400).json({ error: 'Invalid decision' });
      }

      // Update alert status
      await this.db.query(
        `UPDATE fraud_alerts_queue 
        SET status = $1, reviewer_id = $2, review_notes = $3, reviewed_at = NOW()
        WHERE alert_id = $4`,
        [decision, reviewerId, notes, alertId]
      );

      // Update trade status in blockchain
      const alertResult = await this.db.query(
        `SELECT trade_id FROM fraud_alerts_queue WHERE alert_id = $1`,
        [alertId]
      );

      if (alertResult.rows.length > 0) {
        const tradeId = alertResult.rows[0].trade_id;
        const newStatus = decision === 'approved' ? 'ACTIVE' : 'BLOCKED';
        // Call Stellar contract to update trade status
        // await updateTradeStatusInContract(tradeId, newStatus);
      }

      res.json({
        success: true,
        alertId,
        decision,
        message: `Alert ${decision} by reviewer ${reviewerId}`,
      });
    } catch (error) {
      console.error('Error reviewing alert:', error);
      res.status(500).json({ error: 'Failed to review alert' });
    }
  }

  /**
   * GET /fraud/user-risk/:userId
   * Get fraud risk profile for a user
   */
  private async getUserRiskProfile(req: Request, res: Response): Promise<void> {
    try {
      const { userId } = req.params;

      const result = await this.db.query(
        `SELECT 
          user_id,
          total_trades,
          flagged_count,
          average_risk_score,
          is_high_risk,
          last_flagged_at
        FROM user_fraud_history
        WHERE user_id = $1`,
        [userId]
      );

      if (result.rows.length === 0) {
        return res.status(404).json({ userRiskProfile: null });
      }

      res.json({
        userRiskProfile: result.rows[0],
      });
    } catch (error) {
      console.error('Error fetching user risk profile:', error);
      res.status(500).json({ error: 'Failed to fetch user profile' });
    }
  }

  /**
   * GET /fraud/model-performance
   * Get ML model performance metrics
   */
  private async getModelPerformance(req: Request, res: Response): Promise<void> {
    try {
      const days = parseInt(req.query.days as string) || 30;

      const result = await this.db.query(
        `SELECT 
          model_version,
          date,
          precision,
          recall,
          f1_score,
          roc_auc
        FROM ml_model_performance
        WHERE date >= CURRENT_DATE - INTERVAL '${days} days'
        ORDER BY date DESC`
      );

      res.json({
        period_days: days,
        models: result.rows,
      });
    } catch (error) {
      console.error('Error fetching model performance:', error);
      res.status(500).json({ error: 'Failed to fetch model performance' });
    }
  }

  /**
   * Update daily metrics (run hourly/daily)
   */
  async updateDailyMetrics(): Promise<void> {
    try {
      const today = new Date().toISOString().split('T')[0];

      // Calculate metrics for today
      const result = await this.db.query(
        `SELECT 
          COUNT(*) as total_trades,
          SUM(CASE WHEN risk_level IN ('high', 'critical') THEN 1 ELSE 0 END) as flagged,
          SUM(CASE WHEN risk_level = 'critical' THEN 1 ELSE 0 END) as critical,
          AVG(combined_score) as avg_score
        FROM fraud_events
        WHERE DATE(created_at) = $1`,
        [today]
      );

      const metrics = result.rows[0];

      // Upsert into daily metrics
      await this.db.query(
        `INSERT INTO fraud_daily_metrics (date, total_trades, flagged_trades, critical_risk, avg_risk_score)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (date) DO UPDATE SET
          total_trades = $2,
          flagged_trades = $3,
          critical_risk = $4,
          avg_risk_score = $5,
          updated_at = NOW()`,
        [today, metrics.total_trades, metrics.flagged, metrics.critical, metrics.avg_score]
      );

      console.log(`✓ Daily fraud metrics updated for ${today}`);
    } catch (error) {
      console.error('Error updating daily metrics:', error);
    }
  }

  getRouter(): Router {
    return this.router;
  }
}

export default FraudDashboardService;
