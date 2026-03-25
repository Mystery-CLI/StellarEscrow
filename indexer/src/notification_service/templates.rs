use std::collections::HashMap;

/// All notification template IDs used by the service.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateId {
    TradeCreated,
    TradeFunded,
    TradeCompleted,
    TradeConfirmed,
    DisputeRaised,
    DisputeResolved,
    TradeCancelled,
}

impl TemplateId {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateId::TradeCreated    => "trade_created",
            TemplateId::TradeFunded     => "trade_funded",
            TemplateId::TradeCompleted  => "trade_completed",
            TemplateId::TradeConfirmed  => "trade_confirmed",
            TemplateId::DisputeRaised   => "dispute_raised",
            TemplateId::DisputeResolved => "dispute_resolved",
            TemplateId::TradeCancelled  => "trade_cancelled",
        }
    }

    pub fn from_event_type(event_type: &str) -> Option<Self> {
        match event_type {
            "trade_created"    => Some(TemplateId::TradeCreated),
            "trade_funded"     => Some(TemplateId::TradeFunded),
            "trade_completed"  => Some(TemplateId::TradeCompleted),
            "trade_confirmed"  => Some(TemplateId::TradeConfirmed),
            "dispute_raised"   => Some(TemplateId::DisputeRaised),
            "dispute_resolved" => Some(TemplateId::DisputeResolved),
            "trade_cancelled"  => Some(TemplateId::TradeCancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub subject: &'static str,
    pub body: &'static str, // supports {{trade_id}}, {{address}} placeholders
}

/// Render a template by substituting `{{key}}` placeholders.
pub fn render(template: &Template, vars: &HashMap<&str, String>) -> (String, String) {
    let mut subject = template.subject.to_string();
    let mut body = template.body.to_string();
    for (k, v) in vars {
        let placeholder = format!("{{{{{}}}}}", k);
        subject = subject.replace(&placeholder, v);
        body = body.replace(&placeholder, v);
    }
    (subject, body)
}

pub fn get(id: &TemplateId) -> Template {
    match id {
        TemplateId::TradeCreated => Template {
            subject: "New trade #{{trade_id}} created",
            body: "A new escrow trade #{{trade_id}} has been created. Seller: {{seller}}, Buyer: {{buyer}}, Amount: {{amount}} USDC.",
        },
        TemplateId::TradeFunded => Template {
            subject: "Trade #{{trade_id}} funded",
            body: "Trade #{{trade_id}} has been funded by the buyer. The escrow is now active.",
        },
        TemplateId::TradeCompleted => Template {
            subject: "Trade #{{trade_id}} marked complete",
            body: "The seller has marked trade #{{trade_id}} as complete. Please confirm receipt to release funds.",
        },
        TemplateId::TradeConfirmed => Template {
            subject: "Trade #{{trade_id}} settled",
            body: "Trade #{{trade_id}} has been confirmed and settled. Payout: {{payout}} USDC, Fee: {{fee}} USDC.",
        },
        TemplateId::DisputeRaised => Template {
            subject: "Dispute raised on trade #{{trade_id}}",
            body: "A dispute has been raised on trade #{{trade_id}} by {{raised_by}}. An arbitrator will review the case.",
        },
        TemplateId::DisputeResolved => Template {
            subject: "Dispute resolved on trade #{{trade_id}}",
            body: "The dispute on trade #{{trade_id}} has been resolved. Resolution: {{resolution}}. Recipient: {{recipient}}.",
        },
        TemplateId::TradeCancelled => Template {
            subject: "Trade #{{trade_id}} cancelled",
            body: "Trade #{{trade_id}} has been cancelled.",
        },
    }
}
