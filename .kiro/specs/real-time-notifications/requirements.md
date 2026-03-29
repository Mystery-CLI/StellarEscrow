# Requirements Document

## Introduction

This feature adds real-time notifications to the Stellar escrow platform so that traders, arbitrators, and subscribers receive timely alerts when on-chain events occur. The platform already emits structured Soroban contract events (trade lifecycle, disputes, arbitration, subscriptions, governance) that are versioned and categorised. The notification system will consume those events via a WebSocket gateway, fan them out to connected clients, and optionally deliver them through browser push and email channels. Users will be able to configure which event categories and delivery channels they want per their preferences.

## Glossary

- **Notification_Service**: The backend service responsible for ingesting on-chain events, evaluating user preferences, and dispatching notifications to all configured delivery channels.
- **WebSocket_Gateway**: The server-side component that maintains persistent WebSocket connections with browser clients and pushes notification payloads in real time.
- **Notification_Client**: The browser-side library (used by the SvelteKit `client` and React `app` frontends) that manages the WebSocket connection and exposes a notification store.
- **Notification_Component**: A UI component that renders the notification bell, badge count, and notification list panel.
- **Preference_Store**: The persistent storage (per user address) that records which event categories and delivery channels a user has opted into.
- **Push_Provider**: The Web Push / VAPID service used to deliver browser push notifications to users who are not actively connected.
- **Email_Provider**: The transactional email service used to deliver email notifications.
- **Event**: A structured on-chain Soroban event emitted by the escrow contract (see `contract/src/events.rs`), identified by a category symbol and event name symbol.
- **Trade_Event**: An Event in the `trade` category, covering trade lifecycle transitions (created, funded, completed, confirmed, cancelled, disputed, resolved, partial_resolved, time_released, meta_updated).
- **Dispute_Event**: A Trade_Event specifically for dispute lifecycle (dispute raised, dispute resolved, partial resolved).
- **System_Event**: An Event in the `sys` category, covering contract pause/unpause, emergency withdrawal, upgrade, and migration.
- **User_Address**: A Stellar account address that identifies a platform participant (buyer, seller, arbitrator, subscriber).
- **Notification**: A structured message delivered to a User_Address containing event type, trade ID (where applicable), human-readable summary, timestamp, and severity level.
- **Severity**: A classification of notification urgency — `info`, `warning`, or `critical`.
- **Delivery_Channel**: One of three supported delivery mechanisms: `websocket`, `push`, or `email`.

---

## Requirements

### Requirement 1: WebSocket Connection Management

**User Story:** As a trader, I want a persistent real-time connection to the platform so that I receive notifications instantly without polling.

#### Acceptance Criteria

1. WHEN a User_Address authenticates in the browser client, THE Notification_Client SHALL establish a WebSocket connection to the WebSocket_Gateway within 3 seconds.
2. WHEN the WebSocket connection is closed unexpectedly, THE Notification_Client SHALL attempt to reconnect using exponential backoff, with an initial delay of 1 second, doubling on each attempt up to a maximum delay of 30 seconds.
3. WHILE a WebSocket connection is active, THE WebSocket_Gateway SHALL send a heartbeat ping to the Notification_Client every 30 seconds.
4. IF the Notification_Client does not receive a heartbeat ping within 60 seconds, THEN THE Notification_Client SHALL close the current connection and initiate a reconnection attempt.
5. WHEN a User_Address disconnects and reconnects within 5 minutes, THE WebSocket_Gateway SHALL deliver any Notifications that were queued during the disconnection period, up to a maximum of 50 queued Notifications per User_Address.
6. THE WebSocket_Gateway SHALL support a minimum of 1,000 concurrent WebSocket connections.

---

### Requirement 2: On-Chain Event Ingestion

**User Story:** As a platform operator, I want the Notification_Service to reliably consume all relevant on-chain events so that no trade or dispute event is missed.

#### Acceptance Criteria

