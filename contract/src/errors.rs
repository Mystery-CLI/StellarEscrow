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
    MetadataTooManyEntries = 11,
    MetadataValueTooLong = 12,
    InvalidTierConfig = 13,
    TierNotFound = 14,
    TemplateNotFound = 15,
    TemplateInactive = 16,
    TemplateNameTooLong = 17,
    TemplateVersionLimitExceeded = 18,
    TemplateAmountMismatch = 19,
    /// buyer_bps in a Partial resolution must be 0–10000
    InvalidSplitBps = 20,
}