// Portfolio Analysis Example
//
// This example demonstrates comprehensive portfolio analysis:
// - Calculate total portfolio value
// - Analyze cash vs invested percentages
// - Track overall P&L performance
// - Display detailed account metrics
// Essential for portfolio management and performance tracking.

use std::env;
use webull_unofficial::{Result, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    // Test Live Account Portfolio Analysis
    let mut client = WebullClient::new_live(None).expect("Failed to create live client");

    match client
        .login(&username, &password, None, None, None, None)
        .await
    {
        Ok(_) => {
            println!("✓ Logged in to live account");

            // Get account for portfolio analysis
            match client.get_account().await {
                Ok(account) => {
                    println!("\n📊 Portfolio Analysis:");
                    println!("  Account ID: {:?}", account.account_id);
                    println!("  Net Liquidation: {:?}", account.net_liquidation);
                    println!("  Total Cash: {:?}", account.total_cash);
                    println!("  Cash Balance: {:?}", account.cash_balance);
                    println!("  Total Market Value: {:?}", account.total_market_value);
                    println!("  Buying Power: {:?}", account.buying_power);

                    // Calculate portfolio metrics
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

                    println!("\n  Portfolio Metrics:");
                    println!("    Total Value: ${:.2}", total_value);
                    println!("    Cash: {:.1}%", cash_percentage);
                    println!("    Invested: {:.1}%", invested_percentage);

                    if let Some(unrealized_pnl) = account.unrealized_profit_loss {
                        if let Some(unrealized_pnl_rate) = account.unrealized_profit_loss_rate {
                            let pnl_emoji = if unrealized_pnl >= 0.0 {
                                "📈"
                            } else {
                                "📉"
                            };
                            println!(
                                "    {} Total P&L: ${:.2} ({:.2}%)",
                                pnl_emoji,
                                unrealized_pnl,
                                unrealized_pnl_rate * 100.0
                            );
                        }
                    }

                    println!("\n✓ Portfolio analysis successful!");
                }
                Err(e) => {
                    println!("✗ Failed to get account: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to login: {}", e);
        }
    }

    Ok(())
}
