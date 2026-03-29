# Requirements Document

## Introduction

The Admin Dashboard provides a secure, web-based interface for platform operators to manage the StellarEscrow escrow platform. It covers administrator authentication, fee and tier configuration, arbitrator management, platform analytics visualization, and system-level configuration. The dashboard interacts with the on-chain smart contract (fee settings, tier config, arbitrator reputation) and the off-chain API layer.

## Glossary

- **Admin_Dashboard**: The web application providing the administrative interface described in this document.
- **Admin**: An authenticated platform operator with elevated privileges to manage platform settings.
- **Auth_Service**: The component responsible for verifying admin credentials and issuing session tokens.
- **Fee_Manager**: The component responsible for reading and updating platform fee configuration on-chain.
- **Tier_Manager**: The component responsible for reading and updating the Bronze/Silver/Gold fee tier configuration.
- **Arbitrator_Manager**: The component responsible for listing, registering, and managing arbitrators and their reputation data.
- **Analytics_Service**: The component responsible for fetching and presenting platform metrics and time-windowed statistics.
- **Config_Manager**: The component responsible for reading and updating system-level configuration parameters.
- **Session**: An authenticated admin session identified by a signed token with a defined expiry.
- **Fee_BPS**: Platform fee expressed in basis points (1 bps = 0.01%). Valid range: 0–10000.
- **Tier_Config**: The set of fee rates (in bps) applied to Bronze, Silver, and Gold volume tiers.
- **Arbitrator**: An on-chain address registered to resolve trade disputes.
- **Reputation**: An arbitrator's aggregate performance record including dispute counts, resolution breakdown, and star rating.
- **PlatformMetrics**: Aggregate on-chain statistics including trade counts, volume, fees collected, success rate, and dispute rate.
- **TimeWindow**: A rolling period (Last24h, Last7d, Last30d, AllTime) over which windowed metrics are computed.

---

## Requirements

### Requirement 1: Admin Authentication

**User Story:** As an admin, I want to authenticate with secure credentials, so that only authorized operators can access the dashboard.

#### Acceptance Criteria

1. WHEN an admin submits valid credentials, THE Auth_Service SHALL issue a signed session token with an expiry of no more than 8 hours.
2. WHEN an admin submits invalid credentials, THE Auth_Service SHALL return an error response within 2 seconds and SHALL NOT issue a session token.
3. WHEN an admin submits invalid credentials 5 consecutive times within 15 minutes, THE Auth_Service SHALL lock the account for 30 minutes and SHALL notify the admin via a displayed message.
4. WHILE a session token is valid, THE Admin_Dashboard SHALL grant access to protected routes.
5. WHEN a session token expires, THE Admin_Dashboard SHALL redirect the admin to the login page and SHALL invalidate the session.
6. WHEN an admin explicitly logs out, THE Auth_Service SHALL invalidate the session token immediately.
7. THE Auth_Service SHALL store admin passwords using a cryptographic hashing algorithm with a per-user salt.
8. WHEN an admin accesses a protected route without a valid session token, THE Admin_Dashboard SHALL redirect the admin to the login page.

---

### Requirement 2: Fee Management Interface

**User Story:** As an admin, I want to view and update the platform fee and tier fee configuration, so that I can control the cost structure for platform users.

#### Acceptance Criteria

1. WHEN an authenticated admin opens the fee management page, THE Fee_Manager SHALL display the current base platform fee in basis points.
2. WHEN an authenticated admin submits a new base fee value, THE Fee_Manager SHALL validate that the value is an integer in the range 0–10000 inclusive.
3. IF the submitted base fee value is outside the range 0–10000, THEN THE Fee_Manager SHALL display a validation error and SHALL NOT submit the change to the contract.
4. WHEN an authenticated admin submits a valid base fee value, THE Fee_Manager SHALL apply the change on-chain and SHALL display a confirmation message within 10 seconds.
5. WHEN an authenticated admin opens the tier configuration section, THE Tier_Manager SHALL display the current Bronze, Silver, and Gold fee rates in basis points.
6. WHEN an authenticated admin submits updated tier fee rates, THE Tier_Manager SHALL validate that each rate is in the range 0–10000 and that Gold ≤ Silver ≤ Bronze.
7. IF the submitted tier fee rates violate the ordering constraint (Gold ≤ Silver ≤ Bronze), THEN THE Tier_Manager SHALL display a descriptive validation error and SHALL NOT submit the change.
8. WHEN an authenticated admin submits valid tier fee rates, THE Tier_Manager SHALL apply the tier configuration on-chain and SHALL display a confirmation message within 10 seconds.
9. WHEN an authenticated admin sets a custom fee for a specific user address, THE Fee_Manager SHALL validate that the address is a valid Stellar address and the fee is in the range 0–10000.
10. IF the user address or custom fee value is invalid, THEN THE Fee_Manager SHALL display a descriptive validation error and SHALL NOT submit the change.
11. WHEN an authenticated admin submits a valid custom fee for a user address, THE Fee_Manager SHALL apply the custom fee on-chain and SHALL display a confirmation message within 10 seconds.
12. THE Fee_Manager SHALL display a history of the last 50 fee configuration changes, including the timestamp, changed value, and the admin who made the change.

