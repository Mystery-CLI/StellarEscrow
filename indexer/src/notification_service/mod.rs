use std::collections::HashMap;
use std::sync::Arc;

use tracing::warn;

use crate::config::NotificationConfig;
use crate::database::Database;
use crate::models::Event;

mod channels;
pub mod templates;

use templates::TemplateId;

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

        // Build template variables from event data
        let vars = event_vars(&event.data);

        // Fetch addresses that are parties to this trade
        let addresses = trade_addresses(&event.data);

        for address in addresses {
            let Ok(Some(prefs)) = self.db.get_notification_preferences(&address).await else {
                continue;
            };

            // Check per-event toggle
            if !prefs_allow(&prefs, &template_id) {
                continue;
            }

            let tmpl = templates::get(&template_id);
            let (subject, body) = templates::render(&tmpl, &vars);

            if prefs.email_enabled {
                if let Some(ref email) = prefs.email_address {
                    let result = channels::send_email(&self.cfg, email, &subject, &body).await;
                    self.db
                        .log_notification(
                            &address,
                            "email",
                            template_id.as_str(),
                            Some(&subject),
                            &body,
                            result,
                        )
                        .await;
                }
            }

            if prefs.sms_enabled {
                if let Some(ref phone) = prefs.phone_number {
                    let result = channels::send_sms(&self.cfg, phone, &body).await;
                    self.db
                        .log_notification(
                            &address,
                            "sms",
                            template_id.as_str(),
                            None,
                            &body,
                            result,
                        )
                        .await;
                }
            }

            if prefs.push_enabled {
                if let Some(ref token) = prefs.push_token {
                    let result = channels::send_push(&self.cfg, token, &subject, &body).await;
                    self.db
                        .log_notification(
                            &address,
                            "push",
                            template_id.as_str(),
                            Some(&subject),
                            &body,
                            result,
                        )
                        .await;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn event_vars(data: &serde_json::Value) -> HashMap<&'static str, String> {
    let mut m = HashMap::new();
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
    for k in keys {
        if let Some(v) = data.get(k) {
            m.insert(
                k,
                v.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string()),
            );
        }
    }
    m
}

fn trade_addresses(data: &serde_json::Value) -> Vec<String> {
    let mut addrs = Vec::new();
    for key in ["seller", "buyer", "raised_by", "recipient"] {
        if let Some(v) = data.get(key).and_then(|v| v.as_str()) {
            if !v.is_empty() && !addrs.contains(&v.to_string()) {
                addrs.push(v.to_string());
            }
        }
    }
    addrs
}

fn prefs_allow(prefs: &crate::models::NotificationPreferences, id: &TemplateId) -> bool {
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
