use std::thread;
use engine::{matching::{engine::MatchingEngine, Exchange, RegisteredSymbols}, process_order, process_user_request, TOKIO_RUNTIME};
use scylla::client::session_builder::SessionBuilder;
use strum::IntoEnumIterator;




fn main() {
    // build the connection with the scylla db on same thread
    let session = TOKIO_RUNTIME.block_on(
        SessionBuilder::new().known_node("127.0.0.1:9042").build()
    ).unwrap();
    
    // matching engine init
    let mut matching_engine = MatchingEngine::init();

    // block and recover orderbooks on restart
    TOKIO_RUNTIME.block_on(matching_engine.recover_all_orderbooks(&session));
    
    // running each orderbooks parallelly
    RegisteredSymbols::iter().for_each(|symbol|{
        let exchange = Exchange::from_str(&symbol.to_string()).unwrap();
        let orderbook = matching_engine.orderbook.get_mut(&exchange).unwrap();
        // spawn thread for each orderbook
        thread::spawn(process_order(orderbook.clone())); 
    });

    // process user requests
    thread::spawn(process_user_request());
}
