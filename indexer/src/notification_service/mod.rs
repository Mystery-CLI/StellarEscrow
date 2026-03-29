use std::collections::HashMap;
use std::sync::Arc;

use crate::config::NotificationConfig;
use crate::database::Database;
use crate::models::{Event, NotificationPreferences};

mod channels;
pub mod templates;

use templates::{NotificationChannel, TemplateId};

pub struct NotificationService {
    db: Arc<Database>,
    cfg: NotificationConfig,
}

impl NotificationService {
    pub fn new(db: Arc<Database>, cfg: NotificationConfig) -> Self {
        Self { db, cfg }
    }

    /// Called by the event monitor for every new contract event.
    pub async fn process_event(&self, event: &Event) {
        let Some(template_id) = TemplateId::from_event_type(&event.event_type) else {
            return;
        };

        let vars = event_vars(&event.data);
        let addresses = trade_addresses(&event.data);
        if addresses.is_empty() {
            return;
        }

        let template = templates::get(&template_id);

        for address in addresses {
            let prefs = match self.db.get_notification_preferences(&address).await {
                Ok(Some(prefs)) => prefs,
                Ok(None) => NotificationPreferences::default_for_address(address.clone()),
                Err(_) => continue,
            };

            if !prefs_allow(&prefs, &template_id) {
                continue;
            }

            if prefs.email_enabled {
                if let Some(email) = non_empty(prefs.email_address.as_deref()) {
                    let rendered = templates::render(&template, NotificationChannel::Email, &vars);
                    let result =
                        channels::send_email(&self.cfg, email, &rendered.subject, &rendered.body)
                            .await;
                    self.db
                        .log_notification(
                            &address,
                            NotificationChannel::Email.as_str(),
                            template_id.as_str(),
                            Some(&rendered.subject),
                            &rendered.body,
                            result,
                        )
                        .await;
                }
            }

            if prefs.sms_enabled {
                if let Some(phone) = non_empty(prefs.phone_number.as_deref()) {
                    let rendered = templates::render(&template, NotificationChannel::Sms, &vars);
                    let result = channels::send_sms(&self.cfg, phone, &rendered.body).await;
                    self.db
                        .log_notification(
                            &address,
                            NotificationChannel::Sms.as_str(),
                            template_id.as_str(),
                            None,
                            &rendered.body,
                            result,
                        )
                        .await;
                }
            }

            if prefs.push_enabled {
                if let Some(token) = non_empty(prefs.push_token.as_deref()) {
                    let rendered = templates::render(&template, NotificationChannel::Push, &vars);
                    let result =
                        channels::send_push(&self.cfg, token, &rendered.subject, &rendered.body)
                            .await;
                    self.db
                        .log_notification(
                            &address,
                            NotificationChannel::Push.as_str(),
                            template_id.as_str(),
                            Some(&rendered.subject),
                            &rendered.body,
                            result,
                        )
                        .await;
                }
            }
        }
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn event_vars(data: &serde_json::Value) -> HashMap<&'static str, String> {
    let mut vars = HashMap::new();
    let keys = [
        "trade_id",
        "seller",
        "buyer",
        "amount",
        "payout",
        "fee",
        "raised_by",
        "resolution",
        "recipient",
    ];

    for key in keys {
        let value = data
            .get(key)
            .map(|value| match value {
                serde_json::Value::Null => String::new(),
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        vars.insert(key, value);
    }

    vars
}

fn trade_addresses(data: &serde_json::Value) -> Vec<String> {
    let mut addrs = Vec::new();
    for key in ["seller", "buyer", "raised_by", "recipient"] {
        if let Some(value) = data.get(key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() && !addrs.iter().any(|existing| existing == trimmed) {
                addrs.push(trimmed.to_string());
            }
        }
    }
    addrs
}

fn prefs_allow(prefs: &NotificationPreferences, id: &TemplateId) -> bool {
    match id {
        TemplateId::TradeCreated => prefs.on_trade_created,
        TemplateId::TradeFunded => prefs.on_trade_funded,
        TemplateId::TradeCompleted => prefs.on_trade_completed,
        TemplateId::TradeConfirmed => prefs.on_trade_confirmed,
        TemplateId::DisputeRaised => prefs.on_dispute_raised,
        TemplateId::DisputeResolved => prefs.on_dispute_resolved,
        TemplateId::TradeCancelled => prefs.on_trade_cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::{event_vars, prefs_allow, trade_addresses};
    use crate::models::NotificationPreferences;
    use crate::notification_service::templates::TemplateId;
    use serde_json::json;

    #[test]
    fn extracts_unique_addresses_from_event_payload() {
        let data = json!({
            "seller": "S1",
            "buyer": "B1",
            "raised_by": "S1",
            "recipient": "R1"
        });

        let addresses = trade_addresses(&data);

        assert_eq!(addresses, vec!["S1", "B1", "R1"]);
    }

    #[test]
    fn fills_missing_template_values_with_unknown() {
        let vars = event_vars(&json!({ "trade_id": 44 }));

        assert_eq!(vars.get("trade_id").map(String::as_str), Some("44"));
        assert_eq!(vars.get("buyer").map(String::as_str), Some("unknown"));
        assert_eq!(vars.get("amount").map(String::as_str), Some("unknown"));
    }

    #[test]
    fn respects_event_level_preferences() {
        let mut prefs = NotificationPreferences::default_for_address("GABC");
        prefs.on_dispute_raised = false;

        assert!(prefs_allow(&prefs, &TemplateId::TradeCreated));
        assert!(!prefs_allow(&prefs, &TemplateId::DisputeRaised));
    }
}
