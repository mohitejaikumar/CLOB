use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum_macros::EnumIter;

use crate::enums::registered_symbols::RegisteredSymbols;







#[derive(Deserialize)]
pub struct Payload {
    pub user_id: Option<u64>,
    pub method: Method,
    pub event: Event,
    pub symbol: RegisteredSymbols,
}
#[derive(Deserialize, PartialEq, Eq, Hash, Clone, EnumIter, EnumStringify)]
pub enum Event {
    ORDER_UPDATE,
    TRADE,
    TICKER,
    DEPTH,
}
#[derive(Deserialize)]
pub enum Method {
    SUBSCRIBE,
    UNSUBSCRIBE,
}