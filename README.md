# Deribit L2 Order Book Client (Rust)

## Overview

This project implements a multi-instrument Level 2 (L2) order book client for Deribit using WebSocket JSON-RPC.

The system:

- Connects to Deribit WebSocket API
- Subscribes to L2 order book streams (`book.{instrument}.100ms`)
- Reconstructs deterministic per-instrument order books
- Handles reconnection and heartbeat (Ping/Pong)
- Detects change ID gaps and resets state safely
- Publishes reconstructed books to multiple consumers
- Outputs top-5 depth levels to both console and CSV

The design emphasizes correctness, determinism, separation of concerns, and production-readiness.

---


## Output

The Top 5 bid and ask levels order books are:

- Printed in real time to the console (top-5 depth levels)
- Persisted to `l2_output.csv` for offline analysis


This enables:

- Offline analysis
- Backtesting
- Data pipeline ingestion

---

## Running the Application

```bash
cargo run -- --instruments BTC-PERPETUAL --instruments ETH-PERPETUAL
