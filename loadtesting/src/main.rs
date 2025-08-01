use goose::prelude::*;
use loadtesting::state::{
    CancelAll, CancelOrder, Deposit, OpenOrder, OpenOrders, OrderParams, OrderSide, OrderType,
    User, Withdraw,
};
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, str::FromStr, time::Instant};

// Load testing scenarios
#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("User Operations")
                .register_transaction(transaction!(ping).set_weight(1)?)
                .register_transaction(transaction!(create_user_transfer_credits).set_weight(3)?),
        ) // Add your backend URL here
        .execute()
        .await?;

    Ok(())
}

async fn create_user_transfer_credits(user: &mut GooseUser) -> TransactionResult {
    // create user
    // deposite 100000 USDC and 100000 SOL and 100000 ETH
    // execute orders with price [10, 15, 13, 17], quantity [20, 20, 4] both sell and buy limit and market order both
    // get all open orders of user

    let user_response = user.post_json("/api/v1/user/user", &json!({})).await?;
    let temp = user_response.response?;
    let user_id = match temp.json::<User>().await {
        Ok(u) => u.id,
        Err(e) => {
            println!("Failed to parse user response: {:?}", e);
            return Ok(());
        }
    };
    for asset in ["USDT", "SOL", "ETH"] {
        let deposite_body = Deposit {
            user_id,
            asset: asset.to_string(),
            quantity: Decimal::from_str("100000").unwrap(),
        };
        let deposit_funds_response = user
            .post_json("/api/v1/user/deposit", &deposite_body)
            .await?;
        println!("deposit_funds_response: {:?}", deposit_funds_response);
    }
    // Generate 100 random prices between 95.0 and 97.0
    let mut order_price = Vec::new();
    let mut order_quantity = Vec::new();

    for _ in 0..100 {
        // Random price between 95.0 and 97.0 with 2 decimal precision
        let price = rand::rng().random_range(9500..=9700) as f64 / 100.0;
        order_price.push(price);

        // Random quantity between 5.0 and 55.0 with 1 decimal precision
        let quantity = rand::rng().random_range(50..=550) as f64 / 10.0;
        order_quantity.push(quantity);
    }
    let order_side = ["Bid", "Ask"];
    let order_type = ["Limit", "Market"];
    for i in 0..order_price.len() {
        let start = Instant::now();
        let order_body = OrderParams {
            price: Decimal::from_str(&format!("{}", order_price[i])).unwrap(),
            symbol: "SOL_USDT".to_string(),
            order_side: order_side[rand::rng().random_range(0..2)].to_string(),
            order_type: order_type[rand::rng().random_range(0..2)].to_string(),
            quantity: Decimal::from_str(&format!("{}", order_quantity[i])).unwrap(),
            user_id: user_id,
        };
        let order_response = user.post_json("/api/v1/order", &order_body).await?;
        println!("order_response: {:?}", order_response);
        println!("time taken: {:?}", start.elapsed().as_millis());
    }
    Ok(())
}

// Health check endpoint
async fn ping(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/api/v1/ping").await?;
    Ok(())
}

async fn get_user_balances(user: &mut GooseUser) -> TransactionResult {
    // Generate a random user ID for testing
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let _response = user.get(&format!("/?user_id={}", user_id)).await?;
    Ok(())
}

async fn deposit_funds(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let assets = ["USDT", "BTC", "SOL", "ETH"];
    let asset = assets[rand::rng().random_range(0..assets.len())];
    let quantity = Decimal::from_str(&format!("{}", rand::rng().random_range(1..=1000))).unwrap();

    let deposit = Deposit {
        user_id,
        asset: asset.to_string(),
        quantity,
    };

    let _response = user.post_json("/deposit", &deposit).await?;
    Ok(())
}

