// Example demonstrating the builder pattern for placing orders
// This shows how to use the fluent API to construct orders

use dotenv::dotenv;
use std::env;
use webull_unofficial::{error::Result, models::*, PlaceOrderRequest, WebullClient};

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
    println!("Finding ticker AAPL...");
    let tickers = client.find_ticker("AAPL").await?;

    if let Some(ticker) = tickers.first() {
        let ticker_id = ticker.ticker_id;

        // Get current quote for reference
        let quote = client.get_quotes(&ticker_id.to_string()).await?;
        println!("Current AAPL price: ${:.2}", quote.close);

        // Example 1: Market Buy Order
        println!("\n=== Example 1: Market Buy Order ===");
        let market_order = PlaceOrderRequest::market()
            .ticker_id(ticker_id)
            .buy()
            .quantity(1.0)
            .time_in_force(TimeInForce::Day)
            .build()
            .expect("Failed to build market order");

        println!("Market Order: {:#?}", market_order);

        // Example 2: Limit Buy Order with extended hours
        println!("\n=== Example 2: Limit Buy Order ===");
        let limit_order = PlaceOrderRequest::limit(quote.close - 1.0)
            .ticker_id(ticker_id)
            .buy()
            .quantity(2.0)
            .time_in_force(TimeInForce::GoodTillCancel)
            .extended_hours() // Enable extended hours trading
            .build()
            .expect("Failed to build limit order");

        println!("Limit Order: {:#?}", limit_order);

        // Example 3: Stop Loss Order
        println!("\n=== Example 3: Stop Loss Order ===");
        let stop_order = PlaceOrderRequest::stop(quote.close - 5.0)
            .ticker_id(ticker_id)
            .sell()
            .quantity(1.0)
            .time_in_force(TimeInForce::GoodTillCancel)
            .build()
            .expect("Failed to build stop order");

        println!("Stop Order: {:#?}", stop_order);

        // Example 4: Stop-Limit Order
        println!("\n=== Example 4: Stop-Limit Order ===");
        let stop_limit_order = PlaceOrderRequest::stop_limit(
            quote.close - 5.0, // stop price
            quote.close - 5.5, // limit price
        )
        .ticker_id(ticker_id)
        .sell()
        .quantity(1.0)
        .time_in_force(TimeInForce::Day)
        .build()
        .expect("Failed to build stop-limit order");

        println!("Stop-Limit Order: {:#?}", stop_limit_order);

        // Example 5: Using the generic builder
        println!("\n=== Example 5: Generic Builder ===");
        let custom_order = PlaceOrderRequest::builder(OrderType::Limit)
            .ticker_id(ticker_id)
            .action(OrderAction::Buy)
            .quantity(5.0)
            .limit_price(quote.close - 2.0)
            .time_in_force(TimeInForce::ImmediateOrCancel)
            .serial_id(uuid::Uuid::new_v4().to_string())
            .build()
            .expect("Failed to build custom order");

        println!("Custom Order: {:#?}", custom_order);

        // Example 6: Actually place an order (commented out to avoid real trades)
        println!("\n=== Example 6: Place Order (Demo) ===");
        let demo_order = PlaceOrderRequest::limit(quote.close - 10.0) // Far from market price
            .ticker_id(ticker_id)
            .buy()
            .quantity(1.0)
            .build()
            .expect("Failed to build demo order");

        println!("Would place order: {:#?}", demo_order);

        // Uncomment to actually place the order:
        // let order_id = client.place_order(&demo_order).await?;
        // println!("Order placed successfully with ID: {}", order_id);
    } else {
        println!("Ticker AAPL not found");
    }

    Ok(())
}