1. THE Notification_Service SHALL subscribe to the Soroban RPC event stream and ingest all Events emitted by the escrow contract.
2. WHEN an Event with category `trade` is received, THE Notification_Service SHALL parse the event payload according to the schema version indicated by the `v` field and produce a Notification.
3. WHEN an Event with category `arb` is received and the event involves a registered arbitrator, THE Notification_Service SHALL produce a Notification addressed to that arbitrator's User_Address.
4. WHEN an Event with category `sys` of type `paused` or `emrg_wd` is received, THE Notification_Service SHALL produce a Notification with Severity `critical` addressed to all active User_Addresses.
5. IF an Event payload contains an unrecognised schema version, THEN THE Notification_Service SHALL log the raw event and emit a `warning` severity Notification to the platform operator address without dropping the event.
6. THE Notification_Service SHALL process each ingested Event and produce the corresponding Notification within 2 seconds of the event being confirmed on-chain.
7. THE Notification_Service SHALL deduplicate Events using the event's ledger sequence and contract event index, so that each unique Event produces at most one Notification per User_Address.

---

### Requirement 3: Trade Status Change Notifications

**User Story:** As a buyer or seller, I want to be notified when the status of my trade changes so that I can take timely action.

#### Acceptance Criteria

1. WHEN a `trade.created` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `info` addressed to both the seller and buyer User_Addresses identified in the event payload.
2. WHEN a `trade.funded` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `info` addressed to the seller User_Address of the corresponding trade.
3. WHEN a `trade.complete` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `info` addressed to both the seller and buyer User_Addresses of the corresponding trade.
4. WHEN a `trade.cancel` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `warning` addressed to both the seller and buyer User_Addresses of the corresponding trade.
5. WHEN a `trade.time_rel` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `warning` addressed to the buyer User_Address of the corresponding trade, indicating that funds have been auto-released to the seller.
6. THE Notification_Service SHALL include the trade ID, previous status, new status, and a human-readable summary in every trade status change Notification.

---

### Requirement 4: Dispute Notifications

**User Story:** As a trader or arbitrator, I want immediate notification of dispute events so that I can respond before any deadlines expire.

#### Acceptance Criteria

1. WHEN a `trade.dispute` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `critical` addressed to the buyer, seller, and assigned arbitrator User_Addresses of the corresponding trade.
2. WHEN a `trade.resolved` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `info` addressed to the buyer, seller, and assigned arbitrator User_Addresses of the corresponding trade, including the resolution outcome and recipient address.
3. WHEN a `trade.part_res` Event is ingested, THE Notification_Service SHALL produce a Notification with Severity `info` addressed to the buyer and seller User_Addresses, including the buyer amount, seller amount, and fee.
4. WHEN a `trade.dispute` Event is ingested and the trade uses a `MultiSig` arbitration configuration, THE Notification_Service SHALL produce a Notification addressed to each arbitrator in the multi-sig panel.
5. IF a dispute voting period expires without consensus, THEN THE Notification_Service SHALL produce a Notification with Severity `critical` addressed to all arbitrators in the multi-sig panel and to the platform operator address.

---

### Requirement 5: Notification Components

**User Story:** As a user, I want a clear and accessible notification UI so that I can view and manage my notifications without leaving the current page.

#### Acceptance Criteria

1. THE Notification_Component SHALL display a bell icon with a badge showing the count of unread Notifications, updating in real time as new Notifications arrive via the Notification_Client.
2. WHEN the unread Notification count exceeds 99, THE Notification_Component SHALL display "99+" in the badge.
3. WHEN a user activates the bell icon, THE Notification_Component SHALL display a panel listing the 20 most recent Notifications, ordered by timestamp descending.
4. WHEN a user selects a Notification in the panel, THE Notification_Component SHALL mark that Notification as read and, where a trade ID is present, navigate the user to the corresponding trade detail page.
5. WHEN a user activates a "Mark all as read" control, THE Notification_Component SHALL mark all Notifications for the current User_Address as read.
6. THE Notification_Component SHALL render each Notification with a visual indicator corresponding to its Severity (`info`, `warning`, `critical`).
7. THE Notification_Component SHALL be keyboard-navigable and meet ARIA landmark and live-region requirements so that screen readers announce incoming Notifications.

