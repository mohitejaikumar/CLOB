



use std::collections::HashMap;
use crate::matching::{error::MatchingEngineErrors, orderbook::{Limit, Orderbook}};

use super::*;


pub struct MatchingEngine{
    pub orderbook: HashMap<Exchange, Orderbook>
}

impl MatchingEngine{
    pub fn init() -> MatchingEngine {
        MatchingEngine {
            orderbook: HashMap::new()
        }
    }

    pub fn get_asks(
        &mut self,
        exchange: &Exchange
    ) -> Vec<&mut Limit> {
        let orderbook = self.orderbook.get_mut(exchange).unwrap();
        Orderbook::ask_limits(&mut orderbook.asks)
    }

    pub fn get_bids(
        &mut self,
        exchange: &Exchange
    ) -> Vec<&mut Limit> {
        let orderbook = self.orderbook.get_mut(exchange).unwrap();
        Orderbook::bid_limits(&mut orderbook.bids)
    }

    pub fn add_new_market(
        &mut self,
        exchange: Exchange
    ) -> Result<&mut Self, MatchingEngineErrors> {
        let exists = self.orderbook.contains_key(&exchange);
        if exists == true {
            return Err(MatchingEngineErrors::ExchangeAlreadyExist);
        }
        self.orderbook.insert(exchange.clone(), Orderbook::new(exchange));
        Ok(self)
    }

    pub fn get_quote(
        &mut self,
        order_side: &OrderSide,
        order_quantity: Quantity,
        exchange: &Exchange
    ) -> Result<Decimal, MatchingEngineErrors> {
        let mut orderbook = self.orderbook.get_mut(&exchange).unwrap();
        orderbook.get_quote(order_side, order_quantity)

    }

    pub fn registered_exchange(&self) -> Vec<Symbol> {
        let exchanges: Vec<Symbol> = RegisteredSymbols::iter()
            .map(|s| s.to_string())
            .collect();
        exchanges
    }

    pub fn increment_order_id(&mut self, exchange: &Exchange) -> OrderId {
        let mut order_id = &mut self.orderbook.get_mut(exchange).unwrap().order_id;
        *order_id +=1;
        *order_id
    }

    pub async fn recover_all_orderbooks(
        &mut self,
        session: &Session
    ) {
        // recover users and orderbook
        let symbols = self.registered_exchange();
        let mut orderbooks = &mut self.orderbook;
        println!("Recovering users");
        
        let res = session.query_unpaged("SELECT * FROM keyspace_1.user_table", &[]).await.unwrap();
        let mut query_rows = res.into_rows_result().unwrap();
        let mut users = query_rows.rows::<ScyllaUser>().unwrap();
        let users: Vec<User> = users.map(| user | {
            user.unwrap().from_scylla_user()
        }).collect();

        let mut users_global = USERS.lock().unwrap();
        for user in users {
            users_global.users.insert(user.id, user);

        }

        for symbol in symbols {
            println!("Recovering orderbook for {}", symbol);
            let exchange = Exchange::from_str(&symbol.to_string()).unwrap();
            let mut orderbook = Orderbook::new(exchange.clone());
            orderbook.recover_orderbook(session).await;
            orderbooks.insert(exchange, orderbook);
        }

        println!("Recovering orderbook completed");
    }
}


fn setup_engine_and_users() -> (MatchingEngine, Exchange, Orderbook, Vec<Id>) { // pass redis connection as arg
    let mut engine = MatchingEngine::init();
    let exchange = Exchange::new(Asset::SOL, Asset::USDT);
    let mut orderbook = Orderbook::new(exchange.clone());
    engine.add_new_market(exchange.clone());
    
    // redis client
    // redis connection

    let ids: Vec<Id> = [1,2,3,4,5,6,7,8].to_vec();
    (engine, exchange, orderbook, ids)
}