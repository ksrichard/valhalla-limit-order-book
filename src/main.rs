use crate::order_book::OrderBook;
use crate::order_book::limit_order_book::LimitOrderBook;
use axum::Router;
use axum::routing::{get, post};
use log::info;
use simple_logger::SimpleLogger;
use std::sync::Arc;

mod handlers;
mod order_book;

#[derive(Clone)]
pub struct AppState<O: OrderBook> {
    pub order_book: Arc<O>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::new().env().init()?;

    let order_book = LimitOrderBook::new();
    let app_state = AppState {
        order_book: Arc::new(order_book),
    };

    // HTTP server to expose order book functionality
    let router = Router::new()
        .route(
            "/place_order",
            post(handlers::order_book::place_order_handler),
        )
        .route("/best_buy", get(handlers::order_book::best_buy_handler))
        .route("/best_sell", get(handlers::order_book::best_sell_handler))
        .with_state(app_state);

    info!("Starting HTTP server at 0.0.0.0:9999...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
