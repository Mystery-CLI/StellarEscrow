use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidFeeBps = 4,
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