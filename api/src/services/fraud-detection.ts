import axios from 'axios';
import { EventEmitter } from 'events';
import * as pg from 'pg';

/**
 * FraudDetectionService: Hybrid Rule-Based + ML Fraud Detection System
 *
 * Features:
 * - Rule-based detection (transaction value, velocity, patterns)
 * - ML-based detection (scikit-learn model inference)
 * - Slack webhook alerts
 * - Fraud event logging to PostgreSQL
 * - Real-time dashboard updates
 */

// ============================================================================
// TYPES & CONSTANTS
// ============================================================================

interface Trade {
  trade_id: string;
  seller_id: string;
  buyer_id: string;
  amount_usd: number;
  amount_usdc: number;
  status: string;
  created_at: Date;
  funded_at?: Date;
  completed_at?: Date;
  disputed: boolean;
}

interface FraudAnalysis {
  trade_id: string;
  rule_based_score: number;
  ml_score: number;
  combined_score: number;
  risk_level: 'low' | 'medium' | 'high' | 'critical';
  rules_triggered: string[];
  ml_model_version: string;
  timestamp: Date;
  recommended_action: 'approve' | 'review' | 'block';
}

interface FraudAlert {
  alert_id: string;
  trade_id: string;
  risk_score: number;
  risk_level: string;
  reason: string;
  timestamp: Date;
  slack_posted: boolean;
}

interface MLFeatures {
  purchase_amount: number;
  user_age_days: number;
  previous_trades: number;
  disputes_ratio: number;
  time_since_last_trade_hours: number;
  trades_last_24h: number;
  trades_last_hour: number;
  avg_transaction_value: number;
  location_changes_24h: number;
  device_fingerprint_change: boolean;
}

// Fraud detection thresholds
const FRAUD_THRESHOLDS = {
  LARGE_TRANSACTION: 10000, // $10k
  ML_CRITICAL: 0.85,
  ML_HIGH: 0.65,
  ML_MEDIUM: 0.45,
  CMD_RATE_LIMIT: 5, // transactions per minute
};

// ============================================================================
// DATABASE SCHEMA & SETUP
// ============================================================================

const FRAUD_TABLES_SQL = `
-- Fraud detection events log
CREATE TABLE IF NOT EXISTS fraud_events (
  id SERIAL PRIMARY KEY,
  alert_id VARCHAR(255) UNIQUE,
  trade_id VARCHAR(255) NOT NULL,
  seller_id VARCHAR(255) NOT NULL,
  buyer_id VARCHAR(255) NOT NULL,
  amount_usd DECIMAL(18, 2) NOT NULL,
  rule_based_score DECIMAL(3, 2),
  ml_score DECIMAL(3, 2),
  combined_score DECIMAL(3, 2),
  risk_level VARCHAR(50) NOT NULL,
  rules_triggered JSONB,
  recommended_action VARCHAR(50),
  slack_posted BOOLEAN DEFAULT false,
  slack_message_id VARCHAR(255),
  manual_review BOOLEAN DEFAULT false,
  reviewed_by VARCHAR(255),
  final_verdict VARCHAR(50),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Fraud dashboard metrics
CREATE TABLE IF NOT EXISTS fraud_metrics (
  id SERIAL PRIMARY KEY,
  date DATE NOT NULL UNIQUE,
  total_trades BIGINT,
  flagged_trades BIGINT,
  high_risk_trades BIGINT,
  critical_risk_trades BIGINT,
  manual_reviews BIGINT,
  approved_trades BIGINT,
  blocked_trades BIGINT,
  false_positives BIGINT,
  false_negatives BIGINT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ML model version tracking
CREATE TABLE IF NOT EXISTS ml_models (
  id SERIAL PRIMARY KEY,
  model_name VARCHAR(255) NOT NULL,
  version VARCHAR(50) NOT NULL UNIQUE,
  deployment_date TIMESTAMP NOT NULL,
  accuracy DECIMAL(5, 4),
  precision DECIMAL(5, 4),
  recall DECIMAL(5, 4),
  f1_score DECIMAL(5, 4),
  training_data_points BIGINT,
  status VARCHAR(50),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_fraud_events_trade_id ON fraud_events(trade_id);
CREATE INDEX IF NOT EXISTS idx_fraud_events_risk_level ON fraud_events(risk_level);
CREATE INDEX IF NOT EXISTS idx_fraud_events_created_at ON fraud_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fraud_metrics_date ON fraud_metrics(date DESC);
`;

// ============================================================================
// FRAUD DETECTION SERVICE
// ============================================================================

