use webull::{WebullClient, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");
    
    // Test Live Account
    let mut client = WebullClient::new_live(None).expect("Failed to create live client");
    
    match client.login(&username, &password, None, None, None, None).await {
        Ok(_) => {
            println!("✓ Logged in to live account");
            
            // Test getting news for PROK
            match client.get_news("PROK", 0, 5).await {
                Ok(news_items) => {
                    println!("\n📰 News for PROK:");
                    println!("  Total items: {}", news_items.len());
                    for (i, news) in news_items.iter().enumerate() {
                        println!("\n  {}. {}", i + 1, news.title);
                        println!("     Source: {:?}", news.source_name);
                        println!("     Time: {}", news.news_time);
                        if let Some(summary) = &news.summary {
                            let truncated = if summary.len() > 100 {
                                format!("{}...", &summary[..100])
                            } else {
                                summary.clone()
                            };
                            println!("     Summary: {}", truncated);
                        } else {
                            println!("     Summary: (none)");
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to get news: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to login: {}", e);
        }
    }
    
    Ok(())
}