use std::{
    collections::HashMap,
    str::FromStr,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use enum_stringify::EnumStringify;
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use scylla::{DeserializeRow, SerializeRow, client::session::Session, statement::batch::Batch};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{PersistCancel, PersistCancelAll};

pub mod engine;
pub mod error;
pub mod limit;
pub mod orderbook;
pub mod user;

pub type Id = u64;
pub type Price = Decimal;
pub type Symbol = String;
pub type OrderId = u64;
pub type Quantity = Decimal;
pub type TradeId = u64;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub users: HashMap<Id, User>,
}

pub static USERS: Lazy<Mutex<Users>> = Lazy::new(|| {
    Mutex::new(Users {
        users: HashMap::new(),
    })
});

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq, EnumIter, EnumStringify)]
pub enum Asset {
    USDT,
    BTC,
    SOL,
    ETH,
}

impl Asset {
    pub fn from_str(asset_to_match: &str) -> Option<Self> {
        for asset in Asset::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Some(asset);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumStringify, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
}

impl OrderType {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderType::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify, EnumIter, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn from_str(side_to_match: &str) -> Result<Self, ()> {
        for side in OrderSide::iter() {
            let current_side = side.to_string();
            if side_to_match.to_string() == current_side {
                return Ok(side);
            }
        }
        Err(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify, EnumIter)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow, DeserializeRow)]
pub struct ScyllaOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: String,
    pub price: String,
    pub initial_quantity: String,
    pub filled_quantity: String,
    pub quote_quantity: String,
    pub filled_quote_quantity: String,
    pub order_type: String,
    pub order_side: String,
    pub order_status: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, DeserializeRow)]
pub struct ScyllaUser {
    pub id: i64,
    pub balance: HashMap<String, String>,
    pub locked_balance: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow, DeserializeRow)]
pub struct ScyllaCancelOrder {
    pub id: i64,
    pub user_id: i64,
    pub order_side: String,
    pub symbol: String,
    pub price: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecievedOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: Symbol,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub quote_quantity: Quantity,
    pub filled_quote_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumStringify)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled,
}

impl OrderStatus {
    pub fn from_str(status_to_match: &str) -> Result<Self, ()> {
        for status in OrderStatus::iter() {
            let current_status = status.to_string();
            if status_to_match.to_string() == current_status {
                return Ok(status);
            }
        }
        Err(())
    }
}

impl RecievedOrder {
    fn to_scylla_order(&self) -> ScyllaOrder {
        ScyllaOrder {
            id: self.id,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            filled_quote_quantity: self.filled_quote_quantity.to_string(),
            order_type: self.order_type.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            timestamp: self.timestamp,
        }
    }
}

