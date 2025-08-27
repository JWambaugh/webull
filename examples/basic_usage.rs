use dotenv::dotenv;
use std::env;
use webull::{error::Result, models::*, PaperWebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    env_logger::init();

    // Get credentials from environment
    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");
    println!("Using username: {}", username);
    // Create a paper trading client
    let mut client = PaperWebullClient::new(Some(6))?; // 6 = US region

    println!("Logging in...");

    // Login
    match client
        .login(&username, &password, None, None, None, None)
        .await
    {
        Ok(response) => {
            println!("Login successful!");
            println!("Access token: {}", response.access_token);
        }
        Err(e) => {
            eprintln!("Login failed: {}", e);
            return Err(e);
        }
    }

    // Get account information
    println!("\nFetching account details...");
    match client.get_account().await {
        Ok(account) => {
            println!("Account ID: {}", account.account_id);
            println!("Net Liquidation: ${:.2}", account.net_liquidation);
            println!("Total Cash: ${:.2}", account.total_cash);
            println!("Buying Power: ${:.2}", account.buying_power);
        }
        Err(e) => eprintln!("Failed to get account: {}", e),
    }

    // Get positions
    println!("\nFetching positions...");
    match client.get_positions().await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("No positions found");
            } else {
                for position in positions {
                    println!(
                        "Symbol: {}, Quantity: {}, P&L: ${:.2}",
                        position.ticker.symbol, position.position, position.unrealized_profit_loss
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to get positions: {}", e),
    }

    // Search for a ticker
    println!("\nSearching for AAPL...");
    match client.find_ticker("AAPL").await {
        Ok(tickers) => {
            if let Some(ticker) = tickers.first() {
                println!("Found: {} - {}", ticker.symbol, ticker.name);
                println!("Ticker ID: {}", ticker.ticker_id);

                // Get quotes for the ticker
                println!("\nGetting quotes for {}...", ticker.symbol);
                match client.get_quotes(&ticker.ticker_id.to_string()).await {
                    Ok(quote) => {
                        println!("Price: ${:.2}", quote.close);
                        println!(
                            "Change: ${:.2} ({:.2}%)",
                            quote.change,
                            quote.change_ratio * 100.0
                        );
                        println!("Volume: {}", quote.volume);
                    }
                    Err(e) => eprintln!("Failed to get quotes: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to search ticker: {}", e),
    }

    // Get recent orders
    println!("\nFetching recent orders...");
    match client.get_orders(Some(10)).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("No recent orders");
            } else {
                for order in orders.iter().take(5) {
                    println!(
                        "Order {}: {} {} shares of {} at ${:.2}",
                        order.order_id,
                        match order.action {
                            OrderAction::Buy => "BUY",
                            OrderAction::Sell => "SELL",
                        },
                        order.quantity,
                        order.ticker.symbol,
                        order.limit_price.unwrap_or(0.0)
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to get orders: {}", e),
    }

    // Logout
    println!("\nLogging out...");
    client.logout().await?;
    println!("Logged out successfully");

    Ok(())
}