---

### Requirement 3: Arbitrator Management

**User Story:** As an admin, I want to view and manage arbitrators and their performance data, so that I can maintain a high-quality dispute resolution pool.

#### Acceptance Criteria

1. WHEN an authenticated admin opens the arbitrator management page, THE Arbitrator_Manager SHALL display a list of all registered arbitrators including their Stellar address and reputation summary.
2. THE Arbitrator_Manager SHALL display for each arbitrator: total disputes assigned, resolved count, buyer-win count, seller-win count, average star rating, and rating count.
3. WHEN an authenticated admin searches for an arbitrator by Stellar address, THE Arbitrator_Manager SHALL return matching results within 2 seconds.
4. WHEN an authenticated admin submits a new arbitrator address for registration, THE Arbitrator_Manager SHALL validate that the address is a valid Stellar address.
5. IF the submitted arbitrator address is not a valid Stellar address, THEN THE Arbitrator_Manager SHALL display a validation error and SHALL NOT register the address.
6. WHEN an authenticated admin submits a valid arbitrator address, THE Arbitrator_Manager SHALL register the arbitrator and SHALL display a confirmation message within 10 seconds.
7. WHEN an authenticated admin selects an arbitrator and requests removal, THE Arbitrator_Manager SHALL display a confirmation prompt before proceeding.
8. WHEN an authenticated admin confirms arbitrator removal, THE Arbitrator_Manager SHALL deregister the arbitrator and SHALL display a confirmation message within 10 seconds.
9. WHEN an authenticated admin views an arbitrator's detail page, THE Arbitrator_Manager SHALL display the full reputation record including a breakdown of partial resolutions.
10. THE Arbitrator_Manager SHALL allow an authenticated admin to filter the arbitrator list by minimum average rating, minimum resolved count, or both simultaneously.

---

### Requirement 4: Platform Analytics

**User Story:** As an admin, I want to view platform-wide metrics and time-windowed statistics, so that I can monitor platform health and usage trends.

#### Acceptance Criteria

1. WHEN an authenticated admin opens the analytics page, THE Analytics_Service SHALL display all-time PlatformMetrics including total volume, trade counts by status, total fees collected, success rate, and dispute rate.
2. WHEN an authenticated admin selects a time window (Last24h, Last7d, Last30d, or AllTime), THE Analytics_Service SHALL display windowed metrics for that period within 3 seconds.
3. THE Analytics_Service SHALL display the count of unique buyer and seller addresses that have ever interacted with the platform.
4. THE Analytics_Service SHALL present success rate and dispute rate as percentages derived from the on-chain basis-point values.
5. WHEN an authenticated admin views the analytics page, THE Analytics_Service SHALL display the number of currently active trades (created + funded minus terminal states).
6. THE Analytics_Service SHALL display per-arbitrator metrics including disputes resolved, resolution breakdown (buyer/seller/partial), accessible from the analytics page.
7. WHEN an authenticated admin requests a data export, THE Analytics_Service SHALL generate a CSV file containing the currently displayed metrics and SHALL make it available for download within 5 seconds.
8. THE Analytics_Service SHALL refresh displayed metrics automatically every 60 seconds while the analytics page is open.

---

### Requirement 5: System Configuration

**User Story:** As an admin, I want to manage system-level configuration parameters, so that I can control platform behavior without redeploying the contract.

#### Acceptance Criteria

1. WHEN an authenticated admin opens the system configuration page, THE Config_Manager SHALL display all current configurable parameters including subscription tier prices and discount rates.
2. WHEN an authenticated admin submits an updated subscription tier price, THE Config_Manager SHALL validate that the value is a positive integer expressed in USDC micro-units (stroops).
3. IF the submitted subscription price is zero or negative, THEN THE Config_Manager SHALL display a validation error and SHALL NOT apply the change.
4. WHEN an authenticated admin submits a valid subscription tier price, THE Config_Manager SHALL apply the change and SHALL display a confirmation message within 10 seconds.
5. WHEN an authenticated admin submits an updated subscription discount rate, THE Config_Manager SHALL validate that the value is in the range 0–10000 basis points.
6. IF the submitted discount rate is outside the range 0–10000, THEN THE Config_Manager SHALL display a validation error and SHALL NOT apply the change.
7. WHEN an authenticated admin submits a valid discount rate, THE Config_Manager SHALL apply the change and SHALL display a confirmation message within 10 seconds.
8. THE Config_Manager SHALL display a read-only view of governance proposal parameters (voting period, proposal threshold, quorum) for reference.
9. THE Config_Manager SHALL maintain an audit log of all configuration changes, recording the parameter name, previous value, new value, timestamp, and admin identity.
10. WHEN an authenticated admin views the audit log, THE Config_Manager SHALL display the 100 most recent configuration change entries in reverse chronological order.
