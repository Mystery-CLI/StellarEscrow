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
