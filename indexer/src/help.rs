use axum::{extract::Query, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HelpArticle {
    pub id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub content: &'static str,
    pub tags: &'static [&'static str],
}

#[derive(Serialize)]
pub struct FaqItem {
    pub id: &'static str,
    pub question: &'static str,
    pub answer: &'static str,
    pub category: &'static str,
}

#[derive(Serialize)]
pub struct TutorialStep {
    pub step: u8,
    pub title: &'static str,
    pub description: &'static str,
    pub code_example: Option<&'static str>,
}

#[derive(Serialize)]
pub struct Tutorial {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub difficulty: &'static str,
    pub steps: &'static [TutorialStep],
}

#[derive(Serialize)]
pub struct ContactInfo {
    pub channel: &'static str,
    pub description: &'static str,
    pub url: &'static str,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

// ── Static content ────────────────────────────────────────────────────────────

static FAQS: &[FaqItem] = &[
    FaqItem {
        id: "faq-1",
        question: "What is StellarEscrow?",
        answer: "StellarEscrow is a decentralized escrow system on the Stellar blockchain. It enables secure peer-to-peer USDC trades with optional arbitration for dispute resolution.",
        category: "general",
    },
    FaqItem {
        id: "faq-2",
        question: "What token is used for trades?",
        answer: "All trades use USDC (USD Coin) stablecoin on the Stellar network. The contract address is configured at initialization.",
        category: "general",
    },
    FaqItem {
        id: "faq-3",
        question: "What are the trade lifecycle states?",
        answer: "A trade moves through these states: Created → Funded → Completed → Confirmed. It can also transition to Disputed or Cancelled depending on the parties' actions.",
        category: "trades",
    },
    FaqItem {
        id: "faq-4",
        question: "How does dispute resolution work?",
        answer: "Either party can raise a dispute on a funded trade. A registered arbitrator then reviews the case and resolves it by releasing funds to either the buyer or seller.",
        category: "disputes",
    },
    FaqItem {
        id: "faq-5",
        question: "What is the platform fee?",
        answer: "The platform fee is expressed in basis points (bps). For example, 100 bps = 1%. The fee is deducted from the trade amount when funds are released. Maximum allowed is 10000 bps (100%).",
        category: "fees",
    },
    FaqItem {
        id: "faq-6",
        question: "Can a trade be cancelled?",
        answer: "Yes. The seller can cancel a trade that has not yet been funded (status: Created). Once funded, cancellation requires dispute resolution.",
        category: "trades",
    },
    FaqItem {
        id: "faq-7",
        question: "How do I query events via the API?",
        answer: "Use GET /events with optional query params: event_type, trade_id, from_ledger, to_ledger, limit, offset. See the API documentation endpoint for full details.",
        category: "api",
    },
    FaqItem {
        id: "faq-8",
        question: "How does WebSocket streaming work?",
        answer: "Connect to the /ws endpoint. You will receive real-time JSON messages for every contract event as they are indexed from the Stellar network.",
        category: "api",
    },
    FaqItem {
        id: "faq-9",
        question: "Who can register arbitrators?",
        answer: "Only the contract admin can register or remove arbitrators via register_arbitrator and remove_arbitrator_fn contract functions.",
        category: "admin",
    },
    FaqItem {
        id: "faq-10",
        question: "How do I replay historical events?",
        answer: "POST to /events/replay with a JSON body containing from_ledger and optionally to_ledger. This replays stored events and broadcasts them to WebSocket clients.",
        category: "api",
    },
];

static TUTORIALS: &[Tutorial] = &[
    Tutorial {
        id: "tutorial-create-trade",
        title: "Creating Your First Trade",
        description: "Learn how to create an escrow trade as a seller on StellarEscrow.",
        difficulty: "beginner",
        steps: &[
            TutorialStep {
                step: 1,
                title: "Ensure the contract is initialized",
                description: "The contract must be initialized with an admin, USDC token address, and fee in basis points before any trades can be created.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- initialize --admin <ADMIN_ADDR> --usdc_token <USDC_ADDR> --fee_bps 100"),
            },
            TutorialStep {
                step: 2,
                title: "Create the trade",
                description: "As the seller, invoke create_trade with the buyer address and the USDC amount. This returns a trade ID.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- create_trade --seller <SELLER_ADDR> --buyer <BUYER_ADDR> --amount 1000000"),
            },
            TutorialStep {
                step: 3,
                title: "Buyer funds the escrow",
                description: "The buyer calls fund_trade with the trade ID. USDC is transferred from the buyer to the contract.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- fund_trade --buyer <BUYER_ADDR> --trade_id <TRADE_ID>"),
            },
            TutorialStep {
                step: 4,
                title: "Seller marks trade complete",
                description: "Once the seller has delivered, they call complete_trade to signal delivery.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- complete_trade --seller <SELLER_ADDR> --trade_id <TRADE_ID>"),
            },
            TutorialStep {
                step: 5,
                title: "Buyer confirms and releases funds",
                description: "The buyer confirms receipt by calling confirm_trade. Funds minus platform fee are released to the seller.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- confirm_trade --buyer <BUYER_ADDR> --trade_id <TRADE_ID>"),
            },
        ],
    },
    Tutorial {
        id: "tutorial-dispute",
        title: "Raising and Resolving a Dispute",
        description: "Understand how to handle disputes between buyers and sellers.",
        difficulty: "intermediate",
        steps: &[
            TutorialStep {
                step: 1,
                title: "Raise a dispute",
                description: "Either the buyer or seller can raise a dispute on a funded trade by calling raise_dispute.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- raise_dispute --caller <CALLER_ADDR> --trade_id <TRADE_ID>"),
            },
            TutorialStep {
                step: 2,
                title: "Arbitrator reviews the case",
                description: "A registered arbitrator reviews the evidence off-chain and decides the outcome.",
                code_example: None,
            },
            TutorialStep {
                step: 3,
                title: "Arbitrator resolves the dispute",
                description: "The arbitrator calls resolve_dispute specifying whether funds go to the buyer or seller.",
                code_example: Some("stellar contract invoke --id <CONTRACT_ID> -- resolve_dispute --arbitrator <ARB_ADDR> --trade_id <TRADE_ID> --resolution ReleaseToBuyer"),
            },
        ],
    },
    Tutorial {
        id: "tutorial-indexer",
        title: "Setting Up the Event Indexer",
        description: "Run the indexer service to monitor and query contract events.",
        difficulty: "intermediate",
        steps: &[
            TutorialStep {
                step: 1,
                title: "Configure the indexer",
                description: "Copy config.toml to config.local.toml and set your database URL, contract ID, and Stellar Horizon endpoint.",
                code_example: Some("cp config.toml config.local.toml"),
            },
            TutorialStep {
                step: 2,
                title: "Set up the database",
                description: "Create a PostgreSQL database and run migrations.",
                code_example: Some("createdb stellar_escrow\nsqlx migrate run"),
            },
            TutorialStep {
                step: 3,
                title: "Start the indexer",
                description: "Run the indexer service. It will begin polling Stellar for contract events.",
                code_example: Some("cargo run -- --config config.local.toml"),
            },
            TutorialStep {
                step: 4,
                title: "Query events via REST",
                description: "Use the REST API to query indexed events.",
                code_example: Some("curl http://localhost:3000/events?event_type=trade_created&limit=10"),
            },
            TutorialStep {
                step: 5,
                title: "Stream events via WebSocket",
                description: "Connect to the WebSocket endpoint for real-time event streaming.",
                code_example: Some("wscat -c ws://localhost:3000/ws"),
            },
        ],
    },
];

