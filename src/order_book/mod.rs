pub mod limit_order_book;
mod models;
pub mod util;

pub use models::*;

#[async_trait::async_trait]
pub trait OrderBook {
    /// Placing an order.
    async fn place_order(&self, order: Order) -> Vec<Trade>;

    /// Returns the best priced Buy order
    /// (including same price orders added to quantity).
    async fn best_buy(&self) -> Option<BestOrder>;

    /// Returns the best priced Sell order
    /// (including same price orders added to quantity).
    async fn best_sell(&self) -> Option<BestOrder>;
}
