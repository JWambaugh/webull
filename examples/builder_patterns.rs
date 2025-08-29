// Example demonstrating various builder patterns in the API
// Shows how to use builders for orders, bars, and news requests

use dotenv::dotenv;
use std::env;
use webull_unofficial::{
    error::{Result, WebullError},
    models::*,
    BarsRequestBuilder, NewsRequestBuilder, PlaceOrderRequest, WebullClient,
};

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

    // Login
    println!("Logging in...");
    client
        .login(&username, &password, None, None, None, None)
        .await?;

    // Find a ticker (e.g., AAPL)
    println!("\nFinding ticker AAPL...");
    let tickers = client.find_ticker("AAPL").await?;

    if let Some(ticker) = tickers.first() {
        let ticker_id = ticker.ticker_id;
        let ticker_id_str = ticker_id.to_string();

        // ============= Order Builder Examples =============
        println!("\n=== Order Builder Examples ===");

        // Get current quote
        let quote = client.get_quotes(&ticker_id_str).await?;
        println!("Current AAPL price: ${:.2}", quote.close);

        // Example: Build a limit buy order
        let limit_order = PlaceOrderRequest::limit(quote.close - 1.0)
            .ticker_id(ticker_id)
            .buy()
            .quantity(10.0)
            .time_in_force(TimeInForce::GoodTillCancel)
            .build()
            .map_err(|e| WebullError::InvalidRequest(e))?;

        println!("Built limit order: {:?}", limit_order);

        // ============= Bars/Candles Builder Examples =============
        println!("\n=== Bars Builder Examples ===");

        // Example 1: Simple bars request with builder (for demonstration)
        let _bars_request = BarsRequestBuilder::new()
            .ticker_id(&ticker_id_str)
            .interval("5m")
            .count(50)
            .build()
            .map_err(|e| WebullError::InvalidRequest(e))?;

        let bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("5m")
            .count(50)
            .await?;

        println!("Fetched {} 5-minute bars", bars.len());
        if let Some(latest_bar) = bars.first() {
            println!(
                "Latest bar - Open: ${:.2}, Close: ${:.2}, Volume: {:.0}",
                latest_bar.open, latest_bar.close, latest_bar.volume
            );
        }

        // Example 2: Fetch daily bars from specific date
        let daily_bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("1d")
            .count(30)
            .from_date(chrono::Utc::now() - chrono::Duration::days(30))
            .await?;

        println!("Fetched {} daily bars from last 30 days", daily_bars.len());

        // Example 3: Using the builder directly
        let custom_bars_request = BarsRequestBuilder::new()
            .ticker_id(&ticker_id_str)
            .interval("1h")
            .count(100)
            .timestamp(1609459200) // Specific timestamp
            .build()
            .map_err(|e| WebullError::InvalidRequest(e))?;

        println!("Custom bars request built: {:?}", custom_bars_request);

        // ============= News Builder Examples =============
        println!("\n=== News Builder Examples ===");

        // Example 1: Get latest news
        let latest_news = client.get_news_with().ticker("AAPL").latest(10).await?;

        println!("Fetched {} latest news items", latest_news.len());
        for (i, news) in latest_news.iter().take(3).enumerate() {
            println!("{}. {}", i + 1, news.title);
            if let Some(source) = &news.source_name {
                println!("   Source: {}", source);
            }
        }

        // Example 2: Paginated news request
        if let Some(last_news) = latest_news.last() {
            let more_news = client
                .get_news_with()
                .ticker("AAPL")
                .after(last_news.id)
                .count(5)
                .await?;

            println!(
                "\nFetched {} more news items after ID {}",
                more_news.len(),
                last_news.id
            );
        }

        // Example 3: Building news request with all options
        let custom_news_request = NewsRequestBuilder::new()
            .ticker("TSLA")
            .last_id(1000000)
            .count(25)
            .build()
            .map_err(|e| WebullError::InvalidRequest(e))?;

        println!("Custom news request built: {:?}", custom_news_request);

        // ============= Combining Builders =============
        println!("\n=== Combining Builders ===");

        // Fetch data and create order based on analysis
        let recent_bars = client
            .get_bars_with()
            .ticker_id(&ticker_id_str)
            .interval("15m")
            .count(10)
            .await?;

        if let Some(last_bar) = recent_bars.first() {
            // Create a limit order based on recent price action
            let analysis_order = PlaceOrderRequest::limit(last_bar.low - 0.50)
                .ticker_id(ticker_id)
                .buy()
                .quantity(5.0)
                .time_in_force(TimeInForce::Day)
                .build()
                .map_err(|e| WebullError::InvalidRequest(e))?;

            println!("Created order based on recent bars analysis:");
            println!("  Last bar low: ${:.2}", last_bar.low);
            println!(
                "  Order limit price: ${:.2}",
                analysis_order.limit_price.unwrap_or(0.0)
            );
        }
    } else {
        println!("Ticker AAPL not found");
    }

    Ok(())
}
