

use std::sync::{Arc, Mutex};

use serde_json::from_str;
use tokio::{net::{TcpListener, TcpStream}, runtime::Runtime};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use futures_util::{ StreamExt};
use wss::{manager::UserManager, payload::{Event, Method, Payload}};






fn main() {
    let addr = "127.0.0.1:9000".to_string();
    let runtime = Runtime::new().unwrap();
    let user_manager = Arc::new(Mutex::new(UserManager::new()));
    runtime.block_on( async move {
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
) ->  Result<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Error> {
    let result = accept_async(raw_stream).await;
    match result {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err)
    }
}


async fn handle_stream(
    ws_stream: WebSocketStream<TcpStream>,
    user_manager: Arc<Mutex<UserManager>>,
    user_addr: String
) {
    let (write, mut read) = ws_stream.split();
    {
        let mut manager = user_manager.lock().unwrap();
        manager.new_user(user_addr.to_string(), write);
        println!("New WebSocket connection established and user registered from: {}", user_addr);
    }

    tokio::spawn( async move {
        // handle each message
        while let Some(Ok(msg)) = read.next().await {
            match msg {
                Message::Text(text) => {
                    // handle the payloads
                    if let Ok(payload) = from_str::<Payload>(&text) {
                        handle_payload(payload, user_addr.clone(), user_manager.clone());
                    }
                }
                Message::Close(_) => {
                    // handle the close logic 
                }
                _ => {}
            }
        }
    });
}

fn handle_payload(
    payload: Payload,
    user_addr: String,
    user_manager: Arc<Mutex<UserManager>>
) {
    let mut user_manager = user_manager.lock().unwrap();
    match payload.event {
        Event::TRADE => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_trade(user_addr, payload.symbol);
                },
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_trades(user_addr, payload.symbol);
                }
            }
        }
        Event::TICKER => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_ticker(user_addr, payload.symbol);
                },
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_ticker(user_addr, payload.symbol);
                }
            }
        }
        Event::DEPTH => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_depth(user_addr, payload.symbol);
                },
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_depth(user_addr, payload.symbol);
                }
            }
        }
        Event::ORDER_UPDATE => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.assign_user_id(user_addr, payload.user_id);
                },
                Method::UNSUBSCRIBE => {
                    user_manager.dissociate_user_id(user_addr);
                }
            }
        }
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