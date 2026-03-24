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
    BatchLimitExceeded = 11,
    EmptyBatch = 12,
    ContractPaused = 13,
    MetadataTooManyEntries = 13,
    MetadataValueTooLong = 14,
    InvalidTierConfig = 15,
    TierNotFound = 16,
    TemplateNotFound = 17,
    TemplateInactive = 18,
    TemplateNameTooLong = 19,
    TemplateVersionLimitExceeded = 20,
    TemplateAmountMismatch = 21,
}
