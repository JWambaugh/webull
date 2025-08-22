# Webull Rust API

An unofficial Rust library for the Webull API, providing full functionality for trading, market data, and streaming.

## Features

- ✅ Full authentication support (including MFA)
- ✅ Real trading and paper trading
- ✅ Market data and quotes
- ✅ Order placement and management
- ✅ Real-time streaming via MQTT
- ✅ Options trading support
- ✅ Account management
- ✅ Technical indicators and charts
- ✅ News and fundamentals
- ✅ Screener functionality

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
webull = "0.6.1"
```

## Quick Start

```rust
use webull::{WebullClient, models::*, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let mut client = WebullClient::new(Some(6))?; // 6 = US region
    
    // Login
    client.login("email@example.com", "password", None, None, None, None).await?;
    
    // Get account info
    let account = client.get_account().await?;
    println!("Account balance: ${}", account.net_liquidation);
    
    // Get quotes
    let quote = client.get_quotes("913256135").await?; // AAPL ticker ID
    println!("AAPL price: ${}", quote.close);
    
    // Place order (requires trade token)
    client.get_trade_token("password").await?;
    
    let order = PlaceOrderRequest {
        ticker_id: 913256135,
        action: OrderAction::Buy,
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::Day,
        quantity: 1.0,
        limit_price: Some(150.0),
        // ... other fields
    };
    
    let order_id = client.place_order(&order).await?;
    println!("Order placed: {}", order_id);
    
    Ok(())
}
```

## Streaming Example

```rust
use webull::{StreamConn, stream::TopicTypes};

let mut stream = StreamConn::new(None);

// Set callbacks
stream.set_price_callback(|topic, data| {
    println!("Price update: {:?}", data);
});

// Connect and subscribe
stream.connect(&access_token, &device_id).await?;
stream.subscribe_ticker("913256135", TopicTypes::basic()).await?;
```

## Paper Trading

```rust
use webull::PaperWebullClient;

let mut client = PaperWebullClient::new(Some(6))?;
client.login("email@example.com", "password", None, None, None, None).await?;

// Place paper trades
let order_id = client.place_order(&ticker_id, &order).await?;
```

## Environment Variables

Create a `.env` file:

```env
WEBULL_USERNAME=your_email@example.com
WEBULL_PASSWORD=your_password
WEBULL_TRADE_PASSWORD=your_trade_password
```

## Examples

See the `examples/` directory for more complete examples:

- `basic_usage.rs` - Login, get account info, positions, and quotes
- `place_order.rs` - Place and cancel orders
- `streaming.rs` - Real-time data streaming
- `paper_trading.rs` - Paper trading functionality

Run examples with:

```bash
cargo run --example basic_usage
```

## API Coverage

### Account Management
- [x] Login/Logout
- [x] MFA support
- [x] Get account details
- [x] Get positions
- [x] Get orders history
- [x] Get account activities

### Trading
- [x] Place orders (stocks)
- [x] Cancel orders
- [x] Modify orders
- [x] Place option orders
- [x] OTOCO orders

### Market Data
- [x] Get quotes
- [x] Get bars/candles
- [x] Get options chains
- [x] Get Level 2 data
- [x] Search tickers

### Streaming
- [x] Real-time quotes
- [x] Order updates
- [x] Trade executions
- [x] Level 2 updates

### Analysis
- [x] Get fundamentals
- [x] Get news
- [x] Get analyst ratings
- [x] Screener
- [x] Rankings

## Error Handling

The library uses a custom `WebullError` type for comprehensive error handling:

```rust
match client.login(&username, &password, None, None, None, None).await {
    Ok(response) => println!("Logged in!"),
    Err(WebullError::MfaRequired) => println!("Need MFA code"),
    Err(WebullError::InvalidCredentials) => println!("Bad credentials"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## License

MIT

## Disclaimer

This is an unofficial API wrapper. Use at your own risk. The authors are not responsible for any financial losses incurred through use of this software.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.