// Example demonstrating the new fluent async builder API
// Shows how builders can be awaited directly without explicit build() calls

use dotenv::dotenv;
use std::env;
use webull_unofficial::{error::Result, models::*, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Get credentials from environment
    let username =
        env::var("WEBULL_USERNAME").expect("Please set WEBULL_USERNAME environment variable");
    let password =
        env::var("WEBULL_PASSWORD").expect("Please set WEBULL_PASSWORD environment variable");

    // Create paper trading client
    let mut client = WebullClient::new_paper(Some(6))?;

    // Login using new fluent builder API
    println!("Logging in with fluent API...");
    client
        .login_with()
        .username(&username)
        .password(&password)
        .await?;

    // Note: You can also still use the original API:
    // client.login(&username, &password, None, None, None, None).await?;
    // Or the LoginRequestBuilder with login_with_builder:
    // client.login_with_builder(LoginRequestBuilder::new().username(&username).password(&password)).await?;

    // Find a ticker
    println!("\nFinding ticker AAPL...");
    let tickers = client.find_ticker("AAPL").await?;

    if let Some(ticker) = tickers.first() {
        let ticker_id = ticker.ticker_id;
        let ticker_id_str = ticker_id.to_string();

        // Get current quote
        let quote = client.get_quotes(&ticker_id_str).await?;
        println!("Current AAPL price: ${:.2}", quote.close);

        // ============= NEW FLUENT API EXAMPLES =============
        println!("\n=== Fluent API Examples ===");

        // Example 1: Get bars directly with fluent API
        println!("\n1. Getting 5-minute bars with fluent API:");
        let bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("5m")
            .count(50)
            .await?; // <-- Directly await the builder!

        println!("   Fetched {} bars", bars.len());
        if let Some(latest) = bars.first() {
            println!(
                "   Latest bar - Open: ${:.2}, Close: ${:.2}",
                latest.open, latest.close
            );
        }

        // Example 2: Get bars from a specific date
        println!("\n2. Getting daily bars from 30 days ago:");
        let daily_bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("1d")
            .count(30)
            .from_date(chrono::Utc::now() - chrono::Duration::days(30))
            .await?;

        println!("   Fetched {} daily bars", daily_bars.len());

        // Example 3: Get news with fluent API
        println!("\n3. Getting latest news:");
        let news = client.get_news_with().ticker("AAPL").latest(5).await?; // <-- Directly await!

        println!("   Found {} news items:", news.len());
        for (i, item) in news.iter().enumerate() {
            println!("   {}. {}", i + 1, item.title);
        }

        // Example 4: Paginate through news
        if let Some(last_news) = news.last() {
            println!("\n4. Getting more news (pagination):");
            let more_news = client
                .get_news_with()
                .ticker("AAPL")
                .after(last_news.id)
                .count(3)
                .await?;

            println!(
                "   Found {} more news items after ID {}",
                more_news.len(),
                last_news.id
            );
        }

        // Example 5: Get options with fluent API
        println!("\n5. Getting options:");
        let options = client
            .get_options_with()
            .ticker("AAPL")
            .calls_only()
            .near_the_money(quote.close, 5.0) // Within 5% of current price
            .await?;

        println!("   Found {} call options near the money", options.len());

        // Example 6: Place orders with fluent API
        println!("\n6. Building orders with fluent API:");

        // Market order - directly await
        println!("   Market order (demo - not placing):");
        let _market_order_id = client.place_market_order_with()
            .ticker_id(ticker_id)
            .buy()
            .quantity(1.0)
            // .await?;  // <-- Would place the order if uncommented
            ;
        println!("   ✓ Market order builder created");

        // Limit order - directly await
        println!("   Limit order (demo - not placing):");
        let _limit_order_id = client.place_limit_order_with(quote.close - 10.0)
            .ticker_id(ticker_id)
            .sell()
            .quantity(2.0)
            .extended_hours()
            .time_in_force(TimeInForce::GoodTillCancel)
            // .await?;  // <-- Would place the order if uncommented
            ;
        println!("   ✓ Limit order builder created");

        // Stop-limit order - directly await
        println!("   Stop-limit order (demo - not placing):");
        let _stop_limit_order_id = client.place_stop_limit_order_with(quote.close - 5.0, quote.close - 5.5)
            .ticker_id(ticker_id)
            .sell()
            .quantity(1.0)
            // .await?;  // <-- Would place the order if uncommented
            ;
        println!("   ✓ Stop-limit order builder created");

        // Example 7: Chain multiple operations
        println!("\n7. Chaining operations:");

        // Get recent bars and immediately fetch news if price moved significantly
        let recent_bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("15m")
            .count(10)
            .await?;

        if let (Some(first), Some(last)) = (recent_bars.last(), recent_bars.first()) {
            let price_change = ((last.close - first.close) / first.close * 100.0).abs();
            println!("   Price change in last 10 bars: {:.2}%", price_change);

            if price_change > 1.0 {
                println!("   Significant movement detected, fetching news...");
                let related_news = client.get_news_with().ticker("AAPL").latest(3).await?;
                println!(
                    "   Found {} potentially related news items",
                    related_news.len()
                );
            }
        }
    } else {
        println!("Ticker AAPL not found");
    }

    println!("\n✅ All fluent API examples completed successfully!");

    Ok(())
}
