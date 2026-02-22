use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{mpsc::Receiver, watch::Sender};

use crate::orderbook::{ActionInternal, BookState, OrderBook, Side};

//from json to rust structure

#[derive(Debug, Deserialize)]
#[serde(tag = "method")]
pub enum DeribitMessage {
    #[serde(rename = "subscription")]
    Subscription { params: Params },

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    //pub channel: String,
    pub data: BookData,
}

#[derive(Debug, Deserialize)]
pub struct BookData {
    pub timestamp: u64,
    pub instrument_name: String,
    pub change_id: u64,
    pub bids: Vec<BookEntry>,
    pub asks: Vec<BookEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    New,
    Change,
    Delete,
}

#[derive(Debug, Deserialize)]
pub struct BookEntry(pub Action, pub f64, pub f64);

struct InstrumentState {
    book: BookState,
    initialized: bool,
    last_change_id: u64,
}

fn apply_entry(book: &mut BookState, side: Side, entry: BookEntry) {
    let BookEntry(action, price, size) = entry;

    let action_internal = match action {
        Action::New => ActionInternal::New,
        Action::Change => ActionInternal::Change,
        Action::Delete => ActionInternal::Delete,
    };

    book.apply_change(side, action_internal, price, size);
}

//orderbook
pub async fn run(
    mut raw_rx: Receiver<String>,
    book_tx: Sender<OrderBook>,
    tick_factors: HashMap<String, i64>,
) {
    let mut instruments: HashMap<String, InstrumentState> = HashMap::new();

    while let Some(text) = raw_rx.recv().await {
        let message: DeribitMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match message {
            DeribitMessage::Subscription { params } => {
                let data = params.data;
                let instrument = data.instrument_name.clone();
                let tick_factor = match tick_factors.get(&instrument) {
                    Some(f) => *f,
                    None => continue,
                };
                let state =
                    instruments
                        .entry(instrument.clone())
                        .or_insert_with(|| InstrumentState {
                            book: BookState::new(instrument.clone(), tick_factor, 1),
                            initialized: false,
                            last_change_id: 0,
                        });
                if !state.initialized {
                    for entry in data.bids {
                        apply_entry(&mut state.book, Side::Bid, entry);
                    }
                    for entry in data.asks {
                        apply_entry(&mut state.book, Side::Ask, entry);
                    }
                    state.initialized = true;
                    state.last_change_id = data.change_id;
                } else {
                    if data.change_id != state.last_change_id + 1 {
                        state.initialized = false;
                        state.book = BookState::new(instrument.clone(), tick_factor, 1);
                        continue;
                    }

                    for entry in data.bids {
                        apply_entry(&mut state.book, Side::Bid, entry);
                    }
                    for entry in data.asks {
                        apply_entry(&mut state.book, Side::Ask, entry);
                    }

                    state.last_change_id = data.change_id;
                }

                let book = state.book.to_orderbook(data.timestamp);

                if book_tx.send(book).is_err() {
                    break;
                }
            }
            DeribitMessage::Other => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_subscription() {
        let text = r#"
        {
          "jsonrpc": "2.0",
          "method": "subscription",
          "params": {
            "channel": "book.BTC-PERPETUAL.100ms",
            "data": {
              "timestamp": 1535098298227,
              "instrument_name": "BTC-PERPETUAL",
              "change_id": 123456,
              "bids": [
                ["new", 50000.0, 10.5]
              ],
              "asks": []
            }
          }
        }
        "#;

        let msg: DeribitMessage = serde_json::from_str(text).unwrap();

        println!("{:?}", msg);
    }
}
