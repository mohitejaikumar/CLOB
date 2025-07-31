use std::{net::TcpListener, sync::Mutex};

use actix_web::{web::{self, scope}, App, HttpServer};
use redis::Connection;

use crate::{db::ScyllaDb, routes::{orders::{execute_order, get_open_order, get_open_orders, order_cancel, order_cancel_all}, ping::ping, trades::trades, user::{deposit, get_user, new_user, order_history, withdraw}}};






pub struct Application {
    port: u16,
    server: actix_web::dev::Server
}


impl Application {
    pub async fn build(host: &str, port: &str) -> Result<Self, std::io::Error> {
        let address = format!("{}:{}", host, port);
        let listner = TcpListener::bind(&address)?;
        let port = listner.local_addr().unwrap().port();
        let server = run(listner).await?;
        Ok(Self{
            port,
            server
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct AppState {
    pub scylla_db: Mutex<ScyllaDb>, // db integration
    pub redis_connection: Mutex<Connection>,
    pub reqwest: Mutex<reqwest::Client>,
}



pub fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create Redis client");
    let connection = client.get_connection().unwrap_or_else(|e| {
        eprintln!("Failed to connect to Redis at {}: {}", url, e);
        #[cfg(windows)]
        eprintln!("On Windows, this might be a WSAStartup issue. Ensure Redis connection is initialized in main thread first.");
        panic!("Could not connect to Redis: {}", e);
    });
    connection
}

async fn run<'a>(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";


    let mut redis_connection = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap_or_else(| e| {
        eprintln!("Failed to connect to ScyllaDB at {}: {}", uri, e.to_string() );
        panic!("Failed to connect to ScyllaDB: {}", e.to_string());
    });
    scylla_db.initialize().await.unwrap_or_else(| e| {
        eprintln!("Failed to initialize ScyllaDB at {}: {}", uri, e.to_string() );
        panic!("Failed to initialize ScyllaDB: {}", e.to_string());
    });


    let app_state = web::Data::new(AppState {
        scylla_db: Mutex::new(scylla_db),
        redis_connection: Mutex::new(redis_connection),
        reqwest: Mutex::new(reqwest::Client::new())
    });

    let server = HttpServer::new(move || {
        App::new().service(
            scope("/api/v1")
            .app_data(app_state.clone())
            .service(ping)
            .service(execute_order)
            .service(get_open_order)
            .service(get_open_orders)
            .service(order_cancel)
            .service(order_cancel_all)
            .service(trades)
            .service(
                scope("/user")
                .service(new_user)
                .service(get_user)
                .service(deposit)
                .service(withdraw)
                .service(order_history)
            )
        )
    }).listen(listener)?.run();

    Ok(server)
}