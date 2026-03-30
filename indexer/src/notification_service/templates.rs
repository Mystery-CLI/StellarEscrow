use std::collections::HashMap;

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
            TemplateId::TradeCreated => "trade_created",
            TemplateId::TradeFunded => "trade_funded",
            TemplateId::TradeCompleted => "trade_completed",
            TemplateId::TradeConfirmed => "trade_confirmed",
            TemplateId::DisputeRaised => "dispute_raised",
            TemplateId::DisputeResolved => "dispute_resolved",
            TemplateId::TradeCancelled => "trade_cancelled",
        }
    }

    pub fn from_event_type(event_type: &str) -> Option<Self> {
        match event_type {
            "trade_created" => Some(TemplateId::TradeCreated),
            "trade_funded" => Some(TemplateId::TradeFunded),
            "trade_completed" => Some(TemplateId::TradeCompleted),
            "trade_confirmed" => Some(TemplateId::TradeConfirmed),
            "dispute_raised" => Some(TemplateId::DisputeRaised),
            "dispute_resolved" => Some(TemplateId::DisputeResolved),
            "trade_cancelled" => Some(TemplateId::TradeCancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    Sms,
    Push,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationChannel::Email => "email",
            NotificationChannel::Sms => "sms",
            NotificationChannel::Push => "push",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub email_subject: &'static str,
    pub email_body: &'static str,
    pub sms_body: &'static str,
    pub push_title: &'static str,
    pub push_body: &'static str,
}

#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub subject: String,
    pub body: String,
}

pub fn render(
    template: &Template,
    channel: NotificationChannel,
    vars: &HashMap<&str, String>,
) -> RenderedTemplate {
    let (subject, body) = match channel {
        NotificationChannel::Email => (template.email_subject, template.email_body),
        NotificationChannel::Sms => ("", template.sms_body),
        NotificationChannel::Push => (template.push_title, template.push_body),
    };

    RenderedTemplate {
        subject: interpolate(subject, vars),
        body: interpolate(body, vars),
    }
}

fn interpolate(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in vars {
        rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
    }
    rendered
}

pub fn get(id: &TemplateId) -> Template {
    match id {
        TemplateId::TradeCreated => Template {
            email_subject: "New trade #{{trade_id}} created",
            email_body: "A new escrow trade #{{trade_id}} has been created.\nSeller: {{seller}}\nBuyer: {{buyer}}\nAmount: {{amount}} USDC.",
            sms_body: "Trade #{{trade_id}} created for {{amount}} USDC.",
            push_title: "Trade #{{trade_id}} created",
            push_body: "{{amount}} USDC escrow created between {{seller}} and {{buyer}}.",
        },
        TemplateId::TradeFunded => Template {
            email_subject: "Trade #{{trade_id}} funded",
            email_body: "Trade #{{trade_id}} has been funded by the buyer. The escrow is now active.",
            sms_body: "Trade #{{trade_id}} is now funded.",
            push_title: "Trade funded",
            push_body: "Trade #{{trade_id}} is now funded.",
        },
        TemplateId::TradeCompleted => Template {
            email_subject: "Trade #{{trade_id}} marked complete",
            email_body: "The seller has marked trade #{{trade_id}} as complete. Please confirm receipt to release funds.",
            sms_body: "Trade #{{trade_id}} marked complete. Confirm receipt to release funds.",
            push_title: "Trade marked complete",
            push_body: "Trade #{{trade_id}} was marked complete.",
        },
        TemplateId::TradeConfirmed => Template {
            email_subject: "Trade #{{trade_id}} settled",
            email_body: "Trade #{{trade_id}} has been confirmed and settled.\nPayout: {{payout}} USDC\nFee: {{fee}} USDC.",
            sms_body: "Trade #{{trade_id}} settled. Payout {{payout}} USDC.",
            push_title: "Trade settled",
            push_body: "Trade #{{trade_id}} settled successfully.",
        },
        TemplateId::DisputeRaised => Template {
            email_subject: "Dispute raised on trade #{{trade_id}}",
            email_body: "A dispute has been raised on trade #{{trade_id}} by {{raised_by}}. An arbitrator will review the case.",
            sms_body: "Dispute raised on trade #{{trade_id}}.",
            push_title: "Dispute raised",
            push_body: "Trade #{{trade_id}} is now under dispute.",
        },
        TemplateId::DisputeResolved => Template {
            email_subject: "Dispute resolved on trade #{{trade_id}}",
            email_body: "The dispute on trade #{{trade_id}} has been resolved.\nResolution: {{resolution}}\nRecipient: {{recipient}}.",
            sms_body: "Dispute resolved on trade #{{trade_id}}.",
            push_title: "Dispute resolved",
            push_body: "Trade #{{trade_id}} dispute has been resolved.",
        },
        TemplateId::TradeCancelled => Template {
            email_subject: "Trade #{{trade_id}} cancelled",
            email_body: "Trade #{{trade_id}} has been cancelled.",
            sms_body: "Trade #{{trade_id}} was cancelled.",
            push_title: "Trade cancelled",
            push_body: "Trade #{{trade_id}} was cancelled.",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{get, render, NotificationChannel, TemplateId};
    use std::collections::HashMap;

    #[test]
    fn renders_channel_specific_content() {
        let template = get(&TemplateId::TradeConfirmed);
        let mut vars = HashMap::new();
        vars.insert("trade_id", "42".to_string());
        vars.insert("payout", "190".to_string());
        vars.insert("fee", "10".to_string());

        let email = render(&template, NotificationChannel::Email, &vars);
        let sms = render(&template, NotificationChannel::Sms, &vars);
        let push = render(&template, NotificationChannel::Push, &vars);

        assert_eq!(email.subject, "Trade #42 settled");
        assert!(email.body.contains("Payout: 190 USDC"));
        assert!(sms.subject.is_empty());
        assert_eq!(sms.body, "Trade #42 settled. Payout 190 USDC.");
        assert_eq!(push.subject, "Trade settled");
        assert_eq!(push.body, "Trade #42 settled successfully.");
    }
}
