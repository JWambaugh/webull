// Check Positions Example
//
// This example demonstrates how to retrieve and display positions
// for both live and paper trading accounts. It shows:
// - How to get positions from account details
// - How to get positions directly via get_positions()
// - Position details like quantity, average cost, and P&L

use webull::{WebullClient, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");
    let trade_pin = env::var("WEBULL_TRADING_PIN").ok();
    
    println!("Testing Live Account Positions:");
    println!("{}", "=".repeat(50));
    
    // Test Live Account
    {
        let mut client = WebullClient::new_live(None).expect("Failed to create live client");
        
        match client.login(&username, &password, None, None, None, None).await {
            Ok(_) => {
                println!("✓ Logged in to live account");
                
                if let Some(pin) = &trade_pin {
                    if let Ok(_) = client.get_trade_token(pin).await {
                        println!("✓ Got trade token");
                    }
                }
                
                // Get account details first
                match client.get_account().await {
                    Ok(account) => {
                        println!("\n📊 Account Details:");
                        println!("  Account ID: {:?}", account.account_id);
                        println!("  Account Type: {:?}", account.account_type);
                        println!("  Net Liquidation: {:?}", account.net_liquidation);
                        println!("  Positions array: {:?}", account.positions.as_ref().map(|p| p.len()));
                        println!("  Positions2 array: {:?}", account.positions2.as_ref().map(|p| p.len()));
                        
                        // Check positions from account
                        if let Some(positions) = &account.positions {
                            println!("\n  Positions from account.positions: {}", positions.len());
                            for pos in positions {
                                if let Some(ticker) = &pos.ticker {
                                    println!("    - {} : {} shares @ ${}", ticker.symbol, pos.quantity, pos.avg_cost);
                                }
                            }
                        }
                        
                        if let Some(positions) = &account.positions2 {
                            println!("\n  Positions from account.positions2: {}", positions.len());
                            for pos in positions {
                                if let Some(ticker) = &pos.ticker {
                                    println!("    - {} : {} shares @ ${}", ticker.symbol, pos.quantity, pos.avg_cost);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to get account: {}", e);
                    }
                }
                
                // Now try get_positions
                match client.get_positions().await {
                    Ok(positions) => {
                        println!("\n📈 Positions from get_positions():");
                        println!("  Total positions: {}", positions.len());
                        for position in positions {
                            if let Some(ticker) = &position.ticker {
                                println!("  {} : {} shares @ ${}", 
                                    ticker.symbol, 
                                    position.quantity, 
                                    position.avg_cost
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to get positions: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to login to live: {}", e);
            }
        }
    }
    
    println!("\n\nTesting Paper Account Positions:");
    println!("{}", "=".repeat(50));
    
    // Test Paper Account
    {
        let mut client = WebullClient::new_paper(None).expect("Failed to create paper client");
        
        match client.login(&username, &password, None, None, None, None).await {
            Ok(_) => {
                println!("✓ Logged in to paper account");
                
                // Get account details first
                match client.get_account().await {
                    Ok(account) => {
                        println!("\n📊 Account Details:");
                        println!("  Account ID: {:?}", account.account_id);
                        println!("  Net Liquidation: {:?}", account.net_liquidation);
                        println!("  Positions array: {:?}", account.positions.as_ref().map(|p| p.len()));
                        println!("  Positions2 array: {:?}", account.positions2.as_ref().map(|p| p.len()));
                        
                        // Check positions from account
                        if let Some(positions) = &account.positions {
                            println!("\n  Positions from account.positions: {}", positions.len());
                            for pos in positions {
                                if let Some(ticker) = &pos.ticker {
                                    println!("    - {} : {} shares @ ${}", ticker.symbol, pos.quantity, pos.avg_cost);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to get account: {}", e);
                    }
                }
                
                // Now try get_positions
                match client.get_positions().await {
                    Ok(positions) => {
                        println!("\n📈 Positions from get_positions():");
                        println!("  Total positions: {}", positions.len());
                        for position in positions {
                            if let Some(ticker) = &position.ticker {
                                println!("  {} : {} shares @ ${}", 
                                    ticker.symbol, 
                                    position.quantity, 
                                    position.avg_cost
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to get positions: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to login to paper: {}", e);
            }
        }
    }
    
    Ok(())
}