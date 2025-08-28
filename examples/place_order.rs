use webull_unofficial::{PaperWebullClient, models::*, error::Result};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");
    let trade_password = env::var("WEBULL_TRADE_PASSWORD")
        .unwrap_or_else(|_| password.clone());

    let mut client = PaperWebullClient::new(Some(6))?;

    // Login
    println!("Logging in...");
    client.login(&username, &password, None, None, None, None).await?;
    println!("Login successful!");

    // Get trade token (required for placing orders)
    println!("Getting trade token...");
    client.get_trade_token(&trade_password).await?;
    println!("Trade token obtained!");

    // Search for a ticker
    let symbol = "AAPL";
    println!("\nSearching for {}...", symbol);
    let tickers = client.find_ticker(symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        println!("Found: {} - {}", ticker.symbol, ticker.name);
        
        // Get current quote
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        println!("Current price: ${:.2}", quote.close);
        
        // Create a limit order to buy 1 share
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 1.0,
            limit_price: Some(quote.close - 1.0), // $1 below current price
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        println!("\nPlacing order: BUY 1 share of {} at ${:.2}", 
            ticker.symbol, 
            order.limit_price.unwrap()
        );
        
        match client.place_order(&order).await {
            Ok(order_id) => {
                println!("Order placed successfully! Order ID: {}", order_id);
                
                // Optionally cancel the order
                println!("\nCancelling order...");
                if client.cancel_order(&order_id).await? {
                    println!("Order cancelled successfully!");
                } else {
                    println!("Failed to cancel order");
                }
            }
            Err(e) => eprintln!("Failed to place order: {}", e),
        }
    } else {
        println!("Ticker not found");
    }

    client.logout().await?;
    Ok(())
}