use dotenv::dotenv;
use std::env;
use uuid;
use webull::{error::Result, models::*, PaperWebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    // Create paper trading client
    let mut client = PaperWebullClient::new(Some(6))?;

    // Login
    println!("Logging in to paper trading account...");
    client
        .login(&username, &password, None, None, None, None)
        .await?;
    println!("Login successful!");

    // Get paper account details
    println!("\nFetching paper account details...");
    match client.get_account().await {
        Ok(account) => {
            println!("Paper Account ID: {}", account.account_id);
            println!("Net Liquidation: ${:.2}", account.net_liquidation);
            if let Some(total_cash) = account.total_cash {
                println!("Total Cash: ${:.2}", total_cash);
            }
            if let Some(buying_power) = account.buying_power {
                println!("Buying Power: ${:.2}", buying_power);
            }
            if let (Some(day_pl), Some(day_pl_rate)) =
                (account.day_profit_loss, account.day_profit_loss_rate)
            {
                println!("Day P&L: ${:.2} ({:.2}%)", day_pl, day_pl_rate * 100.0);
            }
        }
        Err(e) => eprintln!("Failed to get account: {}", e),
    }

    // Search for a ticker
    let symbol = "AAPL";
    println!("\nSearching for {}...", symbol);
    let tickers = client.find_ticker(symbol).await?;

    if let Some(ticker) = tickers.first() {
        println!("Found: {} - {}", ticker.symbol, ticker.name);

        // Get current quote
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        println!("Current price: ${:.2}", quote.close);

        // Create a limit order to buy 1 share slightly below current price
        let limit_price = quote.close - 1.0; // $1 below current price
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 1.0,
            limit_price: Some(limit_price),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: Some(uuid::Uuid::new_v4().to_string()),
            combo_type: None,
        };

        println!(
            "\nPlacing paper order: BUY 1 share of {} at limit ${:.2}",
            ticker.symbol, limit_price
        );

        match client.place_order(&order).await {
            Ok(order_id) => {
                println!("Paper order placed successfully! Order ID: {}", order_id);

                // Get recent orders
                println!("\nFetching paper orders...");
                match client.get_orders(Some(5)).await {
                    Ok(orders) => {
                        for order in orders.iter().take(3) {
                            let order_type_str = match order.order_type {
                                OrderType::Market => "MARKET".to_string(),
                                OrderType::Limit => {
                                    format!("LIMIT ${:.2}", order.limit_price.unwrap_or(0.0))
                                }
                                _ => "OTHER".to_string(),
                            };
                            println!(
                                "Order {}: {} {} @ {}",
                                order.order_id,
                                match order.action {
                                    OrderAction::Buy => "BUY",
                                    OrderAction::Sell => "SELL",
                                },
                                order.quantity,
                                order_type_str
                            );
                            println!("  Status: {:?}", order.status);
                        }
                    }
                    Err(e) => eprintln!("Failed to get orders: {}", e),
                }

                // Cancel the order (if still pending)
                println!("\nAttempting to cancel order...");
                if client.cancel_order(&order_id).await? {
                    println!("Paper order cancelled successfully!");
                } else {
                    println!("Could not cancel order (may already be filled)");
                }
            }
            Err(e) => eprintln!("Failed to place paper order: {}", e),
        }
    } else {
        println!("Ticker not found");
    }

    Ok(())
}