impl ScyllaOrder {
    fn from_scylla_order(&self) -> RecievedOrder {
        RecievedOrder {
            id: self.id,
            user_id: self.user_id,
            symbol: self.symbol.clone(),
            price: Decimal::from_str(&self.price).unwrap(),
            initial_quantity: Decimal::from_str(&self.initial_quantity).unwrap(),
            filled_quantity: Decimal::from_str(&self.filled_quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            filled_quote_quantity: Decimal::from_str(&self.filled_quote_quantity).unwrap(),
            order_type: OrderType::from_str(&self.order_type).unwrap(),
            order_side: OrderSide::from_str(&self.order_side).unwrap(),
            order_status: OrderStatus::from_str(&self.order_status).unwrap(),
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: Symbol,
}

#[derive(Debug, Serialize)]
pub enum SymbolError {
    InvalidSymbol,
}

impl Exchange {
    pub fn new(base: Asset, quote: Asset) -> Exchange {
        let base_str = base.to_string();
        let quote_str = quote.to_string();
        let symbol = format!("{}_{}", base_str, quote_str);
        Exchange {
            base,
            quote,
            symbol,
        }
    }

    pub fn from_str(symbol: &str) -> Result<Exchange, SymbolError> {
        let parts: Vec<&str> = symbol.split('_').collect();
        if parts.len() != 2 {
            return Err(SymbolError::InvalidSymbol);
        }
        let base = Asset::from_str(parts[0]).ok_or(SymbolError::InvalidSymbol)?;
        let quote = Asset::from_str(parts[1]).ok_or(SymbolError::InvalidSymbol)?;
        Ok(Exchange::new(base, quote))
    }
}

#[derive(Serialize, Deserialize)]
pub struct OrderUpdate {
    order_id: u64,
    client_order_id: u64,
    trade_id: u64,
    user_id: u64,
    trade_timestamp: u128,
    order_side: OrderSide,
    order_status: OrderStatus,
    symbol: String,
    price: Decimal,
    executed_quantity: Decimal,
    executed_quote_quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    id: Id,
    quantity: Quantity,
    quote_quantity: Quantity,
    is_buyer_maker: bool,
    timestamp: u128,
    price: Price,
}

pub async fn new_order(
    session: &Session,
    order: RecievedOrder,
    locked_balance: Quantity,
    lock_asset: Asset,
) {
    let new_order = r#"
        INSERT INTO keyspace_1.order_table (
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity,
            quote_quantity,
            filled_quote_quantity,
            order_type,
            order_side,
            order_status,
            timestamp
        )  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    "#;

    let lock_balance = r#"
        UPDATE keyspace_1.user_table
        SET
            locked_balance[?] = ?
        WHERE id = ?;
    "#;
    let mut batch: Batch = Default::default();
    batch.append_statement(new_order);
    batch.append_statement(lock_balance);
    let prepared_batch: Batch = session.prepare_batch(&batch).await.unwrap();

    let order_value = order.to_scylla_order();
    let user_value = (
        lock_asset.to_string(),
        locked_balance.to_string(),
        order.user_id as i64,
    );

    session
        .batch(&prepared_batch, (order_value, user_value))
        .await
        .unwrap();
}

pub async fn persist_order_cancel_all(session: &Session, cancel_order: PersistCancelAll) {
    let new_cance_order = r#"
        INSERT INTO keyspace_1.cancel_order_table (
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
    let unlock_balance = r#"
        UPDATE keyspace_1.user_table 
        SET
            locked_balance = ?
        WHERE id = ?;
        "#;
    let update_order_status = r#"
        UPDATE keyspace_1.order_table 
        SET
            order_status = ?
        WHERE id = ? AND symbol = ?;
        "#;
    for data in cancel_order.data {
        session
            .query_unpaged(
                new_cance_order,
                (
                    data.id as i64,
                    cancel_order.user_id as i64,
                    data.order_side.to_string(),
                    cancel_order.symbol.clone(),
                    data.price.to_string(),
                    cancel_order.timestamp,
                ),
            )
            .await
            .unwrap();
        session
            .query_unpaged(
                update_order_status,
                (
                    OrderStatus::Cancelled.to_string(),
                    data.id as i64,
                    cancel_order.symbol.clone(),
                ),
            )
            .await
            .unwrap();
    }
    session
        .query_unpaged(
            unlock_balance,
            (cancel_order.locked_balances, cancel_order.user_id as i64),
        )
        .await
        .unwrap();
}

pub async fn persist_order_cancel(session: &Session, cancel_order: PersistCancel) {
    let new_cance_order = r#"
        INSERT INTO keyspace_1.cancel_order_table (
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
    let unlock_balance = r#"
        UPDATE keyspace_1.user_table 
        SET
            locked_balance[?] = ?
        WHERE id = ?;
        "#;
    let update_order_status = r#"
        UPDATE keyspace_1.order_table 
        SET
            order_status = ?
        WHERE id = ? AND symbol = ?;
        "#;
    let mut batch: Batch = Default::default();
    batch.append_statement(new_cance_order);
    batch.append_statement(unlock_balance);
    batch.append_statement(update_order_status);
    let prepared_batch: Batch = session.prepare_batch(&batch).await.unwrap();
    session
        .batch(
            &prepared_batch,
            (
                (
                    cancel_order.id as i64,
                    cancel_order.user_id as i64,
                    cancel_order.order_side.to_string(),
                    cancel_order.symbol,
                    cancel_order.price.to_string(),
                    cancel_order.timestamp,
                ),
                (
                    cancel_order.asset.to_string(),
                    cancel_order.updated_locked_balance.to_string(),
                    cancel_order.user_id as i64,
                ),
                (OrderStatus::Cancelled.to_string(), cancel_order.id as i64),
            ),
        )
        .await
        .unwrap();
}

#[derive(Debug, Serialize)]
pub struct PostUsers {
    pub user: User,   // buyer
    pub client: User, // seller
}

#[derive(Debug, Serialize)]
pub struct Filler {
    trade_id: Id,
    exchange: Exchange,
    quantity: Quantity,
    exchange_price: Price,
    is_buyer_maker: bool,
    post_users: PostUsers,
    order_status: OrderStatus,
    client_order_status: OrderStatus,
    order_id: OrderId,
    client_order_id: OrderId,
    timestamp: u128,
}

pub fn get_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn get_epoch_micro() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}
