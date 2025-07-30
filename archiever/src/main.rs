use std::time::{Duration, Instant};

use archiever::{db::ScyllaDb, state::redis_event::RedisEvent};
use redis::{Connection, Value};
use scylla::response::query_result;
use serde_json::from_str;






#[tokio::main]
async fn main() {
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";
    // get db and redis connection
    let mut con = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();

    // listen of the specific queue

    loop {
        let con = &mut con;
        let result = redis::cmd("RPOP").arg("archiever").query::<String>(con);
        match result {
            Ok(query_trading_string) => {
                let queue_trade: RedisEvent = from_str(&query_trading_string).unwrap();
                let start = Instant::now();
                let result = scylla_db.batch_update(queue_trade).await;
                match result {
                    Ok(trade) => {
                        println!("Order updated for trade id : {} in {} ms", trade.id, start.elapsed().as_millis());
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        redis::cmd("RPUSH").arg("archiever").arg(query_trading_string).query::<Value>(con).unwrap();
                    }
                }
            }
            Err(_) => {
 
            }
        }
    }

}


fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Failed to connect to redis");
    let con = client.get_connection().expect("Could not connect to the client");
    con
}