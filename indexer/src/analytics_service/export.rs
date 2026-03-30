use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

fn scalar_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => v.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn csv_value(value: Option<&serde_json::Value>) -> String {
    let as_text = value.map(scalar_to_string).unwrap_or_default();
    csv_escape(&as_text)
}

pub fn render(rows: Vec<AnalyticsRow>, format: ExportFormat) -> String {
    match format {
        ExportFormat::Json => serde_json::to_string_pretty(&rows).unwrap_or_default(),
        ExportFormat::Csv => {
            let mut dynamic_keys = BTreeSet::new();
            for row in &rows {
                if let Some(map) = row.data.as_object() {
                    for key in map.keys() {
                        dynamic_keys.insert(key.clone());
                    }
                }
            }

            let mut out = String::from("event_type,ledger,recorded_at");
            for key in &dynamic_keys {
                out.push(',');
                out.push_str(key);
            }
            out.push('\n');

            for row in &rows {
                out.push_str(&csv_escape(&row.event_type));
                out.push(',');
                out.push_str(&row.ledger.to_string());
                out.push(',');
                out.push_str(&csv_escape(&row.recorded_at.to_rfc3339()));

                for key in &dynamic_keys {
                    out.push(',');
                    out.push_str(&csv_value(row.data.get(key)));
                }

                out.push('\n');
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn csv_export_includes_dynamic_columns() {
        let now = Utc::now();
        let rows = vec![
            AnalyticsRow {
                event_type: "trade_created".to_string(),
                ledger: 1,
                data: serde_json::json!({"trade_id": 42, "amount": 12345}),
                recorded_at: now,
            },
            AnalyticsRow {
                event_type: "dispute_raised".to_string(),
                ledger: 2,
                data: serde_json::json!({"trade_id": 42, "reason": "seller,offline"}),
                recorded_at: now,
            },
        ];

        let csv = render(rows, ExportFormat::Csv);
        assert!(csv.starts_with("event_type,ledger,recorded_at,amount,reason,trade_id\n"));
        assert!(csv.contains("trade_created,1"));
        assert!(csv.contains("dispute_raised,2"));
        assert!(csv.contains("\"seller,offline\""));
    }

    #[test]
    fn json_export_keeps_structure() {
        let row = AnalyticsRow {
            event_type: "trade_created".to_string(),
            ledger: 3,
            data: serde_json::json!({"trade_id": 100, "amount": 77}),
            recorded_at: Utc::now(),
        };
        let json = render(vec![row], ExportFormat::Json);
        assert!(json.contains("trade_created"));
        assert!(json.contains("trade_id"));
        assert!(json.contains("amount"));
    }
}
