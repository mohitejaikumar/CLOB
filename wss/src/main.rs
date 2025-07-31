use std::{
    sync::{Arc, Mutex},
    thread,
};

use futures_util::StreamExt;
use serde_json::from_str;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use wss::{
    broadcasters::{
        depth::handle_brodcasting_depth, order_update::handle_order_update_stream,
        ticker::handle_brodcasting_ticker, trades::handle_broadcasting_trades,
    },
    manager::UserManager,
    payload::{Event, Method, Payload},
};

fn main() {
    let addr = "127.0.0.1:9000".to_string();

    let redis_client = redis::Client::open("redis://127.0.0.1/").expect("Could not create client");
    let trade_con = redis_client
        .get_connection()
        .expect("Could not get connection");
    let ticker_con = redis_client
        .get_connection()
        .expect("Could not get connection");
    let depth_con = redis_client
        .get_connection()
        .expect("Could not get connection");
    let order_update_con = redis_client
        .get_connection()
        .expect("Could not get connection");

    let user_manager = Arc::new(Mutex::new(UserManager::new()));

    let trade_user_manager = user_manager.clone();
    let ticker_user_manager = user_manager.clone();
    let depth_user_manager = user_manager.clone();
    let order_update_user_manager = user_manager.clone();

    thread::spawn(handle_brodcasting_ticker(ticker_user_manager, ticker_con));
    thread::spawn(handle_brodcasting_depth(depth_user_manager, depth_con));
    thread::spawn(handle_order_update_stream(
        order_update_user_manager,
        order_update_con,
    ));
    thread::spawn(handle_broadcasting_trades(trade_user_manager, trade_con));

    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        // this is the guy who will accept the connection
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        println!("Server is listening on {}", addr);
        while let Ok((stream, user_addr)) = listener.accept().await {
            // now we accept all the new connection and spawn on task for each connection
            let user_manager = user_manager.clone();
            let new_ws_connection = async move {
                if let Ok(ws_stream) = handshake(stream).await {
                    // now we can handle this stream
                    handle_stream(ws_stream, user_manager.clone(), user_addr.to_string()).await;
                }
            };
            tokio::spawn(new_ws_connection);
        }
    })
}

// server level accept
pub async fn handshake(
    raw_stream: TcpStream,
) -> Result<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Error> {
    let result = accept_async(raw_stream).await;
    match result {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err),
    }
}

async fn handle_stream(
    ws_stream: WebSocketStream<TcpStream>,
    user_manager: Arc<Mutex<UserManager>>,
    user_addr: String,
) {
    let (write, mut read) = ws_stream.split();
    {
        let mut manager = user_manager.lock().unwrap();
        manager.new_user(user_addr.to_string(), write);
        println!(
            "New WebSocket connection established and user registered from: {}",
            user_addr
        );
    }

    tokio::spawn(async move {
        // handle each message
        while let Some(Ok(msg)) = read.next().await {
            match msg {
                Message::Text(text) => {
                    // handle the payloads
                    println!("text: {}", text);
                    if let Ok(payload) = from_str::<Payload>(&text) {
                        handle_payload(payload, user_addr.clone(), user_manager.clone());
                    }
                }
                Message::Close(_) => {
                    // handle the close logic
                    let mut manager = user_manager.lock().unwrap();
                    manager.remove_user(user_addr.to_string());
                    println!("WebSocket connection closed from: {}", user_addr);
                }
                _ => {}
            }
        }
    });
}

fn handle_payload(payload: Payload, user_addr: String, user_manager: Arc<Mutex<UserManager>>) {
    let mut user_manager = user_manager.lock().unwrap();
    match payload.event {
        Event::TRADE => match payload.method {
            Method::SUBSCRIBE => {
                user_manager.subscribe_trade(user_addr, payload.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager.unsubscribe_trades(user_addr, payload.symbol);
            }
        },
        Event::TICKER => match payload.method {
            Method::SUBSCRIBE => {
                user_manager.subscribe_ticker(user_addr, payload.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager.unsubscribe_ticker(user_addr, payload.symbol);
            }
        },
        Event::DEPTH => match payload.method {
            Method::SUBSCRIBE => {
                user_manager.subscribe_depth(user_addr, payload.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager.unsubscribe_depth(user_addr, payload.symbol);
            }
        },
        Event::ORDER_UPDATE => match payload.method {
            Method::SUBSCRIBE => {
                user_manager.assign_user_id(user_addr, payload.user_id);
            }
            Method::UNSUBSCRIBE => {
                user_manager.dissociate_user_id(user_addr);
            }
        },
    }
}

/*

TcpListerner bind
listner.accept
accept_async -> server level accept
websocketstream split -> (read, write)


// two part to socket stream -> read half  + write half
read half (SplitStream) -> impl Stream
write half (SplitSink) -> impl Sink

*/
