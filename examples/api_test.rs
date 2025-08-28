use webull::{WebullClient, models::*, error::Result};
use std::env;
use std::fs;
use serde_json::{Value, json};
use chrono::Local;

#[tokio::main]
async fn main() -> Result<()> {
    // Create output directory
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = format!("api_responses_{}", timestamp);
    fs::create_dir_all(&output_dir)?;
    
    // Load credentials
    let username = env::var("WEBULL_USERNAME")
        .expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD")
        .expect("WEBULL_PASSWORD not set");
    let trading_pin = env::var("WEBULL_TRADING_PIN")
        .expect("WEBULL_TRADING_PIN not set for live trading");
    
    // Test both paper and live
    for mode in &["paper", "live"] {
        println!("\n{}", "=".repeat(60));
        println!("Testing {} mode", mode);
        println!("{}\n", "=".repeat(60));
        
        let mode_dir = format!("{}/{}", output_dir, mode);
        fs::create_dir_all(&mode_dir)?;
        
        let mut client = if *mode == "paper" {
            WebullClient::new_paper(Some(6))?
        } else {
            WebullClient::new_live(Some(6))?
        };
        
        // Login
        println!("1. Testing login...");
        let login_response = client.login(&username, &password, None, None, None, None).await?;
        save_response(&mode_dir, "01_login", &json!(login_response))?;
        println!("   ✓ Login successful");
        
        // Get trade token (live only)
        if *mode == "live" {
            println!("2. Getting trade token...");
            match client.get_trade_token(&trading_pin).await {
                Ok(token) => {
                    save_response(&mode_dir, "02_trade_token", &json!({"token": token}))?;
                    println!("   ✓ Trade token obtained");
                }
                Err(e) => {
                    println!("   ✗ Trade token error: {}", e);
                    save_response(&mode_dir, "02_trade_token_error", &json!({"error": e.to_string()}))?;
                }
            }
        }
        
        // Get account details
        println!("3. Testing get_account...");
        match client.get_account().await {
            Ok(account) => {
                save_response(&mode_dir, "03_account", &json!(account))?;
                println!("   ✓ Account details retrieved");
            }
            Err(e) => {
                println!("   ✗ Account error: {}", e);
                save_response(&mode_dir, "03_account_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get positions
        println!("4. Testing get_positions...");
        match client.get_positions().await {
            Ok(positions) => {
                save_response(&mode_dir, "04_positions", &json!(positions))?;
                println!("   ✓ Positions retrieved: {} positions", positions.len());
            }
            Err(e) => {
                println!("   ✗ Positions error: {}", e);
                save_response(&mode_dir, "04_positions_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Find ticker PROK
        println!("5. Testing find_ticker (PROK)...");
        let tickers = client.find_ticker("PROK").await?;
        save_response(&mode_dir, "05_find_ticker", &json!(tickers))?;
        
        let ticker = tickers.iter().find(|t| t.symbol == "PROK")
            .expect("PROK ticker not found");
        println!("   ✓ Found PROK, ticker_id: {}", ticker.ticker_id);
        
        // Get quotes for PROK
        println!("6. Testing get_quotes (PROK)...");
        match client.get_quotes(&ticker.ticker_id.to_string()).await {
            Ok(quote) => {
                save_response(&mode_dir, "06_quotes", &json!(quote))?;
                println!("   ✓ Quote retrieved: ${}", quote.close);
            }
            Err(e) => {
                println!("   ✗ Quotes error: {}", e);
                save_response(&mode_dir, "06_quotes_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get existing orders before placing new ones
        println!("7. Testing get_orders (before)...");
        match client.get_orders(Some(100)).await {
            Ok(orders) => {
                save_response(&mode_dir, "07_orders_before", &json!(orders))?;
                println!("   ✓ Retrieved {} existing orders", orders.len());
            }
            Err(e) => {
                println!("   ✗ Orders error: {}", e);
                save_response(&mode_dir, "07_orders_before_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get historical orders
        println!("8. Testing get_history_orders...");
        match client.get_history_orders("All", 10).await {
            Ok(history) => {
                save_response(&mode_dir, "08_history_orders", &history)?;
                println!("   ✓ Historical orders retrieved");
            }
            Err(e) => {
                println!("   ✗ History orders error: {}", e);
                save_response(&mode_dir, "08_history_orders_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get bars/candles
        println!("9. Testing get_bars (PROK)...");
        match client.get_bars(&ticker.ticker_id.to_string(), "m1", 50, None).await {
            Ok(bars) => {
                save_response(&mode_dir, "09_bars", &json!(bars))?;
                println!("   ✓ Retrieved {} bars", bars.len());
            }
            Err(e) => {
                println!("   ✗ Bars error: {}", e);
                save_response(&mode_dir, "09_bars_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get options chain
        println!("10. Testing get_options (PROK)...");
        match client.get_options("PROK").await {
            Ok(options) => {
                save_response(&mode_dir, "10_options", &json!(options))?;
                println!("   ✓ Retrieved {} option contracts", options.len());
            }
            Err(e) => {
                println!("   ✗ Options error: {}", e);
                save_response(&mode_dir, "10_options_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get news
        println!("11. Testing get_news (PROK)...");
        match client.get_news("PROK", 0, 5).await {
            Ok(news) => {
                save_response(&mode_dir, "11_news", &json!(news))?;
                println!("   ✓ Retrieved {} news items", news.len());
            }
            Err(e) => {
                println!("   ✗ News error: {}", e);
                save_response(&mode_dir, "11_news_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Get fundamentals
        println!("12. Testing get_fundamentals (PROK)...");
        // First try to get raw response for debugging
        if *mode == "live" {
            // Make direct API call to see raw response
            let live_client = match &client {
                WebullClient::Live(c) => Some(c),
                _ => None
            };
            if let Some(lc) = live_client {
                let headers = lc.build_req_headers(false, false, true);
                let response = lc.client
                    .get(&lc.endpoints.fundamentals("PROK"))
                    .headers(headers)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await;
                
                if let Ok(resp) = response {
                    if let Ok(raw_json) = resp.json::<Value>().await {
                        save_response(&mode_dir, "12_fundamentals_raw", &raw_json)?;
                        println!("   → Saved raw fundamentals response for debugging");
                    }
                }
            }
        }
        
        match client.get_fundamentals("PROK").await {
            Ok(fundamentals) => {
                save_response(&mode_dir, "12_fundamentals", &json!(fundamentals))?;
                println!("   ✓ Fundamentals retrieved");
            }
            Err(e) => {
                println!("   ✗ Fundamentals error: {}", e);
                save_response(&mode_dir, "12_fundamentals_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Now test order placement and management
        println!("\n--- Testing Order Placement ---");
        
        // Track order IDs for cleanup
        let mut placed_order_ids = Vec::new();
        
        // Test 1: Buy Limit Order at $1
        println!("13. Testing place_order (Buy Limit $1)...");
        let buy_limit_order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 1.0,
            limit_price: Some(1.0),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        match client.place_order(&buy_limit_order).await {
            Ok(order_id) => {
                save_response(&mode_dir, "13_place_buy_limit", &json!({"order_id": order_id}))?;
                println!("   ✓ Buy limit order placed: {}", order_id);
                placed_order_ids.push(order_id);
            }
            Err(e) => {
                println!("   ✗ Buy limit order error: {}", e);
                save_response(&mode_dir, "13_place_buy_limit_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Test 2: Sell Limit Order at $100 (will fail if no position)
        println!("14. Testing place_order (Sell Limit $100)...");
        let sell_limit_order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 1.0,
            limit_price: Some(100.0),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        match client.place_order(&sell_limit_order).await {
            Ok(order_id) => {
                save_response(&mode_dir, "14_place_sell_limit", &json!({"order_id": order_id}))?;
                println!("   ✓ Sell limit order placed: {}", order_id);
                placed_order_ids.push(order_id);
            }
            Err(e) => {
                println!("   ✗ Sell limit order error (expected if no position): {}", e);
                save_response(&mode_dir, "14_place_sell_limit_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Test 3: Market Buy to create a position
        println!("15. Testing place_order (Buy Market for position test)...");
        let buy_market_order = PlaceOrderRequest {
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
        
        match client.place_order(&buy_market_order).await {
            Ok(order_id) => {
                save_response(&mode_dir, "15_place_buy_market", &json!({"order_id": order_id}))?;
                println!("   ✓ Buy market order placed: {}", order_id);
                
                // Wait for market order to fill
                println!("16. Waiting 3 seconds for market order to fill...");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                
                // Check positions after market buy
                println!("17. Testing get_positions (after market buy)...");
                match client.get_positions().await {
                    Ok(positions) => {
                        save_response(&mode_dir, "17_positions_after_buy", &json!(positions))?;
                        println!("   ✓ Positions after buy: {} positions", positions.len());
                        for pos in &positions {
                            if let Some(ticker) = &pos.ticker {
                                println!("     - {} shares of {} at ${:.2}", 
                                    pos.quantity, &ticker.symbol, pos.avg_cost);
                            }
                        }
                        
                        // If we have a position, sell it
                        if !positions.is_empty() {
                            println!("18. Testing place_order (Sell Market to close position)...");
                            let sell_market_order = PlaceOrderRequest {
                                ticker_id: ticker.ticker_id,
                                action: OrderAction::Sell,
                                order_type: OrderType::Market,
                                time_in_force: TimeInForce::Day,
                                quantity: 1.0,
                                limit_price: None,
                                stop_price: None,
                                outside_regular_trading_hour: false,
                                serial_id: None,
                                combo_type: None,
                            };
                            
                            match client.place_order(&sell_market_order).await {
                                Ok(sell_order_id) => {
                                    save_response(&mode_dir, "18_place_sell_market", &json!({"order_id": sell_order_id}))?;
                                    println!("   ✓ Sell market order placed: {}", sell_order_id);
                                }
                                Err(e) => {
                                    println!("   ✗ Sell market order error: {}", e);
                                    save_response(&mode_dir, "18_place_sell_market_error", &json!({"error": e.to_string()}))?;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ✗ Positions after buy error: {}", e);
                        save_response(&mode_dir, "17_positions_after_buy_error", &json!({"error": e.to_string()}))?;
                    }
                }
            }
            Err(e) => {
                println!("   ✗ Buy market order error: {}", e);
                save_response(&mode_dir, "15_place_buy_market_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Wait a moment after all orders
        println!("19. Waiting 2 seconds for orders to be processed...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Get orders after placing
        println!("20. Testing get_orders (after all orders)...");
        match client.get_orders(Some(100)).await {
            Ok(orders) => {
                save_response(&mode_dir, "20_orders_after", &json!(orders))?;
                println!("   ✓ Retrieved {} orders after placing", orders.len());
                
                // Cancel all working orders
                println!("21. Testing cancel_order...");
                for order in &orders {
                    if order.status == OrderStatus::Working 
                        || order.status == OrderStatus::Pending
                        || order.status == OrderStatus::Submitted {
                        match client.cancel_order(&order.order_id).await {
                            Ok(success) => {
                                save_response(&mode_dir, &format!("21_cancel_{}", order.order_id), 
                                    &json!({"order_id": order.order_id, "success": success}))?;
                                if success {
                                    println!("   ✓ Cancelled order: {}", order.order_id);
                                } else {
                                    println!("   ✗ Failed to cancel order: {}", order.order_id);
                                }
                            }
                            Err(e) => {
                                println!("   ✗ Cancel order {} error: {}", order.order_id, e);
                                save_response(&mode_dir, &format!("21_cancel_{}_error", order.order_id), 
                                    &json!({"error": e.to_string()}))?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ✗ Orders after error: {}", e);
                save_response(&mode_dir, "20_orders_after_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Wait for cancellations to process
        if *mode == "live" {
            println!("22. Waiting 3 seconds for cancellations to process...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
        
        // Final order check
        println!("23. Testing get_orders (final check)...");
        match client.get_orders(Some(100)).await {
            Ok(orders) => {
                save_response(&mode_dir, "23_orders_final", &json!(orders))?;
                println!("   ✓ Final check: {} orders remaining", orders.len());
            }
            Err(e) => {
                println!("   ✗ Final orders error: {}", e);
                save_response(&mode_dir, "23_orders_final_error", &json!({"error": e.to_string()}))?;
            }
        }
        
        // Logout
        println!("24. Testing logout...");
        match client.logout().await {
            Ok(success) => {
                save_response(&mode_dir, "24_logout", &json!({"success": success}))?;
                println!("   ✓ Logout: {}", success);
            }
            Err(e) => {
                println!("   ✗ Logout error: {}", e);
                save_response(&mode_dir, "24_logout_error", &json!({"error": e.to_string()}))?;
            }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("Testing complete! Responses saved to: {}", output_dir);
    println!("{}\n", "=".repeat(60));
    
    Ok(())
}

fn save_response(dir: &str, name: &str, value: &Value) -> Result<()> {
    let path = format!("{}/{}.json", dir, name);
    let pretty_json = serde_json::to_string_pretty(value)?;
    fs::write(&path, pretty_json)?;
    Ok(())
}