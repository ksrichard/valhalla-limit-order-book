use crate::order_book::util;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u128,
}

impl Order {
    pub fn new(id: u64, side: OrderSide, price: u64, quantity: u64) -> Self {
        Order {
            id,
            side,
            price,
            quantity,
            timestamp: util::current_unix_timestamp(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Trade {
    pub maker_id: u64,
    pub taker_id: u64,
    pub price: u64,
    pub quantity: u64,
}

impl Trade {
    pub fn new(maker_id: u64, taker_id: u64, price: u64, quantity: u64) -> Self {
        Trade {
            maker_id,
            taker_id,
            price,
            quantity,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BestOrder {
    pub price: u64,
    pub total_quantity: u64,
}
