use crate::orderbook::OrderBook;
use csv::Writer;
use std::fs::OpenOptions;
use tokio::sync::watch::Receiver;

pub async fn run(mut book_rx: Receiver<OrderBook>) {
    // Open CSV file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("l2_output.csv")
        .expect("Failed to open CSV output file");

    let mut wtr = Writer::from_writer(file);

    // Write CSV header (top 5 levels)
    let mut header = vec!["instrument".to_string(), "timestamp".to_string()];

    for i in 1..=5 {
        header.push(format!("bid{}_price", i));
        header.push(format!("bid{}_size", i));
    }

    for i in 1..=5 {
        header.push(format!("ask{}_price", i));
        header.push(format!("ask{}_size", i));
    }

    wtr.write_record(&header).ok();

    // Main processing loop
    // watch::Receiver only yields when a new OrderBook snapshot is published
    while book_rx.changed().await.is_ok() {
        let book = book_rx.borrow();

        let mut record = vec![book.instrument.clone(), book.timestamp.to_string()];

        // Serialize top-5 bid levels
        for level in book.bids.iter().take(5) {
            record.push(level.price.to_string());
            record.push(level.amount.to_string());
        }

        // Pad empty levels if fewer than 5 bids exist
        for _ in book.bids.len()..5 {
            record.push(String::new());
            record.push(String::new());
        }

        // Serialize top-5 ask levels
        for level in book.asks.iter().take(5) {
            record.push(level.price.to_string());
            record.push(level.amount.to_string());
        }

        // Pad empty levels if fewer than 5 asks exist
        for _ in book.asks.len()..5 {
            record.push(String::new());
            record.push(String::new());
        }

        // Append row to CSV
        wtr.write_record(&record).ok();

        // Flush ensures durability.
        // In high-throughput systems this could be batched or time-based.
        wtr.flush().ok();
    }
}
