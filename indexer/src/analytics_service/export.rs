use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}

impl std::str::FromStr for ExportFormat {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(ExportFormat::Csv),
            _ => Ok(ExportFormat::Json),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRow {
    pub event_type: String,
    pub ledger: i64,
    pub data: serde_json::Value,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

pub fn render(rows: Vec<AnalyticsRow>, format: ExportFormat) -> String {
    match format {
        ExportFormat::Json => serde_json::to_string_pretty(&rows).unwrap_or_default(),
        ExportFormat::Csv => {
            let mut out = String::from("event_type,ledger,trade_id,amount,recorded_at\n");
            for row in &rows {
                let trade_id = row.data.get("trade_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let amount = row.data.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!(
                    "{},{},{},{},{}\n",
                    row.event_type, row.ledger, trade_id, amount, row.recorded_at
                ));
            }
            out
        }
    }
}
