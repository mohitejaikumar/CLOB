use std::{net::TcpListener, sync::Mutex};

use actix_web::{web::{self, scope}, App, HttpServer};
use redis::Connection;

use crate::db::ScyllaDb;






pub struct Application {
    port: u16,
    server: actix_web::dev::Server
}


pub struct AppState {
    pub scylla_db: Mutex<ScyllaDb>, // db integration
    pub redis_connection: Mutex<Connection>,
    pub reqwest: Mutex<reqwest::Client>,
}



fn connect_redis(url:&str) -> Connection {
    let client = redis::Client::open(url).expect("Failed to connect to redis");
    let connection = client.get_connection().expect("Failed to connect to redis");
    connection
}

async fn run<'a>(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";


    let mut redis_connection = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();


    let app_state = web::Data::new(AppState {
        scylla_db: Mutex::new(scylla_db),
        redis_connection: Mutex::new(redis_connection),
        reqwest: Mutex::new(reqwest::Client::new())
    });

    let server = HttpServer::new(move || {
        App::new().service(
            scope("/api/v1")
            .app_data(app_state.clone())
            .service()
        )
    })








}