class FraudDetectionService extends EventEmitter {
  private db: pg.Pool;
  private slackWebhookUrl: string;
  private mlServiceUrl: string;
  private userTradeCache: Map<string, Trade[]> = new Map();
  private alertQueue: FraudAlert[] = [];

  constructor(dbPool: pg.Pool) {
    super();
    this.db = dbPool;
    this.slackWebhookUrl = process.env.SLACK_WEBHOOK_URL || '';
    this.mlServiceUrl = process.env.ML_SERVICE_URL || 'http://ml-service:8000';
    this.initializeDatabase();
  }

  /**
   * Initialize fraud detection tables
   */
  private async initializeDatabase(): Promise<void> {
    try {
      await this.db.query(FRAUD_TABLES_SQL);
      console.log('✓ Fraud detection tables initialized');
    } catch (error) {
      console.error('Failed to initialize fraud tables:', error);
    }
  }

  /**
   * Analyze trade for fraud risk
   */
  async analyzeTrade(trade: Trade): Promise<FraudAnalysis> {
    const timestamp = new Date();

    // Step 1: Rule-based detection
    const ruleScore = this.calculateRuleBasedScore(trade);
    const triggeredRules = this.getTriggueredRules(trade);

    // Step 2: ML-based detection
    const mlScore = await this.getMlPrediction(trade);

    // Step 3: Combined risk score
    const combinedScore = (ruleScore + mlScore) / 2;

    // Step 4: Determine risk level and recommended action
    const riskLevel = this.getRiskLevel(combinedScore);
    const recommendedAction = this.getRecommendedAction(riskLevel, triggeredRules);

    const analysis: FraudAnalysis = {
      trade_id: trade.trade_id,
      rule_based_score: ruleScore,
      ml_score: mlScore,
      combined_score: combinedScore,
      risk_level: riskLevel,
      rules_triggered: triggeredRules,
      ml_model_version: await this.getActiveModelVersion(),
      timestamp,
      recommended_action: recommendedAction,
    };

    // Step 5: Log to database
    await this.logFraudAnalysis(analysis);

    // Step 6: Emit alert if high risk
    if (combinedScore > FRAUD_THRESHOLDS.ML_HIGH) {
      await this.raiseAlert(trade, analysis);
    }

    return analysis;
  }

  /**
   * Rule-based fraud detection (0.0 - 1.0)
   */
  private calculateRuleBasedScore(trade: Trade): number {
    let score = 0;

    // Rule 1: Large transaction amount (>$10k) → +0.3
    if (trade.amount_usd > FRAUD_THRESHOLDS.LARGE_TRANSACTION) {
      score += 0.3;
    }

    // Rule 2: New account (< 7 days old) → +0.25
    const sellerAge = Math.floor((Date.now() - trade.created_at.getTime()) / (24 * 60 * 60 * 1000));
    if (sellerAge < 7) {
      score += 0.25;
    }

    // Rule 3: High velocity (>5 trades/minute) → +0.35
    const tradesInLastMinute = (this.userTradeCache.get(trade.seller_id) || []).filter(
      (t) => Date.now() - t.created_at.getTime() < 60000
    ).length;

    if (tradesInLastMinute > FRAUD_THRESHOLDS.CMD_RATE_LIMIT) {
      score += 0.35;
    }

    // Rule 4: Unusual time pattern (3am-5am) → +0.1
    const hour = new Date().getHours();
    if (hour >= 3 && hour <= 5) {
      score += 0.1;
    }

    // Rule 5: Same amount repeated (duplicate detection) → +0.2
    const duplicates = (this.userTradeCache.get(trade.seller_id) || []).filter(
      (t) => t.amount_usd === trade.amount_usd && Date.now() - t.created_at.getTime() < 3600000
    ).length;

    if (duplicates > 2) {
      score += 0.2;
    }

    return Math.min(score, 1.0); // Normalize to 0-1
  }

  /**
   * Get list of triggered fraud rules
   */
  private getTriggueredRules(trade: Trade): string[] {
    const rules: string[] = [];

    if (trade.amount_usd > FRAUD_THRESHOLDS.LARGE_TRANSACTION) {
      rules.push('large_transaction_value');
    }

    const tradesInLastMinute = (this.userTradeCache.get(trade.seller_id) || []).filter(
      (t) => Date.now() - t.created_at.getTime() < 60000
    ).length;

    if (tradesInLastMinute > FRAUD_THRESHOLDS.CMD_RATE_LIMIT) {
      rules.push('abnormal_transaction_velocity');
    }

    const hour = new Date().getHours();
    if (hour >= 3 && hour <= 5) {
      rules.push('unusual_transaction_time');
    }

    return rules;
  }

