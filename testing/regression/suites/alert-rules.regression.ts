/**
 * Alert Rules Regression Suite
 *
 * Ensures monitoring alert thresholds and evaluation logic haven't changed.
 * Catches accidental threshold changes that could silence critical alerts.
 */

import { evaluate_rule, default_alert_rules } from '../helpers/alert-helpers';

describe('Alert Rules Regression — Default rules exist', () => {
  const rules = default_alert_rules();
  const ruleNames = rules.map(r => r.name);

  const required = [
    'high_error_rate',
    'high_dispute_rate',
    'compliance_blocks_spike',
    'aml_high_risk_spike',
    'fraud_alerts_spike',
  ];

  required.forEach(name => {
    it(`rule "${name}" is defined`, () => {
      expect(ruleNames).toContain(name);
    });
  });
});

describe('Alert Rules Regression — Thresholds', () => {
  const rules = default_alert_rules();
  const byName = Object.fromEntries(rules.map(r => [r.name, r]));

  it('high_error_rate threshold is 5%', () => {
    expect(byName['high_error_rate'].threshold).toBe(5);
    expect(byName['high_error_rate'].severity).toBe('critical');
  });

  it('aml_high_risk_spike threshold is 5', () => {
    expect(byName['aml_high_risk_spike'].threshold).toBe(5);
    expect(byName['aml_high_risk_spike'].severity).toBe('critical');
  });

  it('compliance_blocks_spike threshold is 10', () => {
    expect(byName['compliance_blocks_spike'].threshold).toBe(10);
    expect(byName['compliance_blocks_spike'].severity).toBe('high');
  });

  it('fraud_alerts_spike threshold is 20', () => {
    expect(byName['fraud_alerts_spike'].threshold).toBe(20);
    expect(byName['fraud_alerts_spike'].severity).toBe('high');
  });
});

describe('Alert Rules Regression — Evaluation logic', () => {
  it('fires when metric exceeds threshold (gt)', () => {
    const alert = evaluate_rule(
      { name: 'test', metric: 'error_rate', threshold: 5, operator: 'gt', severity: 'critical', message_template: 'Error {value}' },
      { error_rate: 6 }
    );
    expect(alert).not.toBeNull();
    expect(alert!.current_value).toBe(6);
  });

  it('does not fire when metric is at threshold (gt)', () => {
    const alert = evaluate_rule(
      { name: 'test', metric: 'error_rate', threshold: 5, operator: 'gt', severity: 'critical', message_template: 'Error {value}' },
      { error_rate: 5 }
    );
    expect(alert).toBeNull();
  });

  it('fires when metric is at threshold (gte)', () => {
    const alert = evaluate_rule(
      { name: 'test', metric: 'error_rate', threshold: 5, operator: 'gte', severity: 'high', message_template: 'Error {value}' },
      { error_rate: 5 }
    );
    expect(alert).not.toBeNull();
  });

  it('returns null when metric is missing from snapshot', () => {
    const alert = evaluate_rule(
      { name: 'test', metric: 'missing_metric', threshold: 5, operator: 'gt', severity: 'high', message_template: 'x' },
      { error_rate: 10 }
    );
    expect(alert).toBeNull();
  });

  it('interpolates {value} and {threshold} in message', () => {
    const alert = evaluate_rule(
      { name: 'test', metric: 'error_rate', threshold: 5, operator: 'gt', severity: 'critical', message_template: 'Rate {value} > {threshold}' },
      { error_rate: 8 }
    );
    expect(alert!.message).toBe('Rate 8 > 5');
  });
});