static DOCS: &[HelpArticle] = &[
    HelpArticle {
        id: "doc-contract-overview",
        category: "contract",
        title: "Smart Contract Overview",
        content: "The StellarEscrow smart contract is a Soroban contract written in Rust. It manages the full lifecycle of peer-to-peer escrow trades using USDC on the Stellar blockchain. Key functions: initialize, create_trade, fund_trade, complete_trade, confirm_trade, raise_dispute, resolve_dispute, cancel_trade, register_arbitrator, remove_arbitrator_fn, update_fee, withdraw_fees.",
        tags: &["contract", "soroban", "overview"],
    },
    HelpArticle {
        id: "doc-trade-states",
        category: "contract",
        title: "Trade State Machine",
        content: "Trades follow a strict state machine: Created (initial state after create_trade) → Funded (after fund_trade by buyer) → Completed (after complete_trade by seller) → Confirmed (terminal, funds released to seller). Alternative paths: Created → Cancelled (seller cancels unfunded trade), Funded → Disputed (either party raises dispute), Disputed → Resolved (arbitrator resolves).",
        tags: &["trades", "state", "lifecycle"],
    },
    HelpArticle {
        id: "doc-fees",
        category: "contract",
        title: "Fee Structure",
        content: "Platform fees are set in basis points (bps) at initialization and can be updated by the admin. 1 bps = 0.01%, so 100 bps = 1%. Fees are deducted from the trade amount at settlement and accumulated in the contract. The admin can withdraw accumulated fees at any time via withdraw_fees.",
        tags: &["fees", "bps", "admin"],
    },
    HelpArticle {
        id: "doc-api-reference",
        category: "api",
        title: "REST API Reference",
        content: "GET /health — service health check. GET /events — list events (params: event_type, trade_id, from_ledger, to_ledger, limit, offset). GET /events/:id — get event by UUID. GET /events/trade/:trade_id — events for a specific trade. GET /events/type/:event_type — events by type. POST /events/replay — replay events in ledger range (body: {from_ledger, to_ledger?}). GET /ws — WebSocket upgrade for real-time streaming. GET /help — this help center.",
        tags: &["api", "rest", "endpoints"],
    },
    HelpArticle {
        id: "doc-event-types",
        category: "api",
        title: "Contract Event Types",
        content: "The indexer tracks these event types emitted by the contract: trade_created, trade_funded, trade_completed, trade_confirmed, dispute_raised, dispute_resolved, trade_cancelled, arbitrator_registered, arbitrator_removed, fee_updated, fees_withdrawn, compliance_passed, compliance_failed.",
        tags: &["events", "indexer", "types"],
    },
    HelpArticle {
        id: "doc-security",
        category: "security",
        title: "Security Model",
        content: "All privileged operations require Stellar authorization (require_auth). Admin-only operations: register_arbitrator, remove_arbitrator_fn, update_fee, withdraw_fees. Seller-only: create_trade, complete_trade, cancel_trade. Buyer-only: fund_trade, confirm_trade. Arbitrator-only: resolve_dispute. The contract validates initialization state before every operation.",
        tags: &["security", "auth", "roles"],
    },
];

