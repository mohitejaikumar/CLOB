use std::sync::{Arc, Mutex};
use redis::Connection;
use strum::IntoEnumIterator;
use crate::{enums::registered_symbols::RegisteredSymbols, manager::UserManager, runtime::TOKIO_RUNTIME};


pub fn handle_brodcasting_ticker(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("ticker:{}", symbol)) {
                println!("Could not subscribe to ticker pubsub symbol :{}, {}", symbol, err);
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(ticker) = msg.get_payload::<String>() {
                    let mut manager = manager.lock().unwrap();
                    let symbol_str = msg.get_channel_name().split(":").last().unwrap();
                    let symbol = RegisteredSymbols::from_str(symbol_str).unwrap();
                    TOKIO_RUNTIME.block_on(manager.brodcast_ticker(symbol, ticker));
                }
            }
        }
    }
}