use std::time::{SystemTime, UNIX_EPOCH};

use rust_decimal::Decimal;
pub mod redis_event;
pub mod asset;
pub mod exchange;
pub mod user;
pub mod scylla_state;
pub mod order;
pub mod trade;


pub type Symbol = String;
pub type Id = i64;
pub type OrderId = i64;
pub type Quantity = Decimal;
pub type Price = Decimal;


pub fn get_epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
