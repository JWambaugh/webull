use dotenv::dotenv;
use std::env;
use webull::{error::Result, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    println!("Logging in to paper trading account...");
    let mut client = WebullClient::new_paper(Some(6))?;
    client.login(&username, &password, None, None, None, None).await?;
    
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
                    println!("  Symbol: {}", order.ticker.symbol);
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
    println!("\n\nFetching historical orders...");
    match client.get_history_orders("All", 10).await {
        Ok(history) => {
            println!("Historical orders response: {}", 
                serde_json::to_string_pretty(&history).unwrap_or_else(|_| "Failed to format".to_string()));
        }
        Err(e) => {
            println!("Error fetching historical orders: {:?}", e);
        }
    }
    
    Ok(())
}