// Standard library imports
use std::{
    collections::HashMap,     // For storing the latest price per crypto symbol
    error::Error,             // Trait to return errors from our main() and functions
    fs::File,                 // Used to open the CSV file
    path::Path,               // Used to handle file paths
    sync::{Arc, Mutex},       // Arc for shared state across tasks, Mutex for thread-safe mutation
};

// Async + WebSocket + JSON + CSV handling
use futures_util::{SinkExt, StreamExt};            // For working with WebSocket input/output
use serde::Deserialize;                            // For converting JSON messages into Rust structs
use tokio::time::{sleep, Duration};                // Async sleep and timing
use tokio_tungstenite::connect_async;              // WebSocket client for Tokio
use url::Url;                                      // To parse the wss:// URL
use csv::ReaderBuilder;                            // CSV parser

// Struct representing the JSON format of messages we receive from Coinbase
#[derive(Debug, Deserialize)]
struct TickerMessage {
    #[serde(rename = "type")]
    msg_type: String,        // The message type (e.g., "ticker")
    product_id: String,      // The trading pair (e.g., "BTC-USD")
    price: Option<String>,   // The price (may be None if not present)
}

// Function that reads a CSV file and extracts a list of product IDs (symbols)
fn load_symbols_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;  // Open the file, `?` handles error forwarding
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file); // CSV reader that skips the header
    let mut symbols = Vec::new();  // To store our symbols

    // Iterate over each row in the CSV
    for result in rdr.records() {
        let record = result?;  // Handle any row-level parsing errors

        // Extract the first column (assumes CSV has a single "symbol" column)
        if let Some(sym) = record.get(0) {
            symbols.push(sym.trim().to_string()); // Trim whitespace and store symbol
        }
    }

    Ok(symbols)  // Return the vector of symbols
}

// The async entry point of your application (runs inside the Tokio runtime)
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Step 1: Load trading symbols like BTC-USD, ETH-USD from your CSV file
    let product_ids = load_symbols_from_csv("crypto.csv")?;  // Handle error if file not found

    println!("Loaded symbols from CSV: {:?}", product_ids);

    // Step 2: Format those symbols into the JSON structure that Coinbase expects
    let joined_ids = product_ids.join(r#"", ""#);  // Join with commas and quotes
    let subscribe_msg = format!(
        r#"{{
            "type": "subscribe",
            "channels": [{{ "name": "ticker", "product_ids": ["{}"] }}]
        }}"#,
        joined_ids
    );

    // Step 3: Connect to the Coinbase WebSocket server securely over wss://
    let url = Url::parse("wss://ws-feed.exchange.coinbase.com")?;
    let (ws_stream, _) = connect_async(url).await?;  // Connect and await success
    let (mut write, mut read) = ws_stream.split();   // Split into read/write halves

    // Step 4: Send the subscription message so Coinbase knows what you want
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg))
        .await?;

    // Step 5: Create shared memory (a HashMap) that stores the latest price for each symbol
    let prices = Arc::new(Mutex::new(HashMap::new()));  // Use Arc to share across threads/tasks
    let prices_clone = Arc::clone(&prices);             // Clone for use in the background task

    // Step 6: Spawn a background task that runs every 30 seconds and prints the latest prices
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;  // Wait 30 seconds

            let prices = prices_clone.lock().unwrap();  // Safely access shared memory
            println!("\n==== Latest Prices (every 30 seconds) ====");
            for (symbol, price) in prices.iter() {
                println!("{}: ${}", symbol, price);  // Print each symbol and its latest price
            }
            println!("===========================================\n");
        }
    });

    // Step 7: Main WebSocket reading loop — receive messages from Coinbase continuously
    while let Some(msg) = read.next().await {
        match msg {
            Ok(m) => {
                if m.is_text() {
                    let text = m.to_text().unwrap();

                    // Try to parse the incoming message into our TickerMessage struct
                    if let Ok(parsed) = serde_json::from_str::<TickerMessage>(text) {
                        // Only act on messages of type "ticker" that have a price
                        if parsed.msg_type == "ticker" && parsed.price.is_some() {
                            let mut map = prices.lock().unwrap();  // Get write access to shared price map
                            map.insert(parsed.product_id.clone(), parsed.price.unwrap()); // Update the latest price
                        }
                    }
                }
            }
            Err(e) => {
                // Log any WebSocket errors that happen
                eprintln!("WebSocket error: {}", e);
                break; // Exit the loop on error (optional — you could reconnect instead)
            }
        }
    }

    Ok(())  // Signal successful execution to Rust
}
