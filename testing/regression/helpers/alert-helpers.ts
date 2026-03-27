/**
 * TypeScript mirror of the Rust alert rule evaluation logic.
 * Keeps regression tests independent of the compiled Rust binary.
 */

export interface AlertRule {
  name: string;
  metric: string;
  threshold: number;
  operator: 'gt' | 'gte' | 'lt' | 'lte';
  severity: 'info' | 'warning' | 'high' | 'critical';
  message_template: string;
}

export interface AlertState {
  rule_name: string;
  severity: string;
  current_value: number;
  threshold: number;
  message: string;
}

export function evaluate_rule(
  rule: AlertRule,
  metrics: Record<string, number>
): AlertState | null {
  const value = metrics[rule.metric];
  if (value === undefined) return null;

  const triggered =
    rule.operator === 'gt'  ? value > rule.threshold  :
    rule.operator === 'gte' ? value >= rule.threshold :
    rule.operator === 'lt'  ? value < rule.threshold  :
    rule.operator === 'lte' ? value <= rule.threshold :
    false;

  if (!triggered) return null;

  return {
    rule_name: rule.name,
    severity: rule.severity,
    current_value: value,
    threshold: rule.threshold,
    message: rule.message_template
      .replace('{value}', String(value))
      .replace('{threshold}', String(rule.threshold)),
  };
}

export function default_alert_rules(): AlertRule[] {
  return [
    { name: 'high_error_rate',          metric: 'stellar_escrow_error_rate',                  threshold: 5,  operator: 'gt', severity: 'critical', message_template: 'Error rate {value}% exceeds threshold {threshold}%' },
    { name: 'high_dispute_rate',        metric: 'stellar_escrow_trades_disputed_total',        threshold: 50, operator: 'gt', severity: 'high',     message_template: 'Dispute count {value} exceeds threshold {threshold}' },
    { name: 'compliance_blocks_spike',  metric: 'stellar_escrow_compliance_blocked_total',     threshold: 10, operator: 'gt', severity: 'high',     message_template: 'Compliance blocks {value} exceeds threshold {threshold}' },
    { name: 'aml_high_risk_spike',      metric: 'stellar_escrow_aml_high_risk_total',          threshold: 5,  operator: 'gt', severity: 'critical', message_template: 'AML high-risk addresses {value} exceeds threshold {threshold}' },
    { name: 'fraud_alerts_spike',       metric: 'stellar_escrow_fraud_alerts_total',           threshold: 20, operator: 'gt', severity: 'high',     message_template: 'Fraud alerts {value} exceeds threshold {threshold}' },
  ];
}
