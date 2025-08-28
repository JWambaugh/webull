use dotenv::dotenv;
use std::env;
use webull::{error::Result, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    println!("Logging in to live trading account...");
    let mut client = WebullClient::new_live(Some(6))?;
    client
        .login(&username, &password, None, None, None, None)
        .await?;

    println!("Login successful!");

    // Get current orders
    println!("\nFetching current open orders...");
    match client.get_orders(None).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("No open orders found.");
            } else {
                println!("Found {} open order(s):", orders.len());
                for (i, order) in orders.iter().enumerate() {
                    println!("\n--- Order {} ---", i + 1);
                    println!("  Order ID: {}", order.order_id);
                    let symbol = order
                        .ticker
                        .as_ref()
                        .map(|t| t.symbol.as_str())
                        .unwrap_or("Unknown");
                    println!("  Symbol: {}", symbol);
                    println!("  Action: {:?}", order.action);
                    println!("  Order Type: {:?}", order.order_type);
                    println!("  Status: {:?}", order.status);

                    println!("  Quantity: {}", order.quantity);
                    println!("  Filled: {}", order.filled_quantity);

                    if let Some(price) = order.limit_price {
                        println!("  Limit Price: ${:.2}", price);
                    }

                    if let Some(avg_price) = order.avg_fill_price {
                        println!("  Avg Fill Price: ${:.2}", avg_price);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error fetching orders: {:?}", e);
        }
    }

    // Also try to get historical orders
    println!("\n\nFetching historical orders (last 10)...");
    match client.get_history_orders("All", 10).await {
        Ok(history) => {
            if let Some(orders) = history.as_array() {
                println!("Found {} historical order(s)", orders.len());
                for order in orders.iter().take(3) {
                    if let Some(symbol) = order
                        .get("ticker")
                        .and_then(|t| t.get("symbol"))
                        .and_then(|s| s.as_str())
                    {
                        let action = order
                            .get("action")
                            .and_then(|a| a.as_str())
                            .unwrap_or("N/A");
                        let status = order
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("N/A");
                        let order_type = order
                            .get("orderType")
                            .and_then(|o| o.as_str())
                            .unwrap_or("N/A");
                        println!(
                            "  {} {} {} - Status: {}",
                            action, symbol, order_type, status
                        );
                    }
                }
            } else {
                println!("No historical orders found or unexpected format");
            }
        }
        Err(e) => {
            println!("Error fetching historical orders: {:?}", e);
        }
    }

    Ok(())
}
