use super::{ComplianceCheck, ComplianceStatus};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub period_from: DateTime<Utc>,
    pub period_to: DateTime<Utc>,
    pub total_checks: usize,
    pub approved: usize,
    pub rejected: usize,
    pub blocked: usize,
    pub requires_review: usize,
    pub pending: usize,
    pub avg_risk_score: f64,
    pub high_risk_count: usize,
    /// Breakdown by jurisdiction
    pub by_jurisdiction: HashMap<String, usize>,
    /// Top AML exposure categories
    pub top_exposure_categories: Vec<(String, usize)>,
    /// Sanctions matches count
    pub sanctions_matches: usize,
}

pub fn build_report(
    checks: Vec<ComplianceCheck>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> ComplianceReport {
    let total = checks.len();
    let mut approved = 0usize;
    let mut rejected = 0usize;
    let mut blocked = 0usize;
    let mut requires_review = 0usize;
    let mut pending = 0usize;
    let mut risk_sum = 0u64;
    let mut high_risk = 0usize;
    let mut by_jurisdiction: HashMap<String, usize> = HashMap::new();
    let mut exposure_counts: HashMap<String, usize> = HashMap::new();
    let mut sanctions_matches = 0usize;

    for check in &checks {
        match check.status {
            ComplianceStatus::Approved => approved += 1,
            ComplianceStatus::Rejected => rejected += 1,
            ComplianceStatus::Blocked => blocked += 1,
            ComplianceStatus::RequiresReview => requires_review += 1,
            ComplianceStatus::Pending => pending += 1,
        }

        risk_sum += check.risk_score as u64;
        if check.risk_score >= 70 {
            high_risk += 1;
        }

        if let Some(ref jur) = check.kyc_result.jurisdiction {
            *by_jurisdiction.entry(jur.clone()).or_insert(0) += 1;
        }

        for cat in &check.aml_result.exposure_categories {
            *exposure_counts.entry(cat.clone()).or_insert(0) += 1;
        }

        sanctions_matches += check.aml_result.sanctions_matches.len();
    }

    let avg_risk_score = if total > 0 {
        risk_sum as f64 / total as f64
    } else {
        0.0
    };

    let mut top_exposure: Vec<(String, usize)> = exposure_counts.into_iter().collect();
    top_exposure.sort_by(|a, b| b.1.cmp(&a.1));
    top_exposure.truncate(10);

    ComplianceReport {
        generated_at: Utc::now(),
        period_from: from,
        period_to: to,
        total_checks: total,
        approved,
        rejected,
        blocked,
        requires_review,
        pending,
        avg_risk_score,
        high_risk_count: high_risk,
        by_jurisdiction,
        top_exposure_categories: top_exposure,
        sanctions_matches,
    }
}
