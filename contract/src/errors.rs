use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidFeeBps = 4,
    ArbitratorNotRegistered = 5,
    TradeNotFound = 6,
    InvalidStatus = 7,
    Overflow = 8,
    NoFeesToWithdraw = 9,
    Unauthorized = 10,
    ContractPaused = 11,
    MetadataTooManyEntries = 12,
    MetadataValueTooLong = 13,
    InvalidTierConfig = 14,
    TierNotFound = 15,
    TemplateNotFound = 16,
    TemplateInactive = 17,
    TemplateNameTooLong = 18,
    TemplateVersionLimitExceeded = 19,
    TemplateAmountMismatch = 20,
    MigrationAlreadyApplied = 21,
    MigrationVersionMismatch = 22,
    BridgeOracleNotSet = 23,
    BridgeTradeExpired = 24,
    BridgeTradeNotExpired = 25,
}