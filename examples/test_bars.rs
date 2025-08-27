use dotenv::dotenv;
use std::env;
use webull::{error::Result, PaperWebullClient};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let username = env::var("WEBULL_USERNAME").expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD").expect("WEBULL_PASSWORD not set");

    let mut client = PaperWebullClient::new(Some(6))?;
    client.login(&username, &password, None, None, None, None).await?;
    
    println!("Searching for AAPL...");
    let tickers = client.find_ticker("AAPL").await?;
    
    if let Some(ticker) = tickers.first() {
        println!("Found ticker: {} (ID: {})", ticker.symbol, ticker.ticker_id);
        println!("Fetching bars with interval='1d', count=10...");
        
        match client.get_bars(&ticker.ticker_id.to_string(), "1d", 10, None).await {
            Ok(bars) => {
                println!("Got {} bars", bars.len());
                if bars.is_empty() {
                    println!("No bars returned!");
                } else {
                    for (i, bar) in bars.iter().enumerate().take(5) {
                        println!("Bar {}: open={:.2}, close={:.2}, volume={}", 
                            i, bar.open, bar.close, bar.volume);
                    }
                }
            }
            Err(e) => {
                println!("Error getting bars: {:?}", e);
            }
        }
    }
    
    Ok(())
}