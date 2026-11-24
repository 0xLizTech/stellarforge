use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum YieldError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    ContractPaused = 4,
    InvalidAmount = 5,
    DistributionNotFound = 6,
    AlreadyClaimed = 7,
    SnapshotLedgerNotClosed = 8,
    ZeroSharesAtSnapshot = 9,
    Overflow = 10,
}
