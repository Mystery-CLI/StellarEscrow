import { SecurityFinding, SecuritySeverity } from './types';

export interface SecurityMonitoringEvent {
  type: string;
  severity: SecuritySeverity;
  message: string;
  source: string;
  timestamp: number;
  details?: Record<string, unknown>;
}

export interface SecurityMonitoringAlert {
  id: string;
  severity: SecuritySeverity;
  message: string;
  createdAt: number;
  eventType: string;
  details?: Record<string, unknown>;
}

export interface SecurityMonitorOptions {
  endpoint?: string;
  minimumSeverity?: SecuritySeverity;
  source?: string;
}

const severityRank: Record<SecuritySeverity, number> = {
  low: 0,
  medium: 1,
  high: 2,
  critical: 3,
};

const shouldAlert = (severity: SecuritySeverity, minimumSeverity: SecuritySeverity): boolean =>
  severityRank[severity] >= severityRank[minimumSeverity];

export class SecurityMonitor {
  private readonly endpoint?: string;
  private readonly minimumSeverity: SecuritySeverity;
  private readonly source: string;
  private readonly events: SecurityMonitoringEvent[] = [];
  private readonly alerts: SecurityMonitoringAlert[] = [];

  constructor(options: SecurityMonitorOptions = {}) {
    this.endpoint = options.endpoint;
    this.minimumSeverity = options.minimumSeverity ?? 'medium';
    this.source = options.source ?? 'security-assessment';
  }

  recordEvent(
    event: Omit<SecurityMonitoringEvent, 'timestamp' | 'source'> & Partial<Pick<SecurityMonitoringEvent, 'source'>>
  ): SecurityMonitoringEvent {
    const normalizedEvent: SecurityMonitoringEvent = {
      ...event,
      source: event.source ?? this.source,
      timestamp: Date.now(),
    };

    this.events.push(normalizedEvent);

    if (shouldAlert(normalizedEvent.severity, this.minimumSeverity)) {
      this.alerts.push({
        id: `${normalizedEvent.type}-${normalizedEvent.timestamp}-${this.alerts.length + 1}`,
        severity: normalizedEvent.severity,
        message: normalizedEvent.message,
        createdAt: normalizedEvent.timestamp,
        eventType: normalizedEvent.type,
        details: normalizedEvent.details,
      });
    }

    return normalizedEvent;
  }

  recordFinding(finding: SecurityFinding): void {
    this.recordEvent({
      type: finding.passed ? 'assessment_passed' : 'assessment_failed',
      severity: finding.passed ? 'low' : finding.severity,
      message: `${finding.title}: ${finding.passed ? 'passed' : 'failed'}`,
      details: {
        category: finding.category,
        control: finding.control,
        metadata: finding.metadata,
      },
    });
  }

  observePolicyViolations(): () => void {
    if (typeof window === 'undefined' || typeof window.addEventListener !== 'function') {
      return () => undefined;
    }

    const handler = (event: Event) => {
      const policyEvent = event as Event & {
        blockedURI?: string;
        violatedDirective?: string;
        effectiveDirective?: string;
      };

      this.recordEvent({
        type: 'policy_violation',
        severity: 'high',
        message: `Security policy violation detected for ${
          policyEvent.violatedDirective ?? policyEvent.effectiveDirective ?? 'unknown-directive'
        }`,
        details: {
          blockedURI: policyEvent.blockedURI ?? 'unknown',
          violatedDirective: policyEvent.violatedDirective ?? policyEvent.effectiveDirective ?? 'unknown',
        },
      });
    };

    window.addEventListener('securitypolicyviolation', handler as EventListener);
    return () => window.removeEventListener('securitypolicyviolation', handler as EventListener);
  }

  flush(endpoint = this.endpoint): boolean {
    if (!endpoint || typeof navigator === 'undefined' || typeof navigator.sendBeacon !== 'function') {
      return false;
    }

    const payload = JSON.stringify({
      source: this.source,
      generatedAt: Date.now(),
      events: this.events,
      alerts: this.alerts,
    });

    return navigator.sendBeacon(endpoint, payload);
  }

  getEvents(): SecurityMonitoringEvent[] {
    return [...this.events];
  }

  getAlerts(): SecurityMonitoringAlert[] {
    return [...this.alerts];
  }

  getSummary(): {
    totalEvents: number;
    totalAlerts: number;
    highestSeverity: SecuritySeverity | null;
  } {
    const highestSeverity =
      this.events
        .map((event) => event.severity)
        .sort((left, right) => severityRank[right] - severityRank[left])[0] ?? null;

    return {
      totalEvents: this.events.length,
      totalAlerts: this.alerts.length,
      highestSeverity,
    };
  }

  reset(): void {
    this.events.length = 0;
    this.alerts.length = 0;
  }
}
