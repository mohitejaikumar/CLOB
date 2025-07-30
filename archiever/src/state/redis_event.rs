use serde::{Deserialize, Serialize};

use crate::state::{exchange::Exchange, order::OrderStatus, user::PostUsers, Id, OrderId, Price, Quantity};




#[derive(Debug, Deserialize, Serialize)]
pub struct RedisEvent{
    pub trade_id: Id,
    pub order_id: OrderId,
    pub client_order_id: OrderId, // buyer
    pub is_buyer_maker: bool,
    pub exchange_price: Price,
    pub post_users: PostUsers,
    pub order_status: OrderStatus,
    pub client_order_status: OrderStatus,
    pub quantity: Quantity,
    pub timestamp: u128,
    pub exchange: Exchange
}