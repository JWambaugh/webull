// Example demonstrating automatic order type detection
//
// This example shows how the place_order_with() builder
// automatically detects the order type based on parameters:
// - No prices → Market order
// - limit() only → Limit order
// - stop() only → Stop order
// - limit() + stop() → Stop-Limit order

use std::env;
use webull_unofficial::{models::*, Result, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    // Create paper trading client for testing
    let mut client = WebullClient::new_paper(Some(6))?;

    // Login
    client
        .login_with()
        .username(&username)
        .password(&password)
        .await?;

    println!("✓ Logged in to paper account");

    // Find a ticker
    let tickers = client.find_ticker("AAPL").await?;
    if let Some(ticker) = tickers.first() {
        let ticker_id = ticker.ticker_id;
        println!("\n📊 Ticker found: {} (ID: {})", ticker.symbol, ticker_id);

        // Get current price for reference
        let quote = client.get_quotes(&ticker_id.to_string()).await?;
        println!("Current price: ${:.2}", quote.close);

        println!("\n🎯 Demonstrating automatic order type detection:");

        // Example 1: Market order (no prices specified)
        println!("\n1. Market Order (no prices):");
        println!("   client.place_order_with()");
        println!("       .ticker_id(ticker_id)");
        println!("       .buy()");
        println!("       .quantity(1.0)");
        println!("       .await?");
        println!("   → Detects: MARKET order");

        // Example 2: Limit order (only limit price specified)
        println!("\n2. Limit Order (limit price only):");
        println!("   client.place_order_with()");
        println!("       .ticker_id(ticker_id)");
        println!("       .limit(150.0)  // Sets limit price");
        println!("       .buy()");
        println!("       .quantity(1.0)");
        println!("       .await?");
        println!("   → Detects: LIMIT order at $150.00");

        // Example 3: Stop order (only stop price specified)
        println!("\n3. Stop Order (stop price only):");
        println!("   client.place_order_with()");
        println!("       .ticker_id(ticker_id)");
        println!("       .stop(140.0)   // Sets stop price");
        println!("       .sell()");
        println!("       .quantity(1.0)");
        println!("       .await?");
        println!("   → Detects: STOP order at $140.00");

        // Example 4: Stop-Limit order (both prices specified)
        println!("\n4. Stop-Limit Order (both prices):");
        println!("   client.place_order_with()");
        println!("       .ticker_id(ticker_id)");
        println!("       .stop(140.0)   // Sets stop price");
        println!("       .limit(139.0)  // Sets limit price");
        println!("       .sell()");
        println!("       .quantity(1.0)");
        println!("       .await?");
        println!("   → Detects: STOP-LIMIT order (stop: $140.00, limit: $139.00)");

        // Actually place a test order (paper trading)
        println!("\n📝 Placing a test limit order (paper trading)...");

        match client
            .place_order_with()
            .ticker_id(ticker_id)
            .limit(quote.close * 0.95) // 5% below current price
            .buy()
            .quantity(1.0)
            .time_in_force(TimeInForce::Day)
            .await
        {
            Ok(order_id) => {
                println!("✅ Order placed successfully!");
                println!("   Order ID: {}", order_id);
                println!("   Type: AUTO-DETECTED as LIMIT");
                println!("   Price: ${:.2}", quote.close * 0.95);
            }
            Err(e) => {
                println!("❌ Failed to place order: {}", e);
            }
        }

        // Show alternative: You can still explicitly specify order type
        println!("\n💡 Note: You can still use explicit order type methods:");
        println!("   • client.place_market_order_with()");
        println!("   • client.place_limit_order_with(price)");
        println!("   • client.place_stop_order_with(price)");
        println!("   • client.place_stop_limit_order_with(stop, limit)");
    } else {
        println!("❌ Ticker not found");
    }

    Ok(())
}
