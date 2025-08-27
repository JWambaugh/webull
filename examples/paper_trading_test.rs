use webull::{PaperWebullClient, models::*, error::Result};
use dotenv::dotenv;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use log::{error, warn};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("=====================================");
    println!("   Webull Paper Trading Test Suite  ");
    println!("=====================================\n");

    let (username, password) = get_credentials();

    let mut client = PaperWebullClient::new(Some(6))?;

    println!("📊 Logging in to paper trading account...");
    match client.login(&username, &password, None, None, None, None).await {
        Ok(_) => println!("✅ Login successful!\n"),
        Err(e) => {
            error!("❌ Login failed: {}", e);
            return Err(e);
        }
    }

    loop {
        display_menu();
        
        let choice = get_user_input("Enter your choice: ");
        
        match choice.trim() {
            "1" => display_account_info(&client).await?,
            "2" => get_quote_interactive(&client).await?,
            "3" => get_historical_data_interactive(&client).await?,
            "4" => place_market_order_interactive(&client).await?,
            "5" => place_limit_order_interactive(&client).await?,
            "6" => place_stop_order_interactive(&client).await?,
            "7" => display_current_orders(&client).await?,
            "8" => cancel_order_interactive(&client).await?,
            "9" => analyze_portfolio(&client).await?,
            "10" => get_news_interactive(&client).await?,
            "11" => run_automated_test_suite(&client).await?,
            "0" | "q" | "Q" => {
                println!("\n👋 Thank you for using Webull Paper Trading Test Suite!");
                break;
            }
            _ => println!("❌ Invalid choice. Please try again."),
        }
        
        if choice.trim() != "0" && choice.trim() != "q" && choice.trim() != "Q" {
            println!("\nPress Enter to continue...");
            let _ = get_user_input("");
        }
    }

    Ok(())
}

fn display_menu() {
    println!("\n=====================================");
    println!("           MAIN MENU                 ");
    println!("=====================================");
    println!("1.  View Account Information");
    println!("2.  Get Stock Quote");
    println!("3.  Get Historical Data");
    println!("4.  Place Market Order");
    println!("5.  Place Limit Order");
    println!("6.  Place Stop-Loss Order");
    println!("7.  View Current Orders");
    println!("8.  Cancel Order");
    println!("9.  Analyze Portfolio");
    println!("10. Get Market News");
    println!("11. Run Automated Test Suite");
    println!("0.  Exit");
    println!("=====================================");
}

