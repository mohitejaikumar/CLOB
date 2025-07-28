use rust_decimal::Decimal;
use scylla::errors::ExecutionError;
use std::{error::Error, str::FromStr};
use crate::{db::{get_epoch_micros, schema::{Price, Quantity, Symbol, Trade}, scylla_tables::ScyllaTrade, ScyllaDb}};










impl Trade {
    pub fn new(
        id: i64,
        is_buyer_maker: bool,
        price: Price,
        quantity: Quantity,
        symbol: Symbol
    ) -> Trade {
        let timestamp = get_epoch_micros();
        let quote_quantity = price * quantity;
        Trade {
            id,
            symbol,
            quantity: quantity,
            quote_quantity: quote_quantity,
            is_buyer_maker,
            price: price,
            timestamp: timestamp as i64,
        }
    }
    fn to_scylla_trade(&self) -> ScyllaTrade {
        ScyllaTrade {
            id: self.id,
            symbol: self.symbol.to_string(),
            is_buyer_maker: self.is_buyer_maker,
            price: self.price.to_string(),
            quantity: self.quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            timestamp: self.timestamp,
        }
    }
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

impl ScyllaDb {
    
    pub async fn new_trade(&self, trade: Trade) -> Result<(), ExecutionError> {
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
        let trade = trade.to_scylla_trade();
        self.session.query_unpaged(s, trade).await?;
        Ok(())
    }

    pub async fn get_trades(&self, symbol: Symbol) -> Result<Vec<Trade>, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                id,
                symbol,
                quantity,
                quote_quantity,
                is_buyer_maker,
                price,
                timestamp
            FROM keyspace_1.trade_table
            WHERE symbol = ? ALLOW FILTERING;
        "#;
        let res = self.session.query_unpaged(s, (symbol,)).await?;
        let mut temp = res.into_rows_result().unwrap();
        let mut trades = temp.rows::<ScyllaTrade>().unwrap();
        let trades: Vec<Trade> = trades.map(|trade| trade.unwrap().from_scylla_trade()).collect();
        Ok(trades)
    }

    pub async fn get_trade(&self, trade_id: i64, symbol: Symbol) -> Result<Trade, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                id,
                symbol,
                quantity,
                quote_quantity,
                is_buyer_maker,
                price,
                timestamp
            FROM keyspace_1.trade_table
            WHERE id = ? AND symbol = ?;
        "#;
        let res = self.session.query_unpaged(s, (trade_id, symbol)).await?;
        let mut temp = res.into_rows_result().unwrap();
        let mut trades = temp.rows::<ScyllaTrade>().unwrap();
        let scylla_trade = trades
            .next()
            .transpose()?
            .unwrap();
        let trade = scylla_trade.from_scylla_trade();
        Ok(trade)
    }
}