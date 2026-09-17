//! Order types and status tracking.

use serde::{Deserialize, Serialize};
use crate::exchange::{OrderSide, OrderType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub amount: f64,
    pub filled_amount: f64,
    pub avg_fill_price: Option<f64>,
    pub status: OrderStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: String,
    pub order_id: String,
    pub price: f64,
    pub amount: f64,
    pub fee: f64,
    pub fee_currency: String,
    pub side: OrderSide,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Order {
    pub fn new(exchange: &str, symbol: &str, side: OrderSide, order_type: OrderType, amount: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side,
            order_type,
            amount,
            filled_amount: 0.0,
            avg_fill_price: None,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            fills: vec![],
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected)
    }

    pub fn remaining_amount(&self) -> f64 {
        self.amount - self.filled_amount
    }

    pub fn total_fees(&self) -> f64 {
        self.fills.iter().map(|f| f.fee).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_lifecycle() {
        let mut order = Order::new(
            "coinbase",
            "BTC-USD",
            OrderSide::Buy,
            OrderType::Limit { price: 100_000.0 },
            1.0,
        );

        assert!(!order.is_complete());
        assert_eq!(order.remaining_amount(), 1.0);

        // Simulate partial fill
        order.filled_amount = 0.5;
        order.avg_fill_price = Some(99_900.0);
        order.status = OrderStatus::PartiallyFilled;
        order.fills.push(Fill {
            fill_id: "f1".into(),
            order_id: order.id.clone(),
            price: 99_900.0,
            amount: 0.5,
            fee: 6.0,
            fee_currency: "USD".into(),
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now(),
        });

        assert_eq!(order.remaining_amount(), 0.5);
        assert_eq!(order.total_fees(), 6.0);

        // Complete
        order.filled_amount = 1.0;
        order.status = OrderStatus::Filled;
        assert!(order.is_complete());
    }
}   