use crate::order_book::{BestOrder, Order, OrderBook, OrderSide, Trade};
use log::info;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};

/// In-Memory Limit Order Book implementation.
/// This is an in-memory implementation of a simple limit order book that is fully thread safe.
/// The order book can be cloned easily, because it will point to the same underlying buy and sell orders.
#[derive(Clone)]
pub struct LimitOrderBook {
    buy_orders: Arc<RwLock<Vec<Order>>>,
    sell_orders: Arc<RwLock<Vec<Order>>>,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        LimitOrderBook {
            buy_orders: Arc::new(Default::default()),
            sell_orders: Arc::new(Default::default()),
        }
    }

    /// Placing an order internal handler.
    async fn place_order_internal(&self, order: Order) -> Vec<Trade> {
        let mut lock = match order.side {
            OrderSide::Buy => self.sell_orders.write().await,
            OrderSide::Sell => self.buy_orders.write().await,
        };
        let mut trades = vec![];
        let mut remaining_quantity = order.quantity;

        for curr_order in lock.iter_mut() {
            let order_price_ok = match order.side {
                OrderSide::Buy => curr_order.price <= order.price,
                OrderSide::Sell => curr_order.price >= order.price,
            };
            if order_price_ok && remaining_quantity > 0 {
                // if we have more from current order, just decrease its quantity
                if curr_order.quantity > remaining_quantity {
                    trades.push(Trade::new(
                        curr_order.id,
                        order.id,
                        curr_order.price,
                        remaining_quantity,
                    ));
                    curr_order.quantity -= remaining_quantity;
                    remaining_quantity = 0;
                    break;
                }

                // delete current order from order book if we used all of it's quantity
                if curr_order.quantity <= remaining_quantity {
                    trades.push(Trade::new(
                        curr_order.id,
                        order.id,
                        curr_order.price,
                        curr_order.quantity,
                    ));
                    remaining_quantity -= curr_order.quantity;
                    curr_order.quantity = 0;
                }
            } else {
                break;
            }
        }

        // keep orders only whose has some remaining quantity
        *lock = lock
            .iter()
            .filter(|order| order.quantity > 0)
            .cloned()
            .collect::<Vec<Order>>();
        drop(lock);

        // if we have some quantity left, just add to the corresponding internal order book
        if remaining_quantity > 0 {
            let mut lock = match order.side {
                OrderSide::Buy => self.buy_orders.write().await,
                OrderSide::Sell => self.sell_orders.write().await,
            };
            self.add_order(
                &mut lock,
                Order::new(order.id, OrderSide::Buy, order.price, remaining_quantity),
            )
            .await;
            drop(lock);
        }

        info!("Trades made: {trades:?}");
        info!("Buy orders: {:?}", self.buy_orders.read().await);
        info!("Sell orders: {:?}", self.sell_orders.read().await);

        trades
    }

    /// Collects the best order from Buy or Sell internal order books.
    /// If any found with the same price, just add it's quantity to the total quantity.
    async fn best_order(&self, side: OrderSide) -> Option<BestOrder> {
        let lock = match side {
            OrderSide::Buy => self.buy_orders.read().await,
            OrderSide::Sell => self.sell_orders.read().await,
        };

        if lock.is_empty() {
            return None;
        }
        let mut best_order = BestOrder {
            price: lock[0].price,
            total_quantity: lock[0].quantity,
        };
        for order in lock.iter() {
            if order.price == best_order.price && order.id != lock[0].id {
                best_order.total_quantity += order.quantity;
            }
        }

        Some(best_order)
    }

    /// Adding a new order to the corresponding Buy or Sell internal order book.
    /// After any of the additions there is a sorting to prepare for next matching.
    async fn add_order(&self, lock: &mut RwLockWriteGuard<'_, Vec<Order>>, order: Order) {
        match order.side {
            OrderSide::Buy => {
                lock.push(order);
                lock.sort_by(|a, b| {
                    // if prices are the same, older order wins, so we follow Price-time priority
                    if b.price == a.price {
                        return a.timestamp.cmp(&b.timestamp);
                    }
                    // ordering reverse by price to get the best price leveled order first
                    // for Sell orders
                    b.price.cmp(&a.price)
                });
            }
            OrderSide::Sell => {
                lock.push(order);
                lock.sort_by(|a, b| {
                    // if prices are the same, older order wins, so we follow Price-time priority
                    if b.price == a.price {
                        return a.timestamp.cmp(&b.timestamp);
                    }
                    // ordering incrementally by price to get the best price leveled order first
                    // for Buy orders
                    a.price.cmp(&b.price)
                });
            }
        }
    }
}

#[async_trait::async_trait]
impl OrderBook for LimitOrderBook {
    async fn place_order(&self, order: Order) -> Vec<Trade> {
        self.place_order_internal(order).await
    }

    async fn best_buy(&self) -> Option<BestOrder> {
        self.best_order(OrderSide::Buy).await
    }

    async fn best_sell(&self) -> Option<BestOrder> {
        self.best_order(OrderSide::Sell).await
    }
}