  /**
   * Get ML-based fraud prediction (0.0 - 1.0)
   */
  private async getMlPrediction(trade: Trade): Promise<number> {
    try {
      // Build feature vector for ML model
      const features = await this.buildFeatureVector(trade);

      // Call ML service
      const response = await axios.post(`${this.mlServiceUrl}/predict`, {
        features,
        model_version: await this.getActiveModelVersion(),
      });

      const mlScore = response.data.fraud_probability || 0.5;
      console.log(`ML Prediction for trade ${trade.trade_id}: ${mlScore}`);

      return Math.min(Math.max(mlScore, 0), 1.0); // Clamp to 0-1
    } catch (error) {
      console.error('ML service error:', error);
      // Fallback to rule-based if ML unavailable
      return this.calculateRuleBasedScore(trade);
    }
  }

  /**
   * Build feature vector for ML model
   */
  private async buildFeatureVector(trade: Trade): Promise<MLFeatures> {
    const userTrades = this.userTradeCache.get(trade.seller_id) || [];
    const lastTrade = userTrades[userTrades.length - 1];

    return {
      purchase_amount: trade.amount_usd,
      user_age_days: Math.floor((Date.now() - trade.created_at.getTime()) / (24 * 60 * 60 * 1000)),
      previous_trades: userTrades.length,
      disputes_ratio: await this.getDisputeRatio(trade.seller_id),
      time_since_last_trade_hours: lastTrade
        ? Math.floor((Date.now() - lastTrade.created_at.getTime()) / (60 * 60 * 1000))
        : 999,
      trades_last_24h: userTrades.filter((t) => Date.now() - t.created_at.getTime() < 86400000).length,
      trades_last_hour: userTrades.filter((t) => Date.now() - t.created_at.getTime() < 3600000).length,
      avg_transaction_value:
        userTrades.reduce((sum, t) => sum + t.amount_usd, 0) / Math.max(userTrades.length, 1),
      location_changes_24h: await this.getLocationChanges(trade.seller_id),
      device_fingerprint_change: await this.checkDeviceChange(trade.seller_id),
    };
  }

  /**
   * Determine risk level based on combined score
   */
  private getRiskLevel(score: number): 'low' | 'medium' | 'high' | 'critical' {
    if (score >= FRAUD_THRESHOLDS.ML_CRITICAL) return 'critical';
    if (score >= FRAUD_THRESHOLDS.ML_HIGH) return 'high';
    if (score >= FRAUD_THRESHOLDS.ML_MEDIUM) return 'medium';
    return 'low';
  }

  /**
   * Get recommended action based on risk level
   */
  private getRecommendedAction(
    riskLevel: string,
    rules: string[]
  ): 'approve' | 'review' | 'block' {
    if (riskLevel === 'critical') return 'block';
    if (riskLevel === 'high' || rules.some((r) => r.includes('velocity'))) return 'review';
    return 'approve';
  }

  /**
   * Raise fraud alert and notify Slack
   */
  private async raiseAlert(trade: Trade, analysis: FraudAnalysis): Promise<void> {
    const alertId = `alert_${Date.now()}_${trade.trade_id}`;

    const alert: FraudAlert = {
      alert_id: alertId,
      trade_id: trade.trade_id,
      risk_score: analysis.combined_score,
      risk_level: analysis.risk_level,
      reason: `${analysis.rules_triggered.join(', ')} | ${analysis.recommended_action}`,
      timestamp: new Date(),
      slack_posted: false,
    };

    // Post to Slack if webhook configured
    if (this.slackWebhookUrl) {
      await this.postSlackAlert(trade, analysis, alert);
    }

    // Queue for manual review
    this.alertQueue.push(alert);

    // Update trade status to "Pending Manual Review"
    await this.updateTradeStatus(trade.trade_id, 'PENDING_REVIEW');

    // Emit event for real-time dashboards
    this.emit('fraud-alert', alert);
  }