static CONTACT: &[ContactInfo] = &[
    ContactInfo {
        channel: "GitHub Issues",
        description: "Report bugs or request features on the project repository.",
        url: "https://github.com/your-org/stellar-escrow/issues",
    },
    ContactInfo {
        channel: "GitHub Discussions",
        description: "Ask questions and discuss the project with the community.",
        url: "https://github.com/your-org/stellar-escrow/discussions",
    },
    ContactInfo {
        channel: "Stellar Discord",
        description: "Join the Stellar developer community for broader ecosystem support.",
        url: "https://discord.gg/stellardev",
    },
];

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /help — help center index with all available sections
pub async fn help_index() -> Json<serde_json::Value> {
    Json(json!({
        "title": "StellarEscrow Help Center",
        "description": "Documentation, tutorials, FAQs, and support for the StellarEscrow platform.",
        "sections": {
            "faqs":      "/help/faqs",
            "tutorials": "/help/tutorials",
            "docs":      "/help/docs",
            "search":    "/help/search?q=<query>",
            "contact":   "/help/contact",
        }
    }))
}

/// GET /help/faqs?category=<optional>
pub async fn get_faqs(Query(params): Query<SearchQuery>) -> Json<serde_json::Value> {
    let items: Vec<&FaqItem> = FAQS
        .iter()
        .filter(|f| {
            params.category.as_deref().map_or(true, |c| f.category == c)
        })
        .collect();

    let categories: std::collections::HashSet<&str> = FAQS.iter().map(|f| f.category).collect();

    Json(json!({
        "total": items.len(),
        "categories": categories,
        "items": items,
    }))
}

