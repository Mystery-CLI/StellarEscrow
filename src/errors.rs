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
    BatchLimitExceeded = 12,
    EmptyBatch = 13,
    MetadataTooManyEntries = 14,
    MetadataValueTooLong = 15,
    InvalidTierConfig = 16,
    TierNotFound = 17,
    TemplateNotFound = 18,
    TemplateInactive = 19,
    TemplateNameTooLong = 20,
    TemplateVersionLimitExceeded = 21,
    TemplateAmountMismatch = 22,
    InvalidLedgerRange = 23,
    PresetNotFound = 24,
    PresetNameTooLong = 25,
    TooManyPresets = 26,
    InsufficientAllowance = 27,
    TradeExpired = 28,
    NotExpiredYet = 29,
    DuplicateTradeInBatch = 30,
}