fn get_credentials() -> (String, String) {
    let username = match env::var("WEBULL_USERNAME") {
        Ok(u) => u,
        Err(_) => {
            println!("WEBULL_USERNAME not found in environment.");
            get_user_input("Please enter your Webull username/email: ")
        }
    };

    let password = match env::var("WEBULL_PASSWORD") {
        Ok(p) => p,
        Err(_) => {
            println!("WEBULL_PASSWORD not found in environment.");
            get_password("Please enter your Webull password: ")
        }
    };

    (username, password)
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn get_password(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let password = rpassword::read_password().unwrap_or_else(|_| {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    });
    
    password
}

fn confirm_action(action: &str) -> bool {
    let response = get_user_input(&format!("⚠️  {} Confirm? (y/n): ", action));
    response.to_lowercase() == "y" || response.to_lowercase() == "yes"
}

async fn display_account_info(client: &PaperWebullClient) -> Result<()> {
    println!("\n💰 Account Information");
    println!("─────────────────────────");
    
    match client.get_account().await {
        Ok(account) => {
            println!("Account ID: {}", account.account_id);
            println!("Net Liquidation: ${:.2}", account.net_liquidation);
            if let Some(total_cash) = account.total_cash {
                println!("Total Cash: ${:.2}", total_cash);
            }
            if let Some(buying_power) = account.buying_power {
                println!("Buying Power: ${:.2}", buying_power);
            }
            if let Some(total_market_value) = account.total_market_value {
                println!("Total Market Value: ${:.2}", total_market_value);
            }
            if let (Some(day_pl), Some(day_pl_rate)) = (account.day_profit_loss, account.day_profit_loss_rate) {
                println!("Day P&L: ${:.2} ({:.2}%)", day_pl, day_pl_rate * 100.0);
            }
            if let (Some(total_pl), Some(total_pl_rate)) = (account.total_profit_loss, account.total_profit_loss_rate) {
                println!("Total P&L: ${:.2} ({:.2}%)", total_pl, total_pl_rate * 100.0);
            }
        }
        Err(e) => {
            error!("Failed to get account details: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

async fn get_quote_interactive(client: &PaperWebullClient) -> Result<()> {
    let symbol = get_user_input("Enter stock symbol (e.g., AAPL): ").to_uppercase();
    
    println!("\n🔍 Fetching quote for {}...", symbol);
    
    let tickers = client.find_ticker(&symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        
        println!("\n📊 Quote for {} - {}", ticker.symbol, ticker.name);
        println!("────────────────────────────");
        println!("Current Price: ${:.2}", quote.close);
        println!("Change: ${:.2} ({:.2}%)", 
            quote.close - quote.pre_close, 
            ((quote.close - quote.pre_close) / quote.pre_close) * 100.0
        );
        println!("Volume: {}", quote.volume);
        println!("Day Range: ${:.2} - ${:.2}", quote.low, quote.high);
        println!("Previous Close: ${:.2}", quote.pre_close);
        if let Some(market_value) = quote.market_value {
            println!("Market Cap: ${:.2}M", market_value / 1_000_000.0);
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }
    
    Ok(())
}

async fn get_historical_data_interactive(client: &PaperWebullClient) -> Result<()> {
    let symbol = get_user_input("Enter stock symbol (e.g., AAPL): ").to_uppercase();
    let days = get_user_input("Number of days to fetch (default 10): ");
    let days = days.parse::<i32>().unwrap_or(10);
    
    println!("\n📊 Fetching {} days of historical data for {}...", days, symbol);
    
    let tickers = client.find_ticker(&symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        let bars = client.get_bars(&ticker.ticker_id.to_string(), "1d", days, None).await?;
        
        println!("\nHistorical Data for {}:", ticker.symbol);
        println!("────────────────────────────");
        for (i, bar) in bars.iter().enumerate().take(days as usize) {
            println!("Day {}: Open: ${:.2}, High: ${:.2}, Low: ${:.2}, Close: ${:.2}, Volume: {:.0}",
                i + 1, bar.open, bar.high, bar.low, bar.close, bar.volume
            );
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }
    
    Ok(())
}

async fn place_market_order_interactive(client: &PaperWebullClient) -> Result<()> {
    println!("\n🛒 Place Market Order");
    println!("─────────────────────");
    
    let symbol = get_user_input("Enter stock symbol: ").to_uppercase();
    let action = get_user_input("Buy or Sell? (B/S): ").to_uppercase();
    let quantity = get_user_input("Enter quantity: ");
    
    let action = if action.starts_with('B') {
        OrderAction::Buy
    } else if action.starts_with('S') {
        OrderAction::Sell
    } else {
        println!("❌ Invalid action. Must be B (Buy) or S (Sell)");
        return Ok(());
    };
    
    let quantity = match quantity.parse::<f64>() {
        Ok(q) if q > 0.0 => q,
        _ => {
            println!("❌ Invalid quantity");
            return Ok(());
        }
    };
    
    let tickers = client.find_ticker(&symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        
        let action_str = match action {
            OrderAction::Buy => "BUY",
            OrderAction::Sell => "SELL",
        };
        
        println!("\n📋 Order Summary:");
        println!("  Action: {} {} shares of {}", action_str, quantity, ticker.symbol);
        println!("  Current Price: ${:.2}", quote.close);
        println!("  Estimated Total: ${:.2}", quote.close * quantity);
        
        if !confirm_action(&format!("Place this MARKET order for {} shares of {}", quantity, ticker.symbol)) {
            println!("❌ Order cancelled by user");
            return Ok(());
        }
        
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            quantity,
            limit_price: None,
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        match client.place_order(&order).await {
            Ok(order_id) => {
                println!("✅ Market order placed successfully!");
                println!("   Order ID: {}", order_id);
            }
            Err(e) => {
                error!("❌ Failed to place market order: {}", e);
            }
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }
    
    Ok(())
}

async fn place_limit_order_interactive(client: &PaperWebullClient) -> Result<()> {
    println!("\n💰 Place Limit Order");
    println!("────────────────────");
    
    let symbol = get_user_input("Enter stock symbol: ").to_uppercase();
    let action = get_user_input("Buy or Sell? (B/S): ").to_uppercase();
    let quantity = get_user_input("Enter quantity: ");
    let limit_price = get_user_input("Enter limit price: $");
    
    let action = if action.starts_with('B') {
        OrderAction::Buy
    } else if action.starts_with('S') {
        OrderAction::Sell
    } else {
        println!("❌ Invalid action. Must be B (Buy) or S (Sell)");
        return Ok(());
    };
    
    let quantity = match quantity.parse::<f64>() {
        Ok(q) if q > 0.0 => q,
        _ => {
            println!("❌ Invalid quantity");
            return Ok(());
        }
    };
    
    let limit_price = match limit_price.parse::<f64>() {
        Ok(p) if p > 0.0 => p,
        _ => {
            println!("❌ Invalid limit price");
            return Ok(());
        }
    };
    
    let tickers = client.find_ticker(&symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        
        let action_str = match action {
            OrderAction::Buy => "BUY",
            OrderAction::Sell => "SELL",
        };
        
        println!("\n📋 Order Summary:");
        println!("  Action: {} {} shares of {}", action_str, quantity, ticker.symbol);
        println!("  Current Price: ${:.2}", quote.close);
        println!("  Limit Price: ${:.2}", limit_price);
        println!("  Max Total: ${:.2}", limit_price * quantity);
        
        if !confirm_action(&format!("Place this LIMIT order for {} shares of {} at ${:.2}", 
            quantity, ticker.symbol, limit_price)) {
            println!("❌ Order cancelled by user");
            return Ok(());
        }
        
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GoodTillCancel,
            quantity,
            limit_price: Some(limit_price),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        match client.place_order(&order).await {
            Ok(order_id) => {
                println!("✅ Limit order placed successfully!");
                println!("   Order ID: {}", order_id);
            }
            Err(e) => {
                error!("❌ Failed to place limit order: {}", e);
            }
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }
    
    Ok(())
}

async fn place_stop_order_interactive(client: &PaperWebullClient) -> Result<()> {
    println!("\n🛡️ Place Stop-Loss Order");
    println!("─────────────────────────");
    
    let symbol = get_user_input("Enter stock symbol: ").to_uppercase();
    let quantity = get_user_input("Enter quantity to sell: ");
    let stop_price = get_user_input("Enter stop price: $");
    
    let quantity = match quantity.parse::<f64>() {
        Ok(q) if q > 0.0 => q,
        _ => {
            println!("❌ Invalid quantity");
            return Ok(());
        }
    };
    
    let stop_price = match stop_price.parse::<f64>() {
        Ok(p) if p > 0.0 => p,
        _ => {
            println!("❌ Invalid stop price");
            return Ok(());
        }
    };
    
    let tickers = client.find_ticker(&symbol).await?;
    
    if let Some(ticker) = tickers.first() {
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        
        println!("\n📋 Order Summary:");
        println!("  Action: SELL {} shares of {}", quantity, ticker.symbol);
        println!("  Current Price: ${:.2}", quote.close);
        println!("  Stop Price: ${:.2}", stop_price);
        println!("  Will trigger when price drops to: ${:.2}", stop_price);
        
        if !confirm_action(&format!("Place this STOP order for {} shares of {} at ${:.2}", 
            quantity, ticker.symbol, stop_price)) {
            println!("❌ Order cancelled by user");
            return Ok(());
        }
        
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Sell,
            order_type: OrderType::Stop,
            time_in_force: TimeInForce::GoodTillCancel,
            quantity,
            limit_price: None,
            stop_price: Some(stop_price),
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        match client.place_order(&order).await {
            Ok(order_id) => {
                println!("✅ Stop-loss order placed successfully!");
                println!("   Order ID: {}", order_id);
            }
            Err(e) => {
                error!("❌ Failed to place stop-loss order: {}", e);
            }
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }
    
    Ok(())
}

async fn display_current_orders(client: &PaperWebullClient) -> Result<()> {
    println!("\n📋 Current Orders");
    println!("─────────────────");
    
    match client.get_orders(Some(20)).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("No active orders");
            } else {
                for (i, order) in orders.iter().enumerate() {
                    let order_type_str = match order.order_type {
                        OrderType::Market => "MARKET",
                        OrderType::Limit => "LIMIT",
                        OrderType::Stop => "STOP",
                        OrderType::StopLimit => "STOP_LIMIT",
                    };
                    
                    let action_str = match order.action {
                        OrderAction::Buy => "BUY",
                        OrderAction::Sell => "SELL",
                    };
                    
                    println!("\n{}. Order ID: {}", i + 1, order.order_id);
                    println!("   Type: {} {} @ {}", action_str, order.quantity, order_type_str);
                    if let Some(limit) = order.limit_price {
                        println!("   Limit Price: ${:.2}", limit);
                    }
                    if let Some(stop) = order.stop_price {
                        println!("   Stop Price: ${:.2}", stop);
                    }
                    println!("   Status: {:?}", order.status);
                    println!("   Filled: {}/{}", order.filled_quantity, order.quantity);
                }
            }
        }
        Err(e) => {
            error!("Failed to get orders: {}", e);
        }
    }
    
    Ok(())
}

async fn cancel_order_interactive(client: &PaperWebullClient) -> Result<()> {
    println!("\n🔄 Cancel Order");
    println!("───────────────");
    
    match client.get_orders(Some(20)).await {
        Ok(orders) => {
            let pending_orders: Vec<_> = orders.iter()
                .filter(|o| matches!(o.status, OrderStatus::Pending | OrderStatus::PartialFilled))
                .collect();
            
            if pending_orders.is_empty() {
                println!("No pending orders to cancel");
                return Ok(());
            }
            
            println!("\nPending Orders:");
            for (i, order) in pending_orders.iter().enumerate() {
                let action_str = match order.action {
                    OrderAction::Buy => "BUY",
                    OrderAction::Sell => "SELL",
                };
                println!("{}. {} {} shares - Order ID: {}", 
                    i + 1, action_str, order.quantity, order.order_id);
            }
            
            let choice = get_user_input("\nEnter order number to cancel (0 to go back): ");
            
            if let Ok(idx) = choice.parse::<usize>() {
                if idx == 0 {
                    return Ok(());
                }
                
                if let Some(order) = pending_orders.get(idx - 1) {
                    if confirm_action(&format!("Cancel order {}", order.order_id)) {
                        match client.cancel_order(&order.order_id).await {
                            Ok(success) => {
                                if success {
                                    println!("✅ Order {} cancelled successfully", order.order_id);
                                } else {
                                    println!("⚠️ Could not cancel order {} (may already be filled)", order.order_id);
                                }
                            }
                            Err(e) => {
                                error!("❌ Failed to cancel order {}: {}", order.order_id, e);
                            }
                        }
                    } else {
                        println!("Cancellation aborted");
                    }
                } else {
                    println!("❌ Invalid selection");
                }
            } else {
                println!("❌ Invalid input");
            }
        }
        Err(e) => {
            error!("Failed to get orders: {}", e);
        }
    }
    
    Ok(())
}

async fn analyze_portfolio(client: &PaperWebullClient) -> Result<()> {
    println!("\n📊 Portfolio Analysis");
    println!("─────────────────────");
    
    match client.get_account().await {
        Ok(account) => {
            let total_value = account.net_liquidation;
            let cash_percentage = account.total_cash.map(|c| (c / total_value) * 100.0).unwrap_or(0.0);
            let invested_percentage = account.total_market_value.map(|m| (m / total_value) * 100.0).unwrap_or(0.0);
            
            println!("\nPortfolio Allocation:");
            if let Some(total_cash) = account.total_cash {
                println!("  Cash: ${:.2} ({:.1}%)", total_cash, cash_percentage);
            }
            if let Some(total_market_value) = account.total_market_value {
                println!("  Invested: ${:.2} ({:.1}%)", total_market_value, invested_percentage);
            }
            
            println!("\nPerformance Metrics:");
            if let (Some(day_pl), Some(day_pl_rate)) = (account.day_profit_loss, account.day_profit_loss_rate) {
                println!("  Day P&L: ${:.2} ({:.2}%)", day_pl, day_pl_rate * 100.0);
            }
            if let (Some(total_pl), Some(total_pl_rate)) = (account.total_profit_loss, account.total_profit_loss_rate) {
                println!("  Total P&L: ${:.2} ({:.2}%)", total_pl, total_pl_rate * 100.0);
            }
            
            if let Some(day_pl) = account.day_profit_loss {
                if day_pl > 0.0 {
                    println!("\n  📈 Positive day performance!");
                } else if day_pl < 0.0 {
                    println!("\n  📉 Negative day performance");
                } else {
                    println!("\n  ➡️ Flat day performance");
                }
            }
            
            println!("\nRisk Metrics:");
            if let Some(buying_power) = account.buying_power {
                println!("  Buying Power: ${:.2}", buying_power);
            }
            if let Some(margin) = account.margin {
                println!("  Margin Used: ${:.2}", margin);
            }
            if let Some(unsettled_cash) = account.unsettled_cash {
                println!("  Unsettled Cash: ${:.2}", unsettled_cash);
            }
        }
        Err(e) => {
            error!("Failed to analyze portfolio: {}", e);
        }
    }
    
    Ok(())
}

async fn get_news_interactive(client: &PaperWebullClient) -> Result<()> {
    let symbol = get_user_input("Enter stock symbol for news (e.g., AAPL): ").to_uppercase();
    
    println!("\n📰 Fetching news for {}...", symbol);
    
    match client.get_news(&symbol, 0, 5).await {
        Ok(news_items) => {
            if news_items.is_empty() {
                println!("No recent news found for {}", symbol);
            } else {
                println!("\nLatest News for {}:", symbol);
                println!("────────────────────────");
                
                for (i, news) in news_items.iter().enumerate() {
                    println!("\n{}. {}", i + 1, news.title);
                    let truncated = if news.summary.len() > 200 {
                        format!("{}...", &news.summary[..200])
                    } else {
                        news.summary.clone()
                    };
                    println!("   {}", truncated);
                    println!("   Source: {} | Time: {}", 
                        news.source,
                        news.news_time.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }
        Err(e) => {
            warn!("Failed to fetch news: {}", e);
        }
    }
    
    Ok(())
}

async fn run_automated_test_suite(client: &PaperWebullClient) -> Result<()> {
    println!("\n🤖 Running Automated Test Suite");
    println!("════════════════════════════════");
    
    if !confirm_action("This will place several test orders. Continue?") {
        println!("Test suite cancelled");
        return Ok(());
    }
    
    println!("\n1️⃣ Testing Account Info...");
    display_account_info(client).await?;
    sleep(Duration::from_secs(1)).await;
    
    println!("\n2️⃣ Testing Quote Retrieval...");
    let test_symbols = vec!["AAPL", "MSFT", "GOOGL"];
    for symbol in test_symbols {
        println!("\nFetching quote for {}...", symbol);
        if let Ok(tickers) = client.find_ticker(symbol).await {
            if let Some(ticker) = tickers.first() {
                if let Ok(quote) = client.get_quotes(&ticker.ticker_id.to_string()).await {
                    println!("  {} - Price: ${:.2}, Change: {:.2}%", 
                        ticker.symbol, quote.close, quote.change_ratio * 100.0);
                }
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("\n3️⃣ Testing Order Placement...");
    if confirm_action("Place a test MARKET BUY order for 1 share of AAPL?") {
        if let Ok(tickers) = client.find_ticker("AAPL").await {
            if let Some(ticker) = tickers.first() {
                let order = PlaceOrderRequest {
                    ticker_id: ticker.ticker_id,
                    action: OrderAction::Buy,
                    order_type: OrderType::Market,
                    time_in_force: TimeInForce::Day,
                    quantity: 1.0,
                    limit_price: None,
                    stop_price: None,
                    outside_regular_trading_hour: false,
                    serial_id: None,
                    combo_type: None,
                };
                
                match client.place_order(&order).await {
                    Ok(order_id) => println!("  ✅ Test order placed! ID: {}", order_id),
                    Err(e) => println!("  ❌ Test order failed: {}", e),
                }
            }
        }
    }
    
    println!("\n✅ Automated test suite complete!");
    
    Ok(())
}