async fn withdraw_funds(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let assets = ["USDT", "BTC", "SOL", "ETH"];
    let asset = assets[rand::rng().random_range(0..assets.len())];
    let quantity = Decimal::from_str(&format!("{}", rand::rng().random_range(1..=100))).unwrap();

    let withdraw = Withdraw {
        user_id,
        asset: asset.to_string(),
        quantity,
    };

    let _response = user.post_json("/withdraw", &withdraw).await?;
    Ok(())
}

async fn get_order_history(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let _response = user
        .get(&format!("/history/orders?user_id={}", user_id))
        .await?;
    Ok(())
}

// Trading functions
async fn execute_order(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let symbol = symbols[rand::rng().random_range(0..symbols.len())];
    let order_sides = ["Bid", "Ask"];
    let order_side = order_sides[rand::rng().random_range(0..order_sides.len())];
    let order_types = ["Market", "Limit"];
    let order_type = order_types[rand::rng().random_range(0..order_types.len())];

    let price = Decimal::from_str(&format!("{}", rand::rng().random_range(1000..=50000))).unwrap();
    let quantity = Decimal::from_str(&format!("0.{}", rand::rng().random_range(1..=1000))).unwrap();

    let order = OrderParams {
        price,
        order_side: order_side.to_string(),
        order_type: order_type.to_string(),
        quantity,
        user_id,
        symbol: symbol.to_string(),
    };

    let _response = user.post_json("/order", &order).await?;
    Ok(())
}

// async fn cancel_all_orders(user: &mut GooseUser) -> TransactionResult {
//     let user_id = (rand::rng().random_range(1..=1000)) as i64;
//     let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
//     let symbol = symbols[rand::rng().random_range(0..symbols.len())];

//     let cancel_all = CancelAll {
//         user_id,
//         symbol: symbol.to_string(),
//     };

//     let _response = user.delete("/orders")(&cancel_all).await?;
//     Ok(())
// }

// async fn cancel_order(user: &mut GooseUser) -> TransactionResult {
//     let user_id = (rand::rng().random_range(1..=1000)) as i64;
//     let order_id = (rand::rng().random_range(1..=10000)) as i64;
//     let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
//     let symbol = symbols[rand::rng().random_range(0..symbols.len())];
//     let order_sides = ["Bid", "Ask"];
//     let order_side = order_sides[rand::rng().random_range(0..order_sides.len())];
//     let price = Decimal::from_str(&format!("{}", rand::rng().random_range(1000..=50000))).unwrap();

//     let cancel_order = CancelOrder {
//         id: order_id,
//         user_id,
//         symbol: symbol.to_string(),
//         price,
//         order_side: order_side.to_string(),
//     };

//     let _response = user.delete("/order").json(&cancel_order).await?;
//     Ok(())
// }

async fn get_open_order(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let order_id = (rand::rng().random_range(1..=10000)) as i64;
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let symbol = symbols[rand::rng().random_range(0..symbols.len())];

    let open_order = OpenOrder {
        user_id,
        order_id,
        symbol: symbol.to_string(),
    };

    // GET requests with JSON body are converted to query parameters
    let _response = user
        .get(&format!(
            "/order?user_id={}&order_id={}&symbol={}",
            open_order.user_id, open_order.order_id, open_order.symbol
        ))
        .await?;
    Ok(())
}

async fn get_open_orders(user: &mut GooseUser) -> TransactionResult {
    let user_id = (rand::rng().random_range(1..=1000)) as i64;
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let symbol = symbols[rand::rng().random_range(0..symbols.len())];

    let open_orders = OpenOrders {
        user_id,
        symbol: symbol.to_string(),
    };

    // GET requests with JSON body are converted to query parameters
    let _response = user
        .get(&format!(
            "/orders?user_id={}&symbol={}",
            open_orders.user_id, open_orders.symbol
        ))
        .await?;
    Ok(())
}

async fn get_trades(user: &mut GooseUser) -> TransactionResult {
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let symbol = symbols[rand::rng().random_range(0..symbols.len())];

    let _response = user.get(&format!("/trades?symbol={}", symbol)).await?;
    Ok(())
}