  /**
   * Post alert to Slack
   */
  private async postSlackAlert(
    trade: Trade,
    analysis: FraudAnalysis,
    alert: FraudAlert
  ): Promise<void> {
    try {
      const color = analysis.risk_level === 'critical' ? '#FF0000' : '#FFA500'; // Red/Orange

      const payload = {
        attachments: [
          {
            color,
            title: `🚨 Fraud Alert: ${analysis.risk_level.toUpperCase()}`,
            fields: [
              {
                title: 'Trade ID',
                value: trade.trade_id,
                short: true,
              },
              {
                title: 'Seller ID',
                value: trade.seller_id,
                short: true,
              },
              {
                title: 'Amount',
                value: `$${trade.amount_usd.toFixed(2)} USDC`,
                short: true,
              },
              {
                title: 'Risk Score',
                value: `${(analysis.combined_score * 100).toFixed(1)}%`,
                short: true,
              },
              {
                title: 'Rules Triggered',
                value: analysis.rules_triggered.join(', ') || 'None',
                short: false,
              },
              {
                title: 'Recommended Action',
                value: analysis.recommended_action.toUpperCase(),
                short: true,
              },
              {
                title: 'Timestamp',
                value: new Date().toISOString(),
                short: true,
              },
            ],
            actions: [
              {
                type: 'button',
                text: 'Review in Dashboard',
                url: `${process.env.DASHBOARD_URL || 'http://localhost:3001'}/fraud/${trade.trade_id}`,
              },
            ],
          },
        ],
      };

      const response = await axios.post(this.slackWebhookUrl, payload);
      console.log('✓ Slack alert posted');

      alert.slack_posted = true;
    } catch (error) {
      console.error('Failed to post Slack alert:', error);
    }
  }

  /**
   * Log fraud analysis to database
   */
  private async logFraudAnalysis(analysis: FraudAnalysis): Promise<void> {
    try {
      await this.db.query(
        `INSERT INTO fraud_events 
        (alert_id, trade_id, rule_based_score, ml_score, combined_score, risk_level, rules_triggered, recommended_action)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
        [
          `alert_${Date.now()}_${analysis.trade_id}`,
          analysis.trade_id,
          analysis.rule_based_score,
          analysis.ml_score,
          analysis.combined_score,
          analysis.risk_level,
          JSON.stringify(analysis.rules_triggered),
          analysis.recommended_action,
        ]
      );
    } catch (error) {
      console.error('Failed to log fraud analysis:', error);
    }
  }

  /**
   * Update trade status in the blockchain
   */
  private async updateTradeStatus(tradeId: string, status: string): Promise<void> {
    // This would call the Soroban contract to update status
    console.log(`Updating trade ${tradeId} status to ${status}`);
  }

  /**
   * Helper: Get dispute ratio for user
   */
  private async getDisputeRatio(userId: string): Promise<number> {
    try {
      const result = await this.db.query(
        `SELECT 
          COUNT(*) as total,
          SUM(CASE WHEN disputed = true THEN 1 ELSE 0 END) as disputes
        FROM trades
        WHERE seller_id = $1 OR buyer_id = $1`,
        [userId]
      );

      const { total, disputes } = result.rows[0];
      return total > 0 ? disputes / total : 0;
    } catch {
      return 0;
    }
  }

  /**
   * Helper: Count location changes
   */
  private async getLocationChanges(userId: string): Promise<number> {
    // Placeholder: would integrate with IP geolocation service
    return 0;
  }

  /**
   * Helper: Check device fingerprint change
   */
  private async checkDeviceChange(userId: string): Promise<boolean> {
    // Placeholder: would integrate with device fingerprint service
    return false;
  }

  /**
   * Get active ML model version
   */
  private async getActiveModelVersion(): Promise<string> {
    try {
      const result = await this.db.query(
        `SELECT version FROM ml_models WHERE status = 'active' ORDER BY deployment_date DESC LIMIT 1`
      );

      return result.rows[0]?.version || 'v1.0.0';
    } catch {
      return 'v1.0.0';
    }
  }

  /**
   * Get fraud metrics for dashboard
   */
  async getFraudMetrics(days: number = 30): Promise<any[]> {
    try {
      const result = await this.db.query(
        `SELECT * FROM fraud_metrics WHERE date >= NOW() - INTERVAL '${days} days' ORDER BY date DESC`
      );

      return result.rows;
    } catch (error) {
      console.error('Failed to fetch fraud metrics:', error);
      return [];
    }
  }

  /**
   * Get pending review alerts
   */
  async getPendingReviews(): Promise<any[]> {
    try {
      const result = await this.db.query(
        `SELECT * FROM fraud_events WHERE manual_review = false AND risk_level IN ('high', 'critical') ORDER BY created_at DESC LIMIT 100`
      );

      return result.rows;
    } catch (error) {
      console.error('Failed to fetch pending reviews:', error);
      return [];
    }
  }
}

export { FraudDetectionService, FraudAnalysis, FraudAlert, Trade };
