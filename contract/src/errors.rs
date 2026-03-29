use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Core (1–10)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidFeeBps = 4,
    Overflow = 5,
    Unauthorized = 6,
    ContractPaused = 7,
    InvalidStatus = 8,
    TradeNotFound = 9,
    ArbitratorNotRegistered = 10,

    // Compliance (11–15)
    KycNotVerified = 11,
    AmlNotCleared = 12,
    JurisdictionRestricted = 13,
    TradeAmountLimitExceeded = 14,
    ComplianceDataMissing = 15,

    // Fees / Tiers (16–20)
    NoFeesToWithdraw = 16,
    InvalidTierConfig = 17,
    TierNotFound = 18,
    InvalidMetadata = 19,
    MetadataValueTooLong = 20,

    // Templates (21–26)
    TemplateNotFound = 21,
    TemplateInactive = 22,
    TemplateNameTooLong = 23,
    TemplateVersionLimitExceeded = 24,
    TemplateAmountMismatch = 25,
    InvalidExpiry = 26,

    // Arbitrator reputation (27–30)
    InvalidRating = 27,
    AlreadyRated = 28,
    NoArbitrator = 29,
    TradeExpired = 30,
    TradeNotExpired = 31,
    InvalidSplitBps = 32,

    // Subscriptions (33–36)
    SubscriptionNotFound = 33,
    SubscriptionExpired = 34,
    SubscriptionAlreadyActive = 35,

    // Governance (36–43)
    ProposalNotFound = 36,
    ProposalNotActive = 37,
    AlreadyVoted = 38,
    InsufficientVotingPower = 39,
    ProposalNotPassed = 40,
    ProposalAlreadyExecuted = 41,
    VotingEnded = 42,

    // Privacy (43–45)
    PrivacyDataTooLong = 43,
    DisclosureGrantNotFound = 44,
    DisclosureUnauthorized = 45,

    // Migration / Bridge (46–51)
    MigrationAlreadyApplied = 46,
    MigrationVersionMismatch = 47,
    BridgeOracleNotSet = 48,
    BridgeTradeExpired = 49,
    BridgeTradeNotExpired = 50,

    // Insurance (51–55)
    InsuranceProviderNotRegistered = 51,
    InsurancePremiumTooHigh = 52,
    TradeNotInsured = 53,
    InsuranceAlreadyClaimed = 54,
    InsuranceClaimNotEligible = 55,

    // Oracle (60–64)
    OracleNotFound = 60,
    OracleAlreadyRegistered = 61,
    OracleListFull = 62,
    OracleUnavailable = 63,
    OraclePriceInvalid = 64,

    // AMM (70–74)
    AmmPoolNotFound = 70,
    AmmSlippageExceeded = 71,
    AmmInsufficientShares = 72,
    AmmInvalidPair = 73,
    AmmPoolAlreadyExists = 74,

    // Upgrade (80–85)
    UpgradeInProgress = 80,
    NoUpgradeProposal = 81,
    UpgradeTimelockActive = 82,
    NoUpgradeInProgress = 83,
    RollbackWindowExpired = 84,

    // Multi-sig arbitration (90–94)
    InvalidMultiSigConfig = 90,
    VotingExpired = 91,
    VotingNotExpired = 92,
    NoConsensus = 93,

    // Social (95–96)
    CannotFollowSelf = 95,
    NotFollowing = 96,
}
    KycNotVerified = 5,
    AmlNotCleared = 6,
    JurisdictionRestricted = 7,
    TradeAmountLimitExceeded = 8,
    ArbitratorNotRegistered = 9,
    TradeNotFound = 10,
    InvalidStatus = 7,
    Overflow = 8,
    NoFeesToWithdraw = 9,
    Unauthorized = 10,
    ContractPaused = 11,
    InvalidMetadata = 11,
    MetadataValueTooLong = 12,
    InvalidTierConfig = 13,
    TierNotFound = 14,
    TemplateNotFound = 15,
    TemplateInactive = 16,
    TemplateNameTooLong = 17,
    TemplateVersionLimitExceeded = 18,
    TemplateAmountMismatch = 19,
    /// Star rating must be 1–5
    InvalidRating = 20,
    /// Caller already rated this arbitrator for this trade
    AlreadyRated = 21,
    /// Trade has no arbitrator to rate
    NoArbitrator = 22,
    /// buyer_bps in a Partial resolution must be 0–10000
    InvalidSplitBps = 20,
    InvalidExpiry = 20,
    TradeExpired = 21,
    TradeNotExpired = 22,
    // Metadata errors (duplicates removed)
    InvalidTierConfig = 14,
    TierNotFound = 15,
    TemplateNotFound = 16,
    TemplateInactive = 17,
    TemplateNameTooLong = 18,
    TemplateVersionLimitExceeded = 19,
    TemplateAmountMismatch = 20,
    SubscriptionNotFound = 21,
    SubscriptionExpired = 22,
    SubscriptionAlreadyActive = 23,
    ProposalNotFound = 24,
    ProposalNotActive = 25,
    AlreadyVoted = 26,
    InsufficientVotingPower = 27,
    ProposalNotPassed = 28,
    ProposalAlreadyExecuted = 29,
    VotingEnded = 30,
    PrivacyDataTooLong = 31,
    DisclosureGrantNotFound = 32,
    DisclosureUnauthorized = 33,
    MigrationAlreadyApplied = 21,
    MigrationVersionMismatch = 22,
    BridgeOracleNotSet = 23,
    BridgeTradeExpired = 24,
    BridgeTradeNotExpired = 25,
    InsuranceProviderNotRegistered = 26,
    InsurancePremiumTooHigh = 27,
    TradeNotInsured = 28,
    InsuranceAlreadyClaimed = 29,
    InsuranceClaimNotEligible = 30,
    // Oracle errors (40–44)
    OracleNotFound = 40,
    OracleAlreadyRegistered = 41,
    OracleListFull = 42,
    OracleUnavailable = 43,
    OraclePriceInvalid = 44,
    // AMM errors (50–54)
    AmmPoolNotFound = 50,
    // Bridge errors (80–99)
    BridgeProviderNotFound = 80,
    BridgeProviderAlreadyRegistered = 81,
    BridgeProviderLimitExceeded = 82,
    BridgeTradeNotFound = 83,
    BridgeTradeExpired = 84,
    BridgeTradeNotExpired = 85,
    BridgeRetryLimitExceeded = 86,
    BridgeAttestationInvalid = 87,
    BridgeAttestationExpired = 88,
    BridgeAmountOutOfRange = 89,
    BridgeChainNotSupported = 90,
    BridgeOracleNotAuthorized = 91,
    BridgePaused = 92,
    BridgeSignatureInvalid = 93,
    BridgeNonceAlreadyUsed = 94,
    AmmSlippageExceeded = 51,
    AmmInsufficientShares = 52,
    AmmInvalidPair = 53,
    AmmPoolAlreadyExists = 54,
    // Upgrade system errors (60–66)
    /// An upgrade is already in progress (guard is set).
    UpgradeInProgress = 60,
    /// No upgrade proposal exists to execute or cancel.
    NoUpgradeProposal = 61,
    /// Timelock has not yet expired; upgrade cannot be executed yet.
    UpgradeTimelockActive = 62,
    /// No upgrade guard is set; migrate/rollback called out of sequence.
    NoUpgradeInProgress = 63,
    /// Rollback window has passed; state cannot be reverted automatically.
    RollbackWindowExpired = 64,
    // Multi-sig arbitration errors (70–74)
    /// threshold == 0 or threshold > arbitrators count.
    InvalidMultiSigConfig = 70,
    /// Arbitrator has already cast a vote for this trade.
    AlreadyVoted = 71,
    /// Voting window has expired; no more votes accepted.
    VotingExpired = 72,
    /// Voting window has not yet expired; cannot force-resolve.
    VotingNotExpired = 73,
    /// No consensus reached among arbitrators.
    NoConsensus = 74,
    // Social feature errors (70-74)
    CannotFollowSelf = 70,
    NotFollowing = 71,
}
