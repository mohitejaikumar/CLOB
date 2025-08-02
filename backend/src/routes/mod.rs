use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::db::schema::{Asset, Id, Order, OrderId, OrderSide, Quantity, Symbol};

pub mod orders;
pub mod ping;
pub mod request_output;
pub mod trades;
pub mod user;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserId {
    pub user_id: Id,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineRequests {
    ExecuteOrder(Order),
    CancelOrder(CancelOrder),
    CancelAll(CancelAll),
    OpenOrders(OpenOrders),
    OpenOrder(OpenOrder),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrder {
    id: OrderId,
    #[serde(skip_deserializing)]
    user_id: Id,
    symbol: Symbol,
    price: Decimal,
    order_side: OrderSide,
    #[serde(skip_deserializing)]
    sub_id: i64,
    #[serde(skip_deserializing)]
    timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelAll {
    #[serde(skip_deserializing)]
    user_id: Id,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
    #[serde(skip_deserializing)]
    timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    #[serde(skip_deserializing)]
    user_id: Id,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    #[serde(skip_deserializing)]
    user_id: Id,
    order_id: OrderId,
    symbol: Symbol,
    #[serde(skip_deserializing)]
    sub_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequests {
    NewUser(NewUser),
    Deposit(Deposit),
    Withdraw(Withdraw),
    GetUserBalances(GetUserBalances),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Deposit {
    #[serde(skip_deserializing)]
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    #[serde(skip_deserializing)]
    sub_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Withdraw {
    #[serde(skip_deserializing)]
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalances {
    #[serde(skip_deserializing)]
    user_id: Id,
    #[serde(skip_deserializing)]
    sub_id: i64,
}
