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
}