#[cfg(test)]
mod tests {
    use super::LimitOrderBook;
    use crate::order_book::{Order, OrderBook, OrderSide, Trade};

    fn order(id: u64, side: OrderSide, price: u64, quantity: u64, timestamp: u128) -> Order {
        Order {
            id,
            side,
            price,
            quantity,
            timestamp,
        }
    }

    #[tokio::test]
    async fn best_order_levels_on_empty_book_are_none() {
        let order_book = LimitOrderBook::new();

        let best_buy = order_book.best_buy().await;
        let best_sell = order_book.best_sell().await;

        assert!(best_buy.is_none(), "best_buy should be None on empty book");
        assert!(
            best_sell.is_none(),
            "best_sell should be None on empty book"
        );
    }

    #[tokio::test]
    async fn remainder_after_filling_order() {
        let order_book = LimitOrderBook::new();

        let trades = order_book
            .place_order(order(1, OrderSide::Sell, 100, 5, 1))
            .await;
        assert_eq!(trades.len(), 0);

        let trades = order_book
            .place_order(order(2, OrderSide::Buy, 101, 8, 2))
            .await;
        assert_eq!(trades.len(), 1);

        let best_sell = order_book.best_sell().await;
        assert!(best_sell.is_none(), "All sells at 100 should be consumed");

        let best_buy = order_book
            .best_buy()
            .await
            .expect("Remainder buy should rest");
        assert_eq!(best_buy.price, 101);
        assert_eq!(best_buy.total_quantity, 3);
    }

    #[tokio::test]
    async fn non_matching_orders_are_added_to_order_book() {
        let order_book = LimitOrderBook::new();

        let _ = order_book
            .place_order(order(1, OrderSide::Buy, 100, 5, 1))
            .await;
        let _ = order_book
            .place_order(order(2, OrderSide::Sell, 105, 7, 2))
            .await;

        let bb = order_book.best_buy().await.expect("best_buy should exist");
        assert_eq!(bb.price, 100);
        assert_eq!(bb.total_quantity, 5);

        let bs = order_book
            .best_sell()
            .await
            .expect("best_sell should exist");
        assert_eq!(bs.price, 105);
        assert_eq!(bs.total_quantity, 7);
    }

    #[tokio::test]
    async fn multiple_orders_same_price_level_are_aggregated_in_best() {
        let order_book = LimitOrderBook::new();

        let _ = order_book
            .place_order(order(1, OrderSide::Buy, 100, 2, 1))
            .await;
        let _ = order_book
            .place_order(order(2, OrderSide::Buy, 100, 3, 2))
            .await;
        let _ = order_book
            .place_order(order(3, OrderSide::Buy, 100, 3, 2))
            .await;

        let bb = order_book.best_buy().await.expect("best_buy should exist");
        assert_eq!(bb.price, 100);
        assert_eq!(bb.total_quantity, 8);
    }

    #[tokio::test]
    async fn buy_hits_lowest_sell_first_then_next_level() {
        let order_book = LimitOrderBook::new();

        let _ = order_book
            .place_order(order(1, OrderSide::Sell, 100, 3, 1))
            .await;
        let _ = order_book
            .place_order(order(2, OrderSide::Sell, 101, 4, 2))
            .await;

        let trades = order_book
            .place_order(order(3, OrderSide::Buy, 101, 6, 3))
            .await;
        assert_eq!(
            trades.len(),
            2,
            "Should produce two trades across two levels"
        );

        let best_sell = order_book
            .best_sell()
            .await
            .expect("One sell remainder expected");
        assert_eq!(best_sell.price, 100);
        assert_eq!(best_sell.total_quantity, 1);

        let best_buy = order_book.best_buy().await;
        assert!(best_buy.is_none(), "No buy orders should remain");
    }

    #[tokio::test]
    async fn older_order_wins_when_same_price_level() {
        let order_book = LimitOrderBook::new();

        let _ = order_book
            .place_order(order(1, OrderSide::Buy, 100, 4, 1))
            .await;
        let _ = order_book
            .place_order(order(2, OrderSide::Buy, 100, 5, 2))
            .await;
        let _ = order_book
            .place_order(order(3, OrderSide::Buy, 101, 1, 3))
            .await;

        let trades = order_book
            .place_order(order(4, OrderSide::Sell, 100, 6, 3))
            .await;
        let expected_trades = vec![
            Trade::new(3, 4, 101, 1),
            Trade::new(1, 4, 100, 4),
            Trade::new(2, 4, 100, 1),
        ];
        assert_eq!(trades.len(), 3, "Should hit all buy levels");
        assert_eq!(trades, expected_trades);

        let best_buy = order_book
            .best_buy()
            .await
            .expect("Remainder on 100 price level");
        assert_eq!(best_buy.price, 100);
        assert_eq!(best_buy.total_quantity, 4);

        let best_sell = order_book.best_sell().await;
        assert!(best_sell.is_none(), "No resting sells expected");
    }
}
