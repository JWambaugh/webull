use dotenv::dotenv;
use serde_json::Value;
use std::env;
use tokio::time::{sleep, Duration};
use webull_unofficial::{
    stream::{StreamConfig, TopicTypes},
    PaperWebullClient, StreamConn,
};

#[tokio::main]
async fn main() -> webull_unofficial::error::Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    // Login to get access token (using paper trading client)
    let mut client = PaperWebullClient::new(Some(6))?;
    println!("Logging in...");
    let login_response = client
        .login(&username, &password, None, None, None, None)
        .await?;
    println!("Login successful!");

    // Create streaming connection
    let config = StreamConfig {
        debug: true,
        ..Default::default()
    };

    let mut stream = StreamConn::new(Some(config));

    // Set up callbacks for price updates
    stream.set_price_callback(|topic: Value, data: Value| {
        println!("Price Update:");
        println!("  Topic: {}", topic);

        if let Some(ticker_id) = topic.get("tickerId") {
            println!("  Ticker ID: {}", ticker_id);
        }

        if let Some(price) = data.get("close") {
            println!("  Price: {}", price);
        }

        if let Some(volume) = data.get("volume") {
            println!("  Volume: {}", volume);
        }

        if let Some(change) = data.get("changeRatio") {
            if let Some(change_val) = change.as_f64() {
                println!("  Change: {:.2}%", change_val * 100.0);
            }
        }

        println!("---");
    });

    // Set up callbacks for order updates
    stream.set_order_callback(|topic: Value, data: Value| {
        println!("Order Update:");
        println!("  Topic: {}", topic);

        if let Some(order_id) = data.get("orderId") {
            println!("  Order ID: {}", order_id);
        }

        if let Some(status) = data.get("orderStatus") {
            println!("  Status: {}", status);
        }

        if let Some(filled) = data.get("filledQuantity") {
            println!("  Filled Quantity: {}", filled);
        }

        println!("---");
    });

    // Connect to streaming service
    println!("\nConnecting to streaming service...");
    let access_token = &login_response.access_token;
    let did = client.get_did().to_string();

    stream.connect(access_token, &did).await?;
    println!("Connected to streaming service!");

    // Search for tickers to subscribe to
    let symbols = vec!["AAPL", "TSLA", "SPY"];

    for symbol in &symbols {
        println!("\nSearching for {}...", symbol);
        if let Ok(tickers) = client.find_ticker(symbol).await {
            if let Some(ticker) = tickers.first() {
                println!(
                    "Subscribing to {} (ID: {})",
                    ticker.symbol, ticker.ticker_id
                );

                // Subscribe to basic topics (quote, trade, book)
                stream
                    .subscribe_ticker(&ticker.ticker_id.to_string(), TopicTypes::basic())
                    .await?;
            }
        }
    }

    // Subscribe to order updates
    if let Some(account_id) = client.get_account_id_str() {
        println!("\nSubscribing to order updates for account: {}", account_id);
        stream.subscribe_orders(&account_id).await?;
    }

    // Keep the stream running for 60 seconds
    println!("\nStreaming data for 60 seconds...");
    println!("Press Ctrl+C to stop\n");

    for i in 0..60 {
        sleep(Duration::from_secs(1)).await;

        if i % 10 == 0 {
            println!("Active subscriptions: {:?}", stream.get_subscriptions());
        }
    }

    // Cleanup
    println!("\nDisconnecting...");
    stream.disconnect().await?;
    client.logout().await?;
    println!("Done!");

    Ok(())
}
