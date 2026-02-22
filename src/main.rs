use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use std::collections::HashMap;
use tokio::sync::{mpsc, watch};

mod config;
mod console_writer;
mod csv_writer;
mod orderbook;
mod state_engine;
mod websocket;

async fn fetch_tick_factor(instrument: &str) -> Result<i64> {
    let url = format!(
        "https://www.deribit.com/api/v2/public/get_instrument?instrument_name={}",
        instrument
    );

    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;

    let tick_size = resp["result"]["tick_size"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("tick_size missing"))?;

    let factor = (1.0 / tick_size).round() as i64;

    Ok(factor)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::parse();

    //fetch metadata
    let futures = config
        .instruments
        .iter()
        .map(|inst| fetch_tick_factor(inst));
    let results = join_all(futures).await;

    let mut tick_factors = HashMap::new();

    for (inst, result) in config.instruments.iter().zip(results) {
        let factor = result?;
        tick_factors.insert(inst.clone(), factor);
    }

    println!("Tick factors: {:?}", tick_factors);

    let (raw_tx, raw_rx) = mpsc::channel(1000);
    let (book_tx, book_rx) = watch::channel(orderbook::OrderBook {
        instrument: "".to_string(),
        bids: vec![],
        asks: vec![],
        timestamp: 0,
    });

    let console_rx = book_rx.clone();
    let csv_rx = book_rx.clone();

    let ws_handle = tokio::spawn(websocket::run(config.instruments.clone(), raw_tx));

    let engine_handle = tokio::spawn(state_engine::run(raw_rx, book_tx, tick_factors));

    let _console_handle = tokio::spawn(console_writer::run(console_rx));

    let _csv_handle = tokio::spawn(csv_writer::run(csv_rx));

    println!("All systems go! Monitoring Deribit L2...");
    tokio::select! {
        res = ws_handle => {
            match res {
                Ok(Ok(_)) => println!("WebSocket task finished normally."),
                Ok(Err(e)) => println!("WebSocket task failed with error: {}", e),
                Err(e) => println!("WebSocket task panicked: {}", e),
            }
        }
        res = engine_handle => {
            match res {
                Ok(_) => println!("Engine task finished."),
                Err(e) => println!("Engine task panicked: {}", e),
            }
        }

         _ = tokio::signal::ctrl_c() => {
        println!("Ctrl+C received. Shutting down...");
    }
    }

    Ok(())
}
