use enum_stringify::EnumStringify;
use serde::{Deserialize, Serialize};



#[derive(Clone, Debug, Serialize, Deserialize, EnumStringify)]
pub enum MatchingEngineErrors {
    ExchangeAlreadyExist,
    AskedMoreThanTradeable,
    UserNotFound,
    OverWithdrawl,
    InsufficientBalance,
    InvalidOrderId,
    InvalidPriceLimitOrOrderSide
}

