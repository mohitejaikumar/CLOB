use serde::{Deserialize, Serialize};
use crate::{db::ScyllaDb, state::{scylla_state::ScyllaTrade, Id, Price, Quantity, Symbol}};




#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: Id,
    pub symbol: Symbol,
    pub quantity: Quantity,
    pub quote_quantity: Quantity,
    pub is_buyer_maker: bool,
    pub price: Price,
    pub timestamp: i64,
}


impl Trade {
    pub fn new(
        id: Id,
        is_buyer_maker: bool,
        price: Price,
        quantity: Quantity,
        symbol: Symbol,
        timestamp: u128
    ) -> Self {
        let quote_quantity = price * quantity;
        Self {
            id,
            symbol,
            quantity,
            quote_quantity,
            is_buyer_maker,
            price,
            timestamp: timestamp as i64,
        }
    }

    pub fn to_scylla_trade(&self) -> ScyllaTrade {
        ScyllaTrade {
            id: self.id,
            symbol: self.symbol.to_string(),
            is_buyer_maker: self.is_buyer_maker,
            price: self.price.to_string(),
            quantity: self.quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            timestamp: self.timestamp
        }
    }
}


impl ScyllaDb {
    pub fn trade_entry_statement(&self) -> &str {
        let s =
            r#"
            INSERT INTO keyspace_1.trade_table (
                id,
                symbol,
                quantity,
                quote_quantity,
                is_buyer_maker,
                price,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?);
        "#;
        s
    }
}
