#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]
#![allow(clippy::print_stderr, reason = "Examples are okay to print to stderr")]
//! Subscribe to authenticated (user) WebSocket channels.

use std::str::FromStr as _;

use alloy::primitives::Address;
use futures::StreamExt as _;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::ws::WebSocketClient;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = Uuid::parse_str(&std::env::var("POLYMARKET_API_KEY")?)?;
    let api_secret = std::env::var("POLYMARKET_API_SECRET")?;
    let api_passphrase = std::env::var("POLYMARKET_API_PASSPHRASE")?;
    let address = Address::from_str(&std::env::var("POLYMARKET_ADDRESS")?)?;

    // Build credentials for the authenticated ws channel.
    let credentials = Credentials::new(api_key, api_secret, api_passphrase);

    // Connect using the base wss endpoint and authenticate.
    let client = WebSocketClient::default().authenticate(credentials, address)?;
    println!("Authenticated ws client created.");

    // Provide the specific market IDs you care about, or leave empty to receive all events.
    let markets: Vec<String> = Vec::new();

    let order_stream = client.subscribe_orders(markets.clone())?;
    let trade_stream = client.subscribe_trades(markets)?;
    tokio::pin!(order_stream);
    tokio::pin!(trade_stream);

    println!("Subscribed to user ws channel.");

    loop {
        tokio::select! {
            maybe_order = order_stream.next() => {
                match maybe_order {
                    Some(Ok(order)) => {
                        println!("\n--- Order Update ---");
                        println!("Order ID: {}", order.id);
                        println!("Market: {}", order.market);
                        println!("Side: {:?} Size: {} Status: {:?}", order.side, order.size, order.status);
                    }
                    Some(Err(e)) => eprintln!("Order stream error: {e}"),
                    None => {
                        println!("Order stream ended");
                        break;
                    }
                }
            }
            maybe_trade = trade_stream.next() => {
                match maybe_trade {
                    Some(Ok(trade)) => {
                        println!("\n--- Trade Execution ---");
                        println!("Trade ID: {}", trade.id);
                        println!("Market: {}", trade.market);
                        println!("Side: {:?} Size: {} Price: {}", trade.side, trade.size, trade.price);
                    }
                    Some(Err(e)) => eprintln!("Trade stream error: {e}"),
                    None => {
                        println!("Trade stream ended");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
