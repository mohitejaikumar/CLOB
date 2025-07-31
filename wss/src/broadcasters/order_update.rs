use std::sync::{Arc, Mutex};

use crate::{
    enums::registered_symbols::RegisteredSymbols, manager::UserManager, runtime::TOKIO_RUNTIME,
};
use redis::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use strum::IntoEnumIterator;

pub fn handle_order_update_stream(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection,
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("order_update:{}", symbol)) {
                println!(
                    "Could not subscribe to  order_update pubsub symbol :{}, {}",
                    symbol, err
                );
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(order_update_string) = msg.get_payload::<String>() {
                    println!("order_update_string: {}", order_update_string);
                    if let Ok(order_update) = from_str::<OrderUpdate>(&order_update_string) {
                        TOKIO_RUNTIME.block_on(
                            manager
                                .lock()
                                .unwrap()
                                .send_order_update(order_update.user_id, &order_update_string),
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    order_id: i64,
    client_order_id: i64,
    trade_id: u64,
    user_id: u64,
    trade_timestamp: u128,
    order_side: String,
    order_status: String,
    symbol: String,
    price: Decimal,
    executed_quantity: Decimal,
    executed_quote_quantity: Decimal,
}
