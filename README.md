# 🦀 CrabbyCryptoTracker

A simple Rust-based command-line application that connects to the [Coinbase WebSocket Feed](https://docs.cdp.coinbase.com/exchange/docs/websocket-overview), subscribes to live cryptocurrency price updates, and prints the latest price of each tracked symbol every 30 seconds.

Built with ❤️ using `tokio`, `tungstenite`, and `serde`.

---

## ✨ Features

- Connects securely to `wss://ws-feed.exchange.coinbase.com`
- Subscribes to one or more cryptocurrency symbols (e.g. `BTC-USD`, `ETH-USD`)
- Loads symbols dynamically from a CSV file (`symbols.csv`)
- Periodically prints the latest price for each symbol (every 30 seconds)
- Designed for learning Rust async, WebSockets, and real-time data handling

---

## 📦 Dependencies

- `tokio` – asynchronous runtime
- `tokio-tungstenite` – async WebSocket client
- `serde` / `serde_json` – JSON deserialization
- `csv` – for reading crypto symbols from a CSV file
- `url`, `futures-util` – WebSocket and stream helpers

See [`Cargo.toml`](./Cargo.toml) for exact versions.

---

## 🚀 Getting Started

### 1. Clone the repo

```bash
git clone https://github.com/yourusername/crabbycryptotracker.git
cd crabbycryptotracker

 A simple rust implementation to pull crypto prices 
