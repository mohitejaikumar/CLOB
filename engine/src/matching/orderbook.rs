use rust_decimal_macros::dec;
use serde_json::to_string;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{matching::{error::MatchingEngineErrors, limit::Limit}, EventTransmitter, RedisEmit};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub trade_id: u64,
    pub order_id: u64,
    pub exchange: Exchange,
    pub asks: HashMap<Price, Limit>,
    pub bids: HashMap<Price, Limit>,
}

impl Orderbook {
    pub fn new(exchange: Exchange) -> Orderbook {
        Orderbook {
            trade_id: 0,
            order_id: 0,
            exchange,
            asks: HashMap::new(),
            bids: HashMap::new(),
        }
    }

    pub fn increment_order_id(&mut self) -> OrderId {
        let mut order_id = &mut self.order_id;
        *order_id += 1;
        *order_id
    }

    pub fn users_orders(asks: &mut HashMap<Price, Limit>, user_id: Id) -> Vec<(Price, &mut Order)> {
        asks.values_mut()
            .flat_map(|limit| {
                limit
                    .orders
                    .iter_mut()
                    .filter(|order| order.user_id == user_id)
                    .map(|order| (limit.price, order))
                    .collect::<Vec<(Price, &mut Order)>>()
            })
            .collect::<Vec<(Price, &mut Order)>>()
    }

    pub fn get_open_orders(&mut self, user_id: Id) -> Vec<(Price, &mut Order)> {
        let mut open_orders = Orderbook::users_orders(&mut self.asks, user_id);
        open_orders.extend(Orderbook::users_orders(&mut self.bids, user_id));
        open_orders
    }

    // sorted from lowest to heighest
    pub fn bid_limits(bids: &mut HashMap<Price, Limit>) -> Vec<&mut Limit> {
        let mut bids = bids.values_mut().collect::<Vec<&mut Limit>>();
        bids.sort_by(|a, b| b.price.cmp(&a.price));
        bids
    }

    pub fn ask_limits(asks: &mut HashMap<Price, Limit>) -> Vec<&mut Limit> {
        let mut asks = asks.values_mut().collect::<Vec<&mut Limit>>();
        asks.sort_by(|a, b| a.price.cmp(&b.price));
        asks
    }

    pub fn get_depth(&mut self) -> (HashMap<Price, Quantity>, HashMap<Price, Quantity>) {
        let sorted_bids = Orderbook::bid_limits(&mut self.bids);
        let sorted_asks = Orderbook::bid_limits(&mut self.asks);
        let bids: HashMap<Price, Quantity> = sorted_bids
            .iter()
            .map(|limit| (limit.price, limit.total_volume()))
            .collect();
        let asks: HashMap<Price, Quantity> = sorted_asks
            .iter()
            .map(|limit| (limit.price, limit.total_volume()))
            .collect();
        return (bids, asks);
    }

    pub fn add_limit_order(&mut self, price: Price, order: Order) {
        let order_side = &order.order_side.clone();
        match order_side {
            OrderSide::Bid => {
                let limit = self.bids.get_mut(&price);
                match limit {
                    Some(limit) => limit.add_order(order),
                    None => {
                        let mut limit = Limit::new(price);
                        limit.add_order(order);
                        self.bids.insert(price, limit);
                    }
                }
            }
            OrderSide::Ask => {
                let limit = self.asks.get_mut(&price);
                match limit {
                    Some(limit) => limit.add_order(order),
                    None => {
                        let mut limit = Limit::new(price);
                        limit.add_order(order);
                        self.asks.insert(price, limit);
                    }
                }
            }
        }
    }

    pub fn fill_market_order(
        &mut self,
        mut order: Order,
        should_execute_trade: bool,
        event_tx: Option<EventTransmitter>,
    ) -> (Decimal, Decimal, OrderStatus) {
        let sorted_orders = match order.order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        let mut executed_quantity = dec!(0);
        let mut executed_quote_quantity = dec!(0);
        let mut order_status = order.order_status.clone();

        println!("Recived an {} Market order", order.order_side);

        for limit_order in sorted_orders {
            let price = limit_order.price.clone();
            order = limit_order.fill_order(
                order,
                &self.exchange,
                price,
                &mut self.trade_id,
                should_execute_trade,
                event_tx.clone(),
            );
            let executed_quantity_limit = order.initial_quantity - order.quantity;
            executed_quantity += executed_quantity_limit;
            executed_quote_quantity += executed_quantity_limit * price;
            order_status = order.order_status.clone();
            if order.is_filled() {
                break;
            }
        }
        (executed_quantity, executed_quote_quantity, order_status)
    }

