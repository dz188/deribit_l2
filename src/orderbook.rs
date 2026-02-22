use std::collections::BTreeMap;

pub type Price = i64;
pub type Size = i64;

#[derive(Debug, Clone)]
pub struct OrderBookLevel {
    pub price: Price,
    pub amount: Size,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub instrument: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: u64,
}

#[derive(Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Clone, Copy)]
pub enum ActionInternal {
    New,
    Change,
    Delete,
}

pub struct BookState {
    pub instrument: String,
    bids: BTreeMap<Price, Size>,
    asks: BTreeMap<Price, Size>,
    price_factor: i64,
    size_factor: i64,
}

impl BookState {
    pub fn new(instrument: String, price_factor: i64, size_factor: i64) -> Self {
        Self {
            instrument,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            price_factor,
            size_factor,
        }
    }

    fn to_price(&self, raw: f64) -> Price {
        (raw * self.price_factor as f64).round() as i64
    }

    fn to_size(&self, raw: f64) -> Size {
        (raw * self.size_factor as f64).round() as i64
    }

    pub fn apply_change(
        &mut self,
        side: Side,
        action: ActionInternal,
        price_raw: f64,
        size_raw: f64,
    ) {
        let price = self.to_price(price_raw);

        match action {
            ActionInternal::Delete => match side {
                Side::Bid => {
                    self.bids.remove(&price);
                }
                Side::Ask => {
                    self.asks.remove(&price);
                }
            },

            ActionInternal::New | ActionInternal::Change => {
                let size = self.to_size(size_raw);
                match side {
                    Side::Bid => {
                        self.bids.insert(price, size);
                    }
                    Side::Ask => {
                        self.asks.insert(price, size);
                    }
                }
            }
        }
    }
    pub fn to_orderbook(&self, timestamp: u64) -> OrderBook {
        OrderBook {
            instrument: self.instrument.clone(),
            bids: self
                .bids
                .iter()
                .rev()
                .map(|(p, a)| OrderBookLevel {
                    price: *p,
                    amount: *a,
                })
                .collect(),
            asks: self
                .asks
                .iter()
                .map(|(p, a)| OrderBookLevel {
                    price: *p,
                    amount: *a,
                })
                .collect(),
            timestamp,
        }
    }
}
