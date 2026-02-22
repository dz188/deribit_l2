use crate::orderbook::OrderBook;
use tokio::sync::watch::Receiver;

pub async fn run(mut book_rx: Receiver<OrderBook>) {
    while book_rx.changed().await.is_ok() {
        let book = book_rx.borrow();

        println!("==============================");
        println!("Instrument: {}", book.instrument);
        println!("Timestamp: {}", book.timestamp);

        println!("Top 5 Bids:");
        for level in book.bids.iter().take(5) {
            println!("  {} @ {}", level.amount, level.price);
        }

        println!("Top 5 Asks:");
        for level in book.asks.iter().take(5) {
            println!("  {} @ {}", level.amount, level.price);
        }
    }
}