    pub fn fill_limit_order(
        &mut self,
        price: Price,
        mut order: Order,
        should_execute_trade: bool,
        event_tx: Option<EventTransmitter>,
    ) -> (Decimal, Decimal, OrderStatus) {
        println!("Recived an {} Limit order", order.order_side);
        let mut executed_quantity = dec!(0);
        let mut executed_quote_quantity = dec!(0);
        let mut order_status = order.order_status.clone();
        let result = match order.order_side {
            OrderSide::Ask => {
                let sorted_bids = &mut Orderbook::bid_limits(&mut self.bids);
                let mut i = 0;
                if sorted_bids.len() == 0 {
                    self.add_limit_order(price, order);
                    return (executed_quantity, executed_quote_quantity, order_status);
                }
                while i < sorted_bids.len() {
                    if price > sorted_bids[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    order = sorted_bids[i].fill_order(
                        order,
                        &self.exchange,
                        price,
                        &mut self.trade_id,
                        should_execute_trade,
                        event_tx.clone(),
                    );
                    let executed_quantity_limit = order.initial_quantity - order.quantity;
                    executed_quantity += executed_quantity_limit;
                    executed_quote_quantity += executed_quantity_limit * price;
                    order_status = order.order_status.clone();
                    if order.quantity > dec!(0) && sorted_bids.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
            }
            OrderSide::Bid => {
                let sorted_asks = &mut Orderbook::ask_limits(&mut self.asks);
                let mut i = 0;
                if sorted_asks.len() == 0 {
                    self.add_limit_order(price, order);
                    return (executed_quantity, executed_quote_quantity, order_status);
                }
                while i < sorted_asks.len() {
                    if price < sorted_asks[i].price {
                        self.add_limit_order(price, order);
                        break;
                    }
                    let price = sorted_asks[i].price.clone();
                    order = sorted_asks[i].fill_order(
                        order,
                        &self.exchange,
                        price,
                        &mut self.trade_id,
                        should_execute_trade,
                        event_tx.clone(),
                    );
                    let executed_quantity_limit = order.initial_quantity - order.quantity;
                    executed_quantity += executed_quantity_limit;
                    executed_quote_quantity += executed_quantity_limit * price;
                    order_status = order.order_status.clone();
                    if order.quantity > dec!(0) && sorted_asks.get(i + 1).is_none() {
                        self.add_limit_order(price, order);
                        break;
                    }
                    i += 1;
                }
            }
        };
        (executed_quantity, executed_quote_quantity, order_status)
    }

    pub fn cancel_all_orders(
        &mut self,
        user_id: Id,
    ) -> (Vec<RecievedOrder>, HashMap<String, String>) {
        let quote = self.exchange.quote.clone();
        let base = self.exchange.base.clone();
        let symbol = self.exchange.symbol.clone();
        let mut open_orders = self.get_open_orders(user_id);
        let mut users = USERS.lock().unwrap();
        let orders: Vec<RecievedOrder> = open_orders
            .iter()
            .map(|(price, order)| {
                match order.order_side {
                    OrderSide::Bid => {
                        users.unlock_amount(&quote, user_id, order.quantity * price);
                    }
                    OrderSide::Ask => {
                        users.unlock_amount(&base, user_id, order.quantity);
                    }
                }
                RecievedOrder {
                    id: order.id as i64,
                    filled_quantity: order.initial_quantity - order.quantity,
                    filled_quote_quantity: order.filled_quote_quantity,
                    initial_quantity: order.initial_quantity,
                    order_side: order.order_side.clone(),
                    order_status: OrderStatus::Cancelled,
                    order_type: order.order_type.clone(),
                    price: *price,
                    quote_quantity: order.initial_quantity * price,
                    symbol: symbol.clone(),
                    timestamp: order.timestamp as i64,
                    user_id: order.user_id as i64,
                }
            })
            .collect();
        self.asks
            .values_mut()
            .for_each(|limit| limit.orders.retain(|order| order.user_id != user_id));
        self.bids
            .values_mut()
            .for_each(|limit| limit.orders.retain(|order| order.user_id != user_id));
        let locked_balances: &HashMap<String, String> = &users
            .users
            .get(&user_id)
            .unwrap()
            .locked_balance
            .iter()
            .map(|(asset, balance)| (asset.to_string(), balance.to_string()))
            .collect();
        (orders, locked_balances.clone())
    }
    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        order_side: &OrderSide,
        price: &Price,
    ) -> Result<Order, MatchingEngineErrors> {
        match order_side {
            OrderSide::Bid => {
                let mut limit = self.bids.get_mut(price);
                match limit {
                    Some(limit) => {
                        let index = limit.orders.iter().position(|order| order.id == order_id);
                        match index {
                            Some(index) => {
                                let order = limit.orders.get(index).unwrap().clone();
                                limit.orders.remove(index);
                                Ok(order)
                            }
                            None => Err(MatchingEngineErrors::InvalidOrderId),
                        }
                    }
                    None => Err(MatchingEngineErrors::InvalidPriceLimitOrOrderSide),
                }
            }
            OrderSide::Ask => {
                let mut limit = self.asks.get_mut(price);
                match limit {
                    Some(limit) => {
                        let index = limit.orders.iter().position(|order| order.id == order_id);
                        match index {
                            Some(index) => {
                                let order = limit.orders.get(index).unwrap().clone();
                                limit.orders.remove(index);
                                Ok(order)
                            }
                            None => Err(MatchingEngineErrors::InvalidOrderId),
                        }
                    }
                    None => Err(MatchingEngineErrors::InvalidPriceLimitOrOrderSide),
                }
            }
        }
    }

    pub fn process_order(
        &mut self,
        recieved_order: RecievedOrder,
        order_id: OrderId,
        event_tx: EventTransmitter,
    ) -> (Decimal, Decimal, OrderStatus) {
        let order = Order::new(
            order_id as u64,
            recieved_order.timestamp as u64,
            recieved_order.order_side,
            recieved_order.initial_quantity,
            recieved_order.order_type.clone(),
            recieved_order.user_id as u64,
        );
        match recieved_order.order_type {
            // redis transmitter
            OrderType::Market => self.fill_market_order(order, true, Some(event_tx)),
            OrderType::Limit => {
                self.fill_limit_order(recieved_order.price, order, true, Some(event_tx))
            }
        }
    }

    pub async fn recover_orderbook(&mut self, session: &Session) {
        self.recover_trade_id(&session).await;
        self.recover_order_id(&session).await;
        self.replay_orders(&session).await;
    }
    async fn recover_trade_id(&mut self, session: &Session) {
        let s = r#"
            SELECT COUNT(*) FROM keyspace_1.trade_table;
                "#;

        let res = session.query_unpaged(s, &[]).await.unwrap();
        let mut temp = res.into_rows_result().unwrap();
        let mut res = temp.rows::<(i64,)>().unwrap();
        let trade_id = res.next().transpose().unwrap().unwrap().0;
        self.trade_id = trade_id as u64;
    }
    async fn recover_order_id(&mut self, session: &Session) {
        let s = r#"
            SELECT COUNT(*) FROM keyspace_1.order_table;
                "#;
        let res = session.query_unpaged(s, &[]).await.unwrap();
        let mut temp = res.into_rows_result().unwrap();
        let mut res = temp.rows::<(i64,)>().unwrap();
        let order_id = res.next().transpose().unwrap().unwrap().0;
        self.order_id = order_id as u64;
    }

    async fn replay_orders(&mut self, session: &Session) {
        let current_time = get_epoch_ms() as i64;
        let since = 1000 * 60 * 60 * 24; // 24 hours in millis
        let from_time = current_time - since;
        let canceled_order_s = r#"
        SELECT 
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        FROM keyspace_1.cancel_order_table
        WHERE timestamp > ? AND symbol = ? ALLOW FILTERING;
            "#;
        let normal_order_s = r#"
        SELECT 
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity, 
            quote_quantity,
            filled_quote_quantity,
            order_type,
            order_side,
            order_status,
            timestamp
        FROM keyspace_1.order_table
        WHERE timestamp > ? AND symbol = ? ALLOW FILTERING;
            "#;
        enum OrderRequest {
            Cancel(ScyllaCancelOrder),
            Normal(RecievedOrder),
        }
        let symbol = &self.exchange.symbol;
        let res = session
            .query_unpaged(normal_order_s, (from_time, symbol))
            .await
            .unwrap();
        let cancel_res = session
            .query_unpaged(canceled_order_s, (from_time, symbol))
            .await
            .unwrap();
        let mut temp = res.into_rows_result().unwrap();
        let mut orders = temp.rows::<ScyllaOrder>().unwrap();
        let mut temp_cancel = cancel_res.into_rows_result().unwrap();
        let mut canceled_orders = temp_cancel.rows::<ScyllaCancelOrder>().unwrap();
        let mut replay_orders: Vec<OrderRequest> = orders
            .map(|order| {
                let order = order.unwrap().from_scylla_order();
                OrderRequest::Normal(order)
            })
            .collect();
        let mut canceled_orders: Vec<OrderRequest> = canceled_orders
            .map(|order| {
                let order = order.unwrap();
                OrderRequest::Cancel(order)
            })
            .collect();
        replay_orders.extend(canceled_orders);
        replay_orders.sort_by(|r1, r2| {
            let r1_timestamp = match r1 {
                OrderRequest::Cancel(c_order) => c_order.timestamp,
                OrderRequest::Normal(n_order) => n_order.timestamp,
            };
            let r2_timestamp = match r2 {
                OrderRequest::Cancel(c_order) => c_order.timestamp,
                OrderRequest::Normal(n_order) => n_order.timestamp,
            };
            r1_timestamp.cmp(&r2_timestamp)
        });
        for replay_order in replay_orders {
            match replay_order {
                OrderRequest::Cancel(c_order) => {
                    self.cancel_order(
                        c_order.id as u64,
                        &OrderSide::from_str(&c_order.order_side).unwrap(),
                        &Decimal::from_str(&c_order.price).unwrap(),
                    )
                    .unwrap();
                    println!("Cancelled an {} Open order", c_order.order_side);
                }
                OrderRequest::Normal(replay_order) => {
                    let order = Order::new(
                        replay_order.id as u64,
                        replay_order.timestamp as u64,
                        replay_order.order_side,
                        replay_order.initial_quantity,
                        replay_order.order_type.clone(),
                        replay_order.user_id as u64,
                    );
                    let _ = match replay_order.order_type {
                        OrderType::Market => self.fill_market_order(order, false, None),
                        OrderType::Limit => {
                            self.fill_limit_order(replay_order.price, order, false, None)
                        }
                    };
                }
            }
        }
    }

    pub fn get_quote(
        &mut self,
        order_side: &OrderSide,
        mut order_quantity: Quantity,
    ) -> Result<Decimal, MatchingEngineErrors> {
        let sorted_orders = match order_side {
            OrderSide::Ask => Orderbook::bid_limits(&mut self.bids),
            OrderSide::Bid => Orderbook::ask_limits(&mut self.asks),
        };
        let mut orderbook_quote = dec!(0);
        for limit_order in sorted_orders {
            let total_quantity = limit_order.total_volume();
            if total_quantity >= order_quantity {
                orderbook_quote += order_quantity * limit_order.price;
                return Ok(orderbook_quote);
            }
            orderbook_quote += total_quantity * limit_order.price;
            order_quantity -= total_quantity;
        }
        Err(MatchingEngineErrors::AskedMoreThanTradeable)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: Id,
    pub initial_quantity: Quantity,
    pub filled_quote_quantity: Quantity,
    pub quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u64,
}

impl Order {
    pub fn new(
        id: OrderId,
        timestamp: u64,
        order_side: OrderSide,
        quantity: Quantity,
        order_type: OrderType,
        user_id: Id,
    ) -> Order {
        Order {
            id,
            user_id,
            initial_quantity: quantity,
            filled_quote_quantity: dec!(0),
            quantity,
            order_type,
            order_side,
            order_status: OrderStatus::InProgress,
            timestamp,
        }
    }

    pub fn is_filled(&self) -> bool {
        self.quantity == dec!(0)
    }
}


