# Webull Rust API

An unofficial Rust library for the Webull API, providing full functionality for trading, market data, and streaming.

This library is a Rust port of the excellent [webull Python library](https://github.com/tedchou12/webull) by [@tedchou12](https://github.com/tedchou12). The Python library served as the foundation for understanding Webull's API structure and authentication mechanisms.

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
webull = "0.1.1"
```

## Quick Start

### Unified Client Interface

The library provides a unified `WebullClient` enum that can work with both live and paper trading:

```rust
use webull::{WebullClient, models::*, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create paper trading client
    let mut client = WebullClient::new_paper(Some(6))?; // 6 = US region
    
    // Or create live trading client
    // let mut client = WebullClient::new_live(Some(6))?;
    
    // Login
    client.login("email@example.com", "password", None, None, None, None).await?;
    
    // Get account info
    let account = client.get_account().await?;
    if let Some(net_liquidation) = account.net_liquidation {
        println!("Account balance: ${:.2}", net_liquidation);
    }
    
    // Find ticker
    let tickers = client.find_ticker("AAPL").await?;
    if let Some(ticker) = tickers.first() {
        // Get quotes
        let quote = client.get_quotes(&ticker.ticker_id.to_string()).await?;
        println!("AAPL price: ${}", quote.close);
        
        // IMPORTANT: Live trading requires a trade token before placing orders
        // Paper trading does NOT require a trade token
        if !client.is_paper() {
            client.get_trade_token("your_trading_pin").await?;  // 6-digit PIN
        }
        
        let order = PlaceOrderRequest {
            ticker_id: ticker.ticker_id,
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 1.0,
            limit_price: Some(150.0),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };
        
        let order_id = client.place_order(&order).await?;
        println!("Order placed: {}", order_id);
    }
    
    Ok(())
}
```

### Direct Client Usage

You can also use the specific client implementations directly:

```rust
use webull::{LiveWebullClient, PaperWebullClient, models::*, error::Result};

// For live trading
let mut live_client = LiveWebullClient::new(Some(6))?;
live_client.login("email", "password", None, None, None, None).await?;
live_client.get_trade_token("123456").await?;  // Your 6-digit trading PIN - Required for placing orders!

// For paper trading  
let mut paper_client = PaperWebullClient::new(Some(6))?;
paper_client.login("email", "password", None, None, None, None).await?;
// No trade token needed for paper trading
```

## Architecture

The library is organized into three main client types:

1. **`WebullClient`** - A unified enum that provides a common interface for both live and paper trading
2. **`LiveWebullClient`** - Direct implementation for live trading operations
3. **`PaperWebullClient`** - Implementation for paper (simulated) trading

The unified `WebullClient` enum automatically delegates method calls to the appropriate underlying implementation, making it easy to switch between live and paper trading modes.

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

## Environment Variables

Create a `.env` file:

```env
WEBULL_USERNAME=your_email@example.com
WEBULL_PASSWORD=your_password
WEBULL_TRADING_PIN=123456  # Your 6-digit trading PIN
```

## Examples

See the `examples/` directory for more complete examples:

- `basic_usage.rs` - Login, get account info, positions, and quotes
- `trading_test.rs` - Interactive trading test with both live and paper support
- `paper_trading.rs` - Paper trading specific functionality
- `place_order.rs` - Place and cancel orders with live trading
- `streaming.rs` - Real-time data streaming
- `set_device_id.rs` - Device ID management utility
- `test_bars.rs` - Historical data retrieval example

Run examples with:

```bash
cargo run --example basic_usage

# Interactive trading test (supports both live and paper)
cargo run --example trading_test

# Paper trading only
cargo run --example paper_trading
```

## Important: Live Trading Requirements

### Trade Token
**Live trading requires obtaining a trade token before placing any orders.** This is a security measure that requires your 6-digit trading PIN (NOT your login password).

```rust
// For live trading, get trade token after login
if !client.is_paper() {
    client.get_trade_token("123456").await?;  // Your 6-digit trading PIN
}

// Now you can place orders
let order_id = client.place_order(&order).await?;
```

Paper trading does **NOT** require a trade token - you can place orders immediately after login.

## Working with Orders

### Getting Current Orders

```rust
// Get open orders
let orders = client.get_orders(None).await?;
for order in orders {
    println!("Order {}: {} {} shares of {} at ${:.2}", 
        order.order_id, 
        order.action, 
        order.quantity, 
        order.ticker.symbol,
        order.limit_price.unwrap_or(0.0)
    );
}
```

### Canceling Orders

```rust
// Get and cancel all open orders
let orders = client.get_orders(None).await?;
for order in orders {
    if order.status == OrderStatus::Working {
        client.cancel_order(&order.order_id).await?;
        println!("Cancelled order {}", order.order_id);
    }
}
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

## Acknowledgements

- **[@tedchou12](https://github.com/tedchou12)** - Creator of the original [webull Python library](https://github.com/tedchou12/webull) which this Rust implementation is based on. The Python library's clean design and comprehensive API coverage made this port possible.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.