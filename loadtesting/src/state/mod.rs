use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub type Id = i64;
pub type OrderId = i64;
pub type Symbol = String;
pub type Quantity = Decimal;
pub type Price = Decimal;

// Data structures matching your backend
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderParams {
    pub price: Decimal,
    pub order_side: String,
    pub order_type: String,
    pub quantity: Decimal,
    pub user_id: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deposit {
    pub user_id: i64,
    pub asset: String,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Withdraw {
    pub user_id: i64,
    pub asset: String,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelAll {
    pub user_id: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: String,
    pub price: Decimal,
    pub order_side: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    pub user_id: i64,
    pub order_id: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    pub user_id: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOrders {
    pub user_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}

#[derive(
    Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify,
)]
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

#[derive(Debug, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled,
}
impl OrderStatus {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderStatus::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderSide {
    Bid,
    Ask,
}
impl OrderSide {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderSide::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify, EnumIter)]
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
