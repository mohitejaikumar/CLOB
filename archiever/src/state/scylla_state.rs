use std::collections::HashMap;
use rust_decimal::Decimal;
use scylla::{DeserializeRow, SerializeRow};
use serde::{Deserialize, Serialize};
use crate::state::{order::{Order, OrderSide, OrderStatus, OrderType}, trade::Trade, OrderId, Symbol};
use std::str::FromStr;




#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, DeserializeRow)]
pub struct ScyllaUser {
    pub id: i64,
    pub balance: HashMap<String, String>,
    pub locked_balance: HashMap<String, String>,
}


#[derive(Debug, Deserialize, Serialize, SerializeRow, DeserializeRow)]
pub struct ScyllaOrder {
    pub id: OrderId,
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

impl ScyllaOrder {
    pub fn from_scylla_order(&self) -> Order {
        Order {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quantity: Decimal::from_str(&self.filled_quantity).unwrap(),
            filled_quote_quantity: Decimal::from_str(&self.filled_quote_quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            price: Decimal::from_str(&self.price).unwrap(),
            initial_quantity: Decimal::from_str(&self.initial_quantity).unwrap(),
            order_side: OrderSide::from_str(&self.order_side).unwrap(),
            order_status: OrderStatus::from_str(&self.order_status).unwrap(),
            order_type: OrderType::from_str(&self.order_type).unwrap(),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, SerializeRow, DeserializeRow)]
pub struct ScyllaTrade {
    pub id: i64,
    pub symbol: Symbol,
    pub quantity: String,
    pub quote_quantity: String,
    pub is_buyer_maker: bool,
    pub price: String,
    pub timestamp: i64,
}

impl ScyllaTrade {
    fn from_scylla_trade(&self) -> Trade {
        Trade {
            id: self.id,
            symbol: self.symbol.to_string(),
            is_buyer_maker: self.is_buyer_maker,
            price: Decimal::from_str(&self.price).unwrap(),
            quantity: Decimal::from_str(&self.quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            timestamp: self.timestamp,
        }
    }
}


