use std::error::Error;

use enum_stringify::EnumStringify;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use crate::{db::ScyllaDb, state::{scylla_state::ScyllaOrder, Id, OrderId, Price, Quantity, Symbol}};






#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: Id,
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

impl Order {
    pub fn to_scylla_order(&self) -> ScyllaOrder {
        ScyllaOrder {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            filled_quote_quantity: self.filled_quote_quantity.to_string(),
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}



#[derive(Debug, Clone, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled
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



impl ScyllaDb {
    pub async fn get_order(
        &self,
        order_id: OrderId,
        symbol: &Symbol
    ) -> Result<Order, Box<dyn Error>> {
        let s = r#"
        SELECT
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
        FROM keyspace_1.order_table
        WHERE id = ? AND symbol = ?;
        "#;
        let res = self.session.query_unpaged(s, (order_id, symbol)).await?;
        let temp = res.into_rows_result().unwrap();
        let mut order = temp.rows::<ScyllaOrder>().unwrap();
        let scylla_order = order.next().transpose()?.ok_or(format!("Order not found: {}", order_id))?;
        let order = scylla_order.from_scylla_order();
        Ok(order)
    }

    pub fn update_order_statement(&self) -> &str {
        let s = r#"
            UPDATE keyspace_1.order_table
            SET
                filled_quantity = ?,
                filled_quote_quantity = ?,
                order_status = ?,
            WHERE id = ? AND symbol = ?;
        "#;
        s
    }
}