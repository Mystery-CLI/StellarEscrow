export type SecuritySeverity = 'low' | 'medium' | 'high' | 'critical';
export type SecurityAssessmentType = 'penetration' | 'vulnerability' | 'compliance';

export interface SecurityFinding {
  id: string;
  title: string;
  category: SecurityAssessmentType;
  severity: SecuritySeverity;
  passed: boolean;
  description: string;
  remediation: string;
  control?: string;
  metadata?: Record<string, unknown>;
}

export interface SecurityReportSummary {
  total: number;
  passed: number;
  failed: number;
  low: number;
  medium: number;
  high: number;
  critical: number;
}

export interface SecurityReport {
  type: SecurityAssessmentType;
  passed: boolean;
  findings: SecurityFinding[];
  summary: SecurityReportSummary;
}

export interface SecurityRateLimitTarget {
  maxAttempts?: number;
  windowMs?: number;
  attempts?: number;
  key?: string;
}

export interface SecurityStorageCase {
  key: string;
  value: unknown;
  encrypt?: boolean;
}

export interface SecurityMonitoringTarget {
  enabled?: boolean;
  endpoint?: string;
}

export interface SecurityTestTarget {
  htmlPayloads?: string[];
  attributePayloads?: string[];
  endpoints?: string[];
  headers?: Record<string, string>;
  csp?: string;
  encryptionKey?: string;
  requireHttps?: boolean;
  csrf?: {
    invalidToken?: string;
  };
  rateLimit?: SecurityRateLimitTarget;
  secureStorageCases?: SecurityStorageCase[];
  monitoring?: SecurityMonitoringTarget;
}

export interface SecurityScenario {
  id: string;
  name: string;
  description: string;
  type: SecurityAssessmentType;
  severity: SecuritySeverity;
  control?: string;
  run(target: SecurityTestTarget): Omit<SecurityFinding, 'id' | 'title' | 'category' | 'severity' | 'control'>;
}
