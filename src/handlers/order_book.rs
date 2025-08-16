use crate::AppState;
use crate::order_book::{Order, OrderBook, OrderSide};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PlaceOrderRequest {
    pub id: u64,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u64,
}

impl From<PlaceOrderRequest> for Order {
    fn from(request: PlaceOrderRequest) -> Self {
        Order::new(request.id, request.side, request.price, request.quantity)
    }
}

pub async fn place_order_handler<O: OrderBook>(
    State(AppState { order_book }): State<AppState<O>>,
    Json(request): Json<PlaceOrderRequest>,
) -> impl IntoResponse {
    let trades = order_book.place_order(request.into()).await;
    (StatusCode::OK, Json(trades))
}

pub async fn best_buy_handler<O: OrderBook>(
    State(AppState { order_book }): State<AppState<O>>,
) -> impl IntoResponse {
    let best_order = order_book.best_buy().await;
    (StatusCode::OK, Json(best_order))
}

pub async fn best_sell_handler<O: OrderBook>(
    State(AppState { order_book }): State<AppState<O>>,
) -> impl IntoResponse {
    let best_order = order_book.best_sell().await;
    (StatusCode::OK, Json(best_order))
}
