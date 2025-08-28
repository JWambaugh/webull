use dotenv::dotenv;
use std::env;
use webull_unofficial::{error::Result, WebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    let mut client = WebullClient::new_paper(Some(6))?;
    client.login(&username, &password, None, None, None, None).await?;
    
    println!("Searching for AAPL...");
    let tickers = client.find_ticker("AAPL").await?;
    
    if let Some(ticker) = tickers.first() {
        println!("Found ticker: {} (ID: {})", ticker.symbol, ticker.ticker_id);
        
        // Try different count values
        for count in [1, 10, 100, 500, 1200].iter() {
            println!("\nFetching bars with count={}...", count);
            
            match client.get_bars(&ticker.ticker_id.to_string(), "d1", *count, None).await {
                Ok(bars) => {
                    println!("Got {} bars", bars.len());
                    if !bars.is_empty() {
                        let first = &bars[0];
                        let last = &bars[bars.len() - 1];
                        println!("  First bar: timestamp={}, close={:.2}", first.timestamp, first.close);
                        println!("  Last bar:  timestamp={}, close={:.2}", last.timestamp, last.close);
                    }
                }
                Err(e) => {
                    println!("Error getting bars: {:?}", e);
                }
            }
        }
    }
    
    Ok(())
}