/// GET /help/tutorials/:id  or  GET /help/tutorials
pub async fn get_tutorials() -> Json<serde_json::Value> {
    let summaries: Vec<serde_json::Value> = TUTORIALS
        .iter()
        .map(|t| json!({
            "id": t.id,
            "title": t.title,
            "description": t.description,
            "difficulty": t.difficulty,
            "step_count": t.steps.len(),
            "url": format!("/help/tutorials/{}", t.id),
        }))
        .collect();

    Json(json!({ "total": summaries.len(), "tutorials": summaries }))
}

/// GET /help/tutorials/:id
pub async fn get_tutorial_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    TUTORIALS
        .iter()
        .find(|t| t.id == id)
        .map(|t| Json(json!(t)))
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

/// GET /help/docs?category=<optional>
pub async fn get_docs(Query(params): Query<SearchQuery>) -> Json<serde_json::Value> {
    let articles: Vec<&&HelpArticle> = DOCS
        .iter()
        .filter(|a| {
            params.category.as_deref().map_or(true, |c| a.category == c)
        })
        .collect::<Vec<_>>()
        .iter()
        .collect();

    let categories: std::collections::HashSet<&str> = DOCS.iter().map(|a| a.category).collect();

    Json(json!({
        "total": articles.len(),
        "categories": categories,
        "articles": articles,
    }))
}

/// GET /help/search?q=<query>&category=<optional>
/// Searches across FAQs, docs, and tutorial titles/descriptions
pub async fn search_help(Query(params): Query<SearchQuery>) -> Json<serde_json::Value> {
    let query = params.q.as_deref().unwrap_or("").to_lowercase();

    if query.is_empty() {
        return Json(json!({ "error": "query parameter 'q' is required" }));
    }

    let matched_faqs: Vec<&FaqItem> = FAQS
        .iter()
        .filter(|f| {
            let cat_ok = params.category.as_deref().map_or(true, |c| f.category == c);
            let text_ok = f.question.to_lowercase().contains(&query)
                || f.answer.to_lowercase().contains(&query);
            cat_ok && text_ok
        })
        .collect();

    let matched_docs: Vec<&HelpArticle> = DOCS
        .iter()
        .filter(|a| {
            let cat_ok = params.category.as_deref().map_or(true, |c| a.category == c);
            let text_ok = a.title.to_lowercase().contains(&query)
                || a.content.to_lowercase().contains(&query)
                || a.tags.iter().any(|t| t.to_lowercase().contains(&query));
            cat_ok && text_ok
        })
        .collect();

    let matched_tutorials: Vec<serde_json::Value> = TUTORIALS
        .iter()
        .filter(|t| {
            t.title.to_lowercase().contains(&query)
                || t.description.to_lowercase().contains(&query)
                || t.steps.iter().any(|s| {
                    s.title.to_lowercase().contains(&query)
                        || s.description.to_lowercase().contains(&query)
                })
        })
        .map(|t| json!({
            "id": t.id,
            "title": t.title,
            "description": t.description,
            "difficulty": t.difficulty,
            "url": format!("/help/tutorials/{}", t.id),
        }))
        .collect();

    Json(json!({
        "query": query,
        "results": {
            "faqs":      { "count": matched_faqs.len(),      "items": matched_faqs },
            "docs":      { "count": matched_docs.len(),      "items": matched_docs },
            "tutorials": { "count": matched_tutorials.len(), "items": matched_tutorials },
        }
    }))
}

/// GET /help/contact
pub async fn get_contact() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Need help? Reach us through any of the following channels.",
        "channels": CONTACT,
    }))
}
