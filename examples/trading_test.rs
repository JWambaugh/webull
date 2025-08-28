use chrono;
use dotenv::dotenv;
use log::{error, warn};
use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use webull_unofficial::{error::Result, models::*, WebullClient};

// Interactive trading test suite

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("=====================================");
    println!("      Webull Trading Test Suite      ");
    println!("=====================================\n");

    // Ask user for trading mode
    println!("Select trading mode:");
    println!("1. Paper Trading (simulated)");
    println!("2. Live Trading (real money)");
    println!();

    let mode_choice = get_user_input("Enter your choice (1 or 2): ");
    let is_paper = match mode_choice.trim() {
        "1" => true,
        "2" => {
            println!("\n⚠️  WARNING: Live trading uses REAL MONEY!");
            let confirm = get_user_input("Type 'CONFIRM' to proceed with live trading: ");
            if confirm.trim() != "CONFIRM" {
                println!("Live trading cancelled. Switching to paper trading.");
                true
            } else {
                false
            }
        }
        _ => {
            println!("Invalid choice. Defaulting to paper trading.");
            true
        }
    };

    let (username, password) = get_credentials();

    let mut client = if is_paper {
        println!("\n📊 Logging in to PAPER trading account...");
        WebullClient::new_paper(Some(6))?
    } else {
        println!("\n💰 Logging in to LIVE trading account...");
        WebullClient::new_live(Some(6))?
    };

    match client
        .login(&username, &password, None, None, None, None)
        .await
    {
        Ok(_) => {
            if is_paper {
                println!("✅ Login successful to PAPER account!\n");
            } else {
                println!("✅ Login successful to LIVE account!\n");

                // For live trading, we need to get the trade token to place orders
                println!("🔐 Getting trade token for live trading...");

                // Try to get trading PIN from environment or prompt user
                let trading_pin = match env::var("WEBULL_TRADING_PIN") {
                    Ok(pin) => pin,
                    Err(_) => {
                        println!("Trading PIN required for placing orders.");
                        get_user_input("Enter your 6-digit trading PIN: ")
                    }
                };

                match client.get_trade_token(&trading_pin).await {
                    Ok(token) => {
                        println!("✅ Trade token obtained successfully!");
                        println!("   Token length: {} characters\n", token.len());
                    }
                    Err(e) => {
                        error!("⚠️  Failed to get trade token: {}", e);
                        println!("Note: You won't be able to place orders without a trade token.");
                        println!("You can still view account info, quotes, and other read-only operations.\n");
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Login failed: {}", e);
            return Err(e);
        }
    }

    loop {
        display_menu(client.is_paper());

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
            "9" => display_positions(&client).await?,
            "10" => analyze_portfolio(&client).await?,
            "11" => get_news_interactive(&client).await?,
            "12" => run_automated_test_suite(&client).await?,
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

fn display_menu(is_paper: bool) {
    println!("\n=====================================");
    println!(
        "      {} TRADING MENU",
        if is_paper { "PAPER" } else { "LIVE" }
    );
    println!("=====================================");
    println!("1.  View Account Information");
    println!("2.  Get Stock Quote");
    println!("3.  Get Historical Data");
    println!("4.  Place Market Order");
    println!("5.  Place Limit Order");
    println!("6.  Place Stop-Loss Order");
    println!("7.  View Current Orders");
    println!("8.  Cancel Order");
    println!("9.  View Positions");
    println!("10. Analyze Portfolio");
    println!("11. Get Market News");
    println!("12. Run Automated Test Suite");
    println!("0.  Exit");
    if !is_paper {
        println!("\n⚠️  LIVE TRADING - Real Money!");
    }
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

async fn display_account_info(client: &WebullClient) -> Result<()> {
    println!("\n💰 Account Information");
    println!("─────────────────────────");

    match client.get_account().await {
        Ok(account) => {
            if let Some(account_id) = account.account_id {
                println!("Account ID: {}", account_id);
            }
            if let Some(net_liquidation) = account.net_liquidation {
                println!("Net Liquidation: ${:.2}", net_liquidation);
            }
            if let Some(total_cash) = account.total_cash {
                println!("Total Cash: ${:.2}", total_cash);
            }
            if let Some(cash_balance) = account.cash_balance {
                println!("Cash Balance: ${:.2}", cash_balance);
            }
            if let Some(buying_power) = account.buying_power {
                println!("Buying Power: ${:.2}", buying_power);
            }
            if let Some(total_market_value) = account.total_market_value {
                println!("Total Market Value: ${:.2}", total_market_value);
            }
            if let Some(unrealized_pl) = account.unrealized_profit_loss {
                println!("Unrealized P&L: ${:.2}", unrealized_pl);
            }
            if let Some(unrealized_pl_rate) = account.unrealized_profit_loss_rate {
                println!("Unrealized P&L Rate: {:.2}%", unrealized_pl_rate * 100.0);
            }
        }
        Err(e) => {
            error!("Failed to get account details: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

async fn get_quote_interactive(client: &WebullClient) -> Result<()> {
    let symbol = get_user_input("Enter stock symbol (e.g., AAPL): ").to_uppercase();

    println!("\n🔍 Fetching quote for {}...", symbol);

    let tickers = client.find_ticker(&symbol).await?;

    if let Some(ticker) = tickers.first() {
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;

        println!("\n📊 Quote for {} - {}", ticker.symbol, ticker.name);
        println!("────────────────────────────");
        println!("Current Price: ${:.2}", quote.close);
        println!(
            "Change: ${:.2} ({:.2}%)",
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

async fn get_historical_data_interactive(client: &WebullClient) -> Result<()> {
    let symbol = get_user_input("Enter stock symbol (e.g., AAPL): ").to_uppercase();
    let days = get_user_input("Number of days to fetch (default 10): ");
    let days = days.parse::<i32>().unwrap_or(10);

    println!(
        "\n📊 Fetching {} days of historical data for {}...",
        days, symbol
    );

    // Request more bars to ensure we get the desired number
    // API may return fewer bars for recent dates
    let count = days; // Request at least 100 to get more history

    let tickers = client.find_ticker(&symbol).await?;

    if let Some(ticker) = tickers.first() {
        let bars = client
            .get_bars(&ticker.ticker_id.to_string(), "d1", count, None)
            .await?;

        if bars.is_empty() {
            println!("\n⚠️  No historical data available for {}", ticker.symbol);
        } else {
            println!(
                "\nHistorical Data for {} ({} bar(s) returned):",
                ticker.symbol,
                bars.len()
            );
            println!("────────────────────────────");
            // Display only the requested number of days
            for (i, bar) in bars.iter().take(days as usize).enumerate() {
                // Convert timestamp to readable date if available
                let date_str = if bar.timestamp > 0 {
                    chrono::DateTime::from_timestamp(bar.timestamp, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    format!("Bar {}", i + 1)
                };

                println!(
                    "{}: Open: ${:.2}, High: ${:.2}, Low: ${:.2}, Close: ${:.2}, Volume: {:.0}",
                    date_str, bar.open, bar.high, bar.low, bar.close, bar.volume
                );
            }

            if bars.len() < days as usize {
                println!(
                    "\n⚠️  Note: Only {} bar(s) available (requested {})",
                    bars.len(),
                    days
                );
            }
        }
    } else {
        println!("❌ Ticker {} not found", symbol);
    }

    Ok(())
}

async fn place_market_order_interactive(client: &WebullClient) -> Result<()> {
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
        println!(
            "  Action: {} {} shares of {}",
            action_str, quantity, ticker.symbol
        );
        println!("  Current Price: ${:.2}", quote.close);
        println!("  Estimated Total: ${:.2}", quote.close * quantity);

        if !confirm_action(&format!(
            "Place this MARKET order for {} shares of {}",
            quantity, ticker.symbol
        )) {
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

async fn place_limit_order_interactive(client: &WebullClient) -> Result<()> {
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
        println!(
            "  Action: {} {} shares of {}",
            action_str, quantity, ticker.symbol
        );
        println!("  Current Price: ${:.2}", quote.close);
        println!("  Limit Price: ${:.2}", limit_price);
        println!("  Max Total: ${:.2}", limit_price * quantity);

        if !confirm_action(&format!(
            "Place this LIMIT order for {} shares of {} at ${:.2}",
            quantity, ticker.symbol, limit_price
        )) {
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

async fn place_stop_order_interactive(client: &WebullClient) -> Result<()> {
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

        if !confirm_action(&format!(
            "Place this STOP order for {} shares of {} at ${:.2}",
            quantity, ticker.symbol, stop_price
        )) {
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

async fn display_current_orders(client: &WebullClient) -> Result<()> {
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
                    println!(
                        "   Type: {} {} @ {}",
                        action_str, order.quantity, order_type_str
                    );
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

async fn cancel_order_interactive(client: &WebullClient) -> Result<()> {
    println!("\n🔄 Cancel Order");
    println!("───────────────");

    match client.get_orders(Some(20)).await {
        Ok(orders) => {
            let pending_orders: Vec<_> = orders
                .iter()
                .filter(|o| {
                    matches!(
                        o.status,
                        OrderStatus::Working | OrderStatus::Pending | OrderStatus::PartialFilled
                    )
                })
                .collect();

            if pending_orders.is_empty() {
                println!("No open orders to cancel");
                return Ok(());
            }

            println!("\nOpen Orders:");
            for (i, order) in pending_orders.iter().enumerate() {
                let action_str = match order.action {
                    OrderAction::Buy => "BUY",
                    OrderAction::Sell => "SELL",
                };
                println!(
                    "{}. {} {} shares - Order ID: {}",
                    i + 1,
                    action_str,
                    order.quantity,
                    order.order_id
                );
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
                                    println!(
                                        "⚠️ Could not cancel order {} (may already be filled)",
                                        order.order_id
                                    );
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

async fn display_positions(client: &WebullClient) -> Result<()> {
    println!("\n📈 Current Positions");
    println!("───────────────────");

    match client.get_positions().await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("No positions currently held");
            } else {
                let mut total_market_value = 0.0;
                let mut total_unrealized_pnl = 0.0;
                let mut total_cost_basis = 0.0;

                println!(
                    "\n{:<8} {:<10} {:<12} {:<12} {:<12} {:<10}",
                    "Symbol", "Quantity", "Avg Cost", "Current", "Market Val", "P&L"
                );
                println!("{}", "─".repeat(70));

                for position in positions.iter() {
                    if let Some(ticker) = &position.ticker {
                        // Get current quote for the position
                        let current_price =
                            match client.get_quotes(&ticker.ticker_id.to_string()).await {
                                Ok(quote) => quote.close,
                                Err(_) => position.last_price,
                            };

                        let market_value = position.market_value;
                        let cost_basis = position.quantity * position.avg_cost;
                        let unrealized_pnl = position
                            .unrealized_profit_loss
                            .unwrap_or(market_value - cost_basis);
                        let pnl_percentage =
                            position.unrealized_profit_loss_rate.unwrap_or_else(|| {
                                if cost_basis > 0.0 {
                                    unrealized_pnl / cost_basis
                                } else {
                                    0.0
                                }
                            }) * 100.0;

                        // Track totals
                        total_market_value += market_value;
                        total_unrealized_pnl += unrealized_pnl;
                        total_cost_basis += cost_basis;

                        // Color coding for P&L (emoji indicators)
                        let pnl_indicator = if unrealized_pnl > 0.0 {
                            "📈"
                        } else if unrealized_pnl < 0.0 {
                            "📉"
                        } else {
                            "➡️"
                        };

                        println!(
                            "{:<8} {:<10.2} ${:<11.2} ${:<11.2} ${:<11.2} {}{:>8.2} ({:>6.2}%)",
                            ticker.symbol,
                            position.quantity,
                            position.avg_cost,
                            current_price,
                            market_value,
                            pnl_indicator,
                            unrealized_pnl,
                            pnl_percentage
                        );
                    }
                }

                println!("{}", "─".repeat(70));

                // Summary
                let total_pnl_percentage = if total_cost_basis > 0.0 {
                    (total_unrealized_pnl / total_cost_basis) * 100.0
                } else {
                    0.0
                };

                println!("\nPortfolio Summary:");
                println!("  Total Positions: {}", positions.len());
                println!("  Total Cost Basis: ${:.2}", total_cost_basis);
                println!("  Total Market Value: ${:.2}", total_market_value);
                println!(
                    "  Total Unrealized P&L: ${:.2} ({:.2}%)",
                    total_unrealized_pnl, total_pnl_percentage
                );

                // Performance indicator
                if total_unrealized_pnl > 0.0 {
                    println!("\n  📈 Overall: Positive Performance!");
                } else if total_unrealized_pnl < 0.0 {
                    println!("\n  📉 Overall: Negative Performance");
                } else {
                    println!("\n  ➡️ Overall: Flat Performance");
                }

                // Show top gainers/losers
                let mut sorted_positions = positions.clone();
                sorted_positions.sort_by(|a, b| {
                    let a_rate = a.unrealized_profit_loss_rate.unwrap_or(0.0);
                    let b_rate = b.unrealized_profit_loss_rate.unwrap_or(0.0);
                    b_rate.partial_cmp(&a_rate).unwrap()
                });

                if sorted_positions.len() > 0 {
                    println!("\n  Top Performers:");
                    for (i, position) in sorted_positions.iter().take(3).enumerate() {
                        if let Some(ticker) = &position.ticker {
                            let pnl_rate =
                                position.unrealized_profit_loss_rate.unwrap_or(0.0) * 100.0;
                            if pnl_rate != 0.0 {
                                println!("    {}. {} ({:+.2}%)", i + 1, ticker.symbol, pnl_rate);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get positions: {}", e);
            println!("⚠️  Unable to retrieve positions. This may not be available for paper trading accounts.");
        }
    }

    Ok(())
}

async fn analyze_portfolio(client: &WebullClient) -> Result<()> {
    println!("\n📊 Portfolio Analysis");
    println!("─────────────────────");

    match client.get_account().await {
        Ok(account) => {
            // Get total value - use net_liquidation if available, otherwise try to calculate
            let total_value = account.net_liquidation.unwrap_or_else(|| {
                let cash = account.total_cash.unwrap_or(0.0);
                let market_value = account.total_market_value.unwrap_or(0.0);
                cash + market_value
            });

            let cash_percentage = if total_value > 0.0 {
                account
                    .total_cash
                    .map(|c| (c / total_value) * 100.0)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let invested_percentage = if total_value > 0.0 {
                account
                    .total_market_value
                    .map(|m| (m / total_value) * 100.0)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            println!("\nPortfolio Allocation:");
            if total_value > 0.0 {
                println!("  Total Value: ${:.2}", total_value);
            }
            if let Some(total_cash) = account.total_cash {
                println!("  Cash: ${:.2} ({:.1}%)", total_cash, cash_percentage);
            }
            if let Some(total_market_value) = account.total_market_value {
                println!(
                    "  Invested: ${:.2} ({:.1}%)",
                    total_market_value, invested_percentage
                );
            }

            println!("\nPerformance Metrics:");
            if let Some(unrealized_pl) = account.unrealized_profit_loss {
                println!("  Unrealized P&L: ${:.2}", unrealized_pl);
                if unrealized_pl > 0.0 {
                    println!("  📈 Positive performance!");
                } else if unrealized_pl < 0.0 {
                    println!("  📉 Negative performance");
                } else {
                    println!("  ➡️ Flat performance");
                }
            }
            if let Some(unrealized_pl_rate) = account.unrealized_profit_loss_rate {
                println!("  Unrealized P&L Rate: {:.2}%", unrealized_pl_rate * 100.0);
            }

            println!("\nRisk Metrics:");
            if let Some(buying_power) = account.buying_power {
                println!("  Buying Power: ${:.2}", buying_power);
            }
            if let Some(unsettled_funds) = account.unsettled_funds {
                println!("  Unsettled Funds: ${:.2}", unsettled_funds);
            }
        }
        Err(e) => {
            error!("Failed to analyze portfolio: {}", e);
        }
    }

    // Display positions (if available for live trading)
    match client.get_positions().await {
        Ok(positions) => {
            if !positions.is_empty() {
                println!("\nCurrent Positions:");
                println!("──────────────────");
                for position in positions.iter() {
                    if let Some(ticker) = &position.ticker {
                        println!(
                            "  {} {} shares @ avg ${:.2}",
                            ticker.symbol, position.quantity, position.avg_cost
                        );
                        println!("    Market Value: ${:.2}", position.market_value);

                        if let Some(pnl) = position.unrealized_profit_loss {
                            let emoji = if pnl > 0.0 { "📈" } else { "📉" };
                            let pnl_rate = position.unrealized_profit_loss_rate.unwrap_or(0.0);
                            println!(
                                "    {} Unrealized P&L: ${:.2} ({:.2}%)",
                                emoji,
                                pnl,
                                pnl_rate * 100.0
                            );
                        }
                    }
                }
            }
        }
        Err(_) => {
            // Silently ignore errors getting positions (not available for paper trading)
        }
    }

    Ok(())
}

async fn get_news_interactive(client: &WebullClient) -> Result<()> {
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
                    if let Some(summary) = &news.summary {
                        let truncated = if summary.len() > 200 {
                            format!("{}...", &summary[..200])
                        } else {
                            summary.clone()
                        };
                        println!("   {}", truncated);
                    } else {
                        println!("   (No summary available)");
                    }
                    let source = news
                        .source_name
                        .as_ref()
                        .or(news.collect_source.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown");
                    // news_time is now a String, extract date part if available
                    let time_str = if news.news_time.len() >= 10 {
                        &news.news_time[..10] // Just show date part
                    } else {
                        &news.news_time
                    };
                    println!("   Source: {} | Date: {}", source, time_str);
                }
            }
        }
        Err(e) => {
            warn!("Failed to fetch news: {}", e);
        }
    }

    Ok(())
}

async fn run_automated_test_suite(client: &WebullClient) -> Result<()> {
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
                    println!(
                        "  {} - Price: ${:.2}, Change: {:.2}%",
                        ticker.symbol,
                        quote.close,
                        quote.change_ratio * 100.0
                    );
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