---

### Requirement 6: Notification Preferences

**User Story:** As a user, I want to control which notifications I receive and how they are delivered so that I am not overwhelmed by irrelevant alerts.

#### Acceptance Criteria

1. THE Preference_Store SHALL persist notification preferences per User_Address, including the set of enabled event categories and the set of enabled Delivery_Channels.
2. WHEN a user saves notification preferences, THE Notification_Service SHALL apply the updated preferences to all subsequent Notifications for that User_Address within 5 seconds.
3. THE Notification_Service SHALL support the following configurable event categories: `trade_lifecycle`, `disputes`, `arbitration`, `subscriptions`, `governance`, `system`.
4. WHEN a Notification is produced for a User_Address and the corresponding event category is disabled in that user's Preference_Store, THE Notification_Service SHALL not deliver that Notification to any Delivery_Channel for that User_Address.
5. WHERE a user has enabled the `email` Delivery_Channel, THE Notification_Service SHALL respect a configurable minimum interval between email Notifications per User_Address of no less than 1 minute, to prevent email flooding.
6. THE Preference_Store SHALL provide default preferences for new User_Addresses that enable `trade_lifecycle` and `disputes` categories on the `websocket` channel only.

---

### Requirement 7: Browser Push Notifications

**User Story:** As a user, I want to receive push notifications when I am not actively using the platform so that I do not miss critical events.

#### Acceptance Criteria

1. WHEN a user grants browser push permission, THE Notification_Client SHALL register a push subscription with the Push_Provider using VAPID authentication and store the subscription endpoint in the Preference_Store.
2. WHEN a Notification with Severity `critical` or `warning` is produced for a User_Address that has `push` enabled and has no active WebSocket connection, THE Notification_Service SHALL dispatch the Notification to the Push_Provider within 5 seconds.
3. WHEN a Notification is dispatched via the Push_Provider, THE Notification_Service SHALL include the notification title, body, Severity, and a deep-link URL to the relevant trade or event page.
4. IF the Push_Provider returns a subscription-expired or subscription-not-found error, THEN THE Notification_Service SHALL remove the stale push subscription from the Preference_Store and log the removal.
5. WHERE a user has enabled `push` for the `trade_lifecycle` category, THE Notification_Service SHALL also deliver `info` Severity Notifications via push when no active WebSocket connection exists.

---

### Requirement 8: Email Notifications

**User Story:** As a user, I want to receive email summaries of important events so that I have a persistent record outside the platform.

#### Acceptance Criteria

1. WHEN a Notification with Severity `critical` is produced for a User_Address that has `email` enabled and a registered email address, THE Notification_Service SHALL dispatch an email via the Email_Provider within 60 seconds.
2. THE Notification_Service SHALL use a transactional email template that includes the event type, trade ID (where applicable), human-readable summary, Severity, timestamp, and a link to the relevant page.
3. WHEN a user updates their registered email address, THE Notification_Service SHALL send a confirmation email to the new address before using it for Notification delivery.
4. IF the Email_Provider returns a permanent delivery failure for a User_Address, THEN THE Notification_Service SHALL disable the `email` Delivery_Channel for that User_Address and produce a `warning` Severity in-app Notification informing the user.
5. THE Notification_Service SHALL batch `info` Severity email Notifications for a User_Address into a digest delivered at most once per hour, rather than sending individual emails for each `info` event.
6. THE Notification_Service SHALL include an unsubscribe link in every email that, when followed, disables the `email` Delivery_Channel for the User_Address in the Preference_Store.
