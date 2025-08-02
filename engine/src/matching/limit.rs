use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{matching::{get_epoch_micro, orderbook::Order, Exchange, Filler, Id, OrderSide, OrderStatus, OrderType, OrderUpdate, PostUsers, Price, Quantity, Trade, USERS}, EventTransmitter, RedisEmit};











#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limit {
    pub price: Price,
    pub orders: Vec<Order>,
}

impl Limit {
    pub fn new(price: Price) -> Limit {
        Limit {
            price,
            orders: Vec::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn fill_order(
        &mut self,
        mut order: Order,
        exchange: &Exchange,
        exchange_price: Price,
        mut trade_id: &mut u64,
        should_execute_trade: bool,
        event_tx: Option<EventTransmitter>,
    ) -> Order {
        // move through the orders for self.price and fill the order
        let mut remaining_quantity = order.quantity.clone();
        let mut i = 0;

        while i < self.orders.len() {
            if remaining_quantity == dec!(0) {
                break;
            }
            let limit_order = &mut self.orders[i];
            let event_tx = event_tx.clone();
            // now two cases if current order.quantity is less than or greater_equal to
            match limit_order.quantity > remaining_quantity {
                true => {
                    println!("\tOrder matched");
                    limit_order.quantity -= remaining_quantity;
                    order.quantity = dec!(0);
                    order.order_status = OrderStatus::Filled;
                    limit_order.order_status = OrderStatus::PartiallyFilled;
                    limit_order.filled_quote_quantity += remaining_quantity * exchange_price;
                    if should_execute_trade == true {
                        *trade_id += 1; // number of trades
                        let timestamp = get_epoch_micro();
                        // (seller, buyer)
                        let user_ids = match order.order_side {
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                        };

                        let post_users = exchange_balance(
                            &exchange,
                            remaining_quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1,
                        );

                        let is_buyer_maker = if order.order_type == OrderType::Market
                            && order.order_side == OrderSide::Bid
                        {
                            true
                        } else {
                            false
                        };

                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: remaining_quantity,
                            exchange_price,
                            is_buyer_maker,
                            order_status: order.order_status.clone(),
                            client_order_status: limit_order.order_status.clone(),
                            order_id: order.id,
                            client_order_id: limit_order.id,
                            timestamp,
                        };

                        let order_update_1 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.quantity * exchange_price,
                            order_side: order.order_side.clone(),
                            order_status: order.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: order.user_id,
                        };

                        let order_update_2 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: limit_order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: limit_order.user_id,
                        };

                        let publish_trade = Trade {
                            id: trade.trade_id,
                            is_buyer_maker: trade.is_buyer_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp,
                        };

                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();

                        // publish to redis

                        event_tx.unwrap().send(vec![
                            RedisEmit {
                                cmd: "PUBLISH".to_string(),
                                arg_1: format!("order_update:{}", trade.exchange.symbol),
                                arg_2: serialized_order_update_1,
                            },
                            RedisEmit {
                                cmd: "PUBLISH".to_string(),
                                arg_1: format!("order_update:{}", trade.exchange.symbol),
                                arg_2: serialized_order_update_2,
                            },
                            RedisEmit {
                                cmd: "PUBLISH".to_string(),
                                arg_1: format!("trade:{}", trade.exchange.symbol),
                                arg_2: serialized_publish_trade,
                            },
                            RedisEmit {
                                cmd: "LPUSH".to_string(),
                                arg_1: "archiever".to_string(),
                                arg_2: serialized_filler,
                            },
                        ]);
                    }
                }
                false => {
                    println!("\tAn order was matched");
                    let order_status = match limit_order.quantity == remaining_quantity {
                        true => OrderStatus::Filled,
                        false => OrderStatus::PartiallyFilled,
                    };
                    remaining_quantity -= limit_order.quantity;
                    order.quantity -= limit_order.quantity;
                    order.order_status = order_status;
                    limit_order.order_status = OrderStatus::Filled;
                    limit_order.filled_quote_quantity += exchange_price * limit_order.quantity;

                    if should_execute_trade == true {
                        *trade_id += 1;
                        let timestamp = get_epoch_micro();
                        let user_ids = match order.order_side {
                            OrderSide::Bid => (limit_order.user_id, order.user_id),
                            OrderSide::Ask => (order.user_id, limit_order.user_id),
                        };
                        let post_users = exchange_balance(
                            &exchange,
                            limit_order.quantity,
                            exchange_price,
                            user_ids.0,
                            user_ids.1,
                        );
                        let is_buyer_maker = if order.order_type == OrderType::Market
                            && order.order_side == OrderSide::Bid
                        {
                            true
                        } else {
                            false
                        };
                        let trade = Filler {
                            trade_id: *trade_id,
                            post_users,
                            exchange: exchange.clone(),
                            quantity: limit_order.quantity,
                            exchange_price,
                            is_buyer_maker,
                            order_status: order.order_status.clone(),
                            client_order_status: limit_order.order_status.clone(),
                            order_id: order.id,
                            client_order_id: limit_order.id,
                            timestamp,
                        };
                        let order_update_1 = OrderUpdate {
                            order_id: trade.order_id,
                            client_order_id: trade.client_order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: order.user_id,
                        };
                        let order_update_2 = OrderUpdate {
                            order_id: trade.client_order_id,
                            client_order_id: trade.order_id,
                            executed_quantity: trade.quantity,
                            executed_quote_quantity: trade.exchange_price * trade.quantity,
                            order_side: limit_order.order_side.clone(),
                            order_status: trade.order_status.clone(),
                            price: trade.exchange_price,
                            symbol: trade.exchange.symbol.clone(),
                            trade_id: trade.trade_id,
                            trade_timestamp: timestamp,
                            user_id: limit_order.user_id,
                        };
                        let publish_trade = Trade {
                            id: trade.trade_id,
                            is_buyer_maker: trade.is_buyer_maker,
                            price: trade.exchange_price,
                            quantity: trade.quantity,
                            quote_quantity: trade.exchange_price * trade.quantity,
                            timestamp: timestamp,
                        };
                        let serialized_filler = to_string(&trade).unwrap();
                        let serialized_order_update_1 = to_string(&order_update_1).unwrap();
                        let serialized_order_update_2 = to_string(&order_update_2).unwrap();
                        let serialized_publish_trade = to_string(&publish_trade).unwrap();
                        // publish to mpsc channel
                        event_tx.unwrap().send(vec![
                            RedisEmit {
                                cmd: "LPUSH".to_string(),
                                arg_1: format!("order_update:{}", trade.exchange.symbol),
                                arg_2: serialized_order_update_1,
                            },
                            RedisEmit {
                                cmd: "PUBLISH".to_string(),
                                arg_1: format!("order_update:{}", trade.exchange.symbol),
                                arg_2: serialized_order_update_2,
                            },
                            RedisEmit {
                                cmd: "PUBLISH".to_string(),
                                arg_1: format!("trade:{}", trade.exchange.symbol),
                                arg_2: serialized_publish_trade,
                            },
                            RedisEmit {
                                cmd: "LPUSH".to_string(),
                                arg_1: "archiever".to_string(),
                                arg_2: serialized_filler,
                            },
                        ]);
                    }
                    self.orders.remove(i);
                    continue;
                }
            }
            if order.is_filled() {
                break;
            }
            i += 1;
        }
        order
    }

    pub fn total_volume(&self) -> Decimal {
        self.orders
            .iter()
            .map(|order| order.quantity)
            .reduce(|a, b| a + b)
            .unwrap_or(dec!(0))
    }
}

pub fn exchange_balance(
    exchange: &Exchange,
    quantity: Quantity,
    exchange_price: Price,
    user_id: Id,        // seller
    client_user_id: Id, // buyer
) -> PostUsers {
    // lock the users balance
    let mut users = USERS.lock().unwrap();
    users.unlock_amount(&exchange.base, user_id, quantity);
    users.withdraw(&exchange.base, quantity, user_id);
    users.deposit(&exchange.quote, quantity * exchange_price, user_id);

    users.unlock_amount(&exchange.quote, client_user_id, quantity * exchange_price);
    users.withdraw(&exchange.quote, quantity * exchange_price, client_user_id);
    users.deposit(&exchange.base, quantity, client_user_id);

    let user = users.users.get(&user_id).unwrap().clone();
    let client = users.users.get(&client_user_id).unwrap().clone();
    PostUsers { client, user }
}
