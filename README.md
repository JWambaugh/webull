# Webull Rust API

An unofficial Rust library for the Webull API, providing full functionality for trading, market data, and streaming.
Because this library uses the webull app API's, Official API key access is not required.

This library is a Rust port of the excellent [webull Python library](https://github.com/tedchou12/webull) by [@tedchou12](https://github.com/tedchou12). The Python library served as the foundation for understanding Webull's API structure and authentication mechanisms.

## ⚠️ Important Disclaimer

**This is an unofficial library that is not affiliated with or endorsed by Webull.**

Webull may change their API endpoints, authentication methods, or data structures at any time without notice, which could cause this library to stop working partially or completely. Users of this library should:

- Be prepared for potential breaking changes
- Test thoroughly in paper trading before using with real money
- Implement proper error handling for API failures
- Consider this library experimental and use at your own risk

The maintainers of this library cannot guarantee its continued functionality and are not responsible for any losses incurred from its use.

## Features

- ✅ Full authentication support (including MFA)
- ✅ Real trading and paper trading
- ✅ Market data and quotes
- ✅ Order placement and management
- ✅ Real-time streaming via MQTT
- ✅ News and fundamentals

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
webull = "1.1.0"
```

## Quick Start

### Unified Client Interface

The library provides a unified `WebullClient` enum that can work with both live and paper trading:

```rust
use webull_unofficial::{WebullClient, PlaceOrderRequest, LoginRequestBuilder, models::*, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create paper trading client
    let mut client = WebullClient::new_paper(Some(6))?; // 6 = US region

    // Or create live trading client
    // let mut client = WebullClient::new_live(Some(6))?;

    // Login - multiple options available:
    // Option 1: Original API (still supported)
    client.login("email@example.com", "password", None, None, None, None).await?;

    // Option 2: Fluent builder API (NEW - recommended)
    // client.login_with()
    //     .username("email@example.com")
    //     .password("password")
    //     .await?;


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

        // Place order using NEW auto-detect syntax - type detected from parameters
        let order_id = client.place_order_with()
            .ticker_id(ticker.ticker_id)
            .limit(150.0)  // Auto-detects as LIMIT order
            .buy()
            .quantity(1.0)
            .time_in_force(TimeInForce::Day)
            .await?;
        println!("Order placed: {}", order_id);
    }

    Ok(())
}
```

### Direct Client Usage

You can also use the specific client implementations directly:

```rust
use webull_unofficial::{LiveWebullClient, PaperWebullClient, models::*, error::Result};

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

## API Options

The library supports both traditional function calls and modern fluent builder patterns:

### Traditional API (Original - Still Fully Supported)

```rust
// Direct method calls with parameters
client.login("email", "password", None, None, None, None).await?;
client.get_bars("913256135", "5m", 100, None).await?;
client.get_news("AAPL", 0, 20).await?;
```

### Fluent Builder API (New - Recommended)

```rust
// Fluent builders that can be awaited directly
client.login_with()
    .username("email")
    .password("password")
    .mfa("123456")  // Optional
    .await?;

let bars = client.get_bars_with()
    .ticker_id("913256135")
    .interval("5m")
    .count(100)
    .await?;  // No build() needed!
```

## Builder Patterns

The library provides fluent builder patterns for constructing complex requests:

### Order Builder

```rust
// RECOMMENDED: Auto-detect order type from parameters
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .limit(150.0)  // Automatically detects LIMIT order
    .buy()
    .quantity(10.0)
    .await?;

// Stop-limit with auto-detection (both prices set)
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .limit(144.0)  // Both prices = STOP-LIMIT
    .stop(145.0)   // order automatically!
    .sell()
    .quantity(10.0)
    .await?;

// Also supported: Explicit order type methods
let order_id = client.place_limit_order_with(150.0)
    .ticker_id(ticker_id)
    .sell()
    .quantity(5.0)
    .extended_hours()
    .time_in_force(TimeInForce::GoodTillCancel)
    .await?;

// Traditional builder pattern (still supported)
let order = PlaceOrderRequest::market()
    .ticker_id(ticker_id)
    .buy()
    .quantity(10.0)
    .build()?;
let order_id = client.place_order(&order).await?;
```

### News Builder (Fluent API)

```rust
// Get latest news - directly await the builder!
let news = client.get_news_with()
    .ticker("AAPL")
    .latest(20)
    .await?;

// Paginate through news
let more_news = client.get_news_with()
    .ticker("AAPL")
    .after(last_news_id)
    .count(10)
    .await?;
```

### Bars/Candles Builder (Fluent API)

```rust
// Get historical bars
let bars = client.get_bars_with()
    .ticker_id("913256135")
    .interval("5m")
    .count(100)
    .from_date(chrono::Utc::now() - chrono::Duration::days(7))
    .await?;
```

### Options Builder (Fluent API)

```rust
// Get options near the money
let options = client.get_options_with()
    .ticker("AAPL")
    .calls_only()
    .near_the_money(current_price, 10.0) // Within 10% of current price
    .await?;
```

### Order Builders (Fluent API)

```rust
// NEW: Automatic order type detection
// The order type is automatically detected based on which parameters you set:

// Market order (no prices specified)
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .buy()
    .quantity(10.0)
    .await?;

// Limit order (limit price only)
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .limit(150.0)  // Auto-detects as LIMIT order
    .buy()
    .quantity(10.0)
    .await?;

// Stop order (stop price only)
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .stop(145.0)   // Auto-detects as STOP order
    .sell()
    .quantity(10.0)
    .await?;

// Stop-Limit order (both prices)
let order_id = client.place_order_with()
    .ticker_id(ticker_id)
    .limit(144.0)  // Auto-detects as STOP-LIMIT order
    .stop(145.0)   // when both prices are set
    .sell()
    .quantity(10.0)
    .await?;

// You can still use explicit order type methods if preferred:
let order_id = client.place_market_order_with()
    .ticker_id(ticker_id)
    .buy()
    .quantity(10.0)
    .await?;

let order_id = client.place_limit_order_with(150.0)
    .ticker_id(ticker_id)
    .sell()
    .quantity(5.0)
    .extended_hours()
    .time_in_force(TimeInForce::GoodTillCancel)
    .await?;
```

## Streaming Example

```rust
use webull_unofficial::{StreamConn, stream::{StreamConfig, TopicTypes}};

// Create streaming connection
let config = StreamConfig {
    debug: true,  // Enable debug logging
    ..Default::default()
};
let mut stream = StreamConn::new(Some(config));

// Set callbacks
stream.set_price_callback(|topic, data| {
    println!("Price update: {:?}", data);
});

// Connect using access token and device ID from login
stream.connect(&access_token, &device_id).await?;

// Subscribe to ticker updates
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
    let symbol = order.ticker.as_ref()
        .map(|t| t.symbol.as_str())
        .unwrap_or("Unknown");
    println!("Order {}: {:?} {} shares of {} at ${:.2}",
        order.order_id,
        order.action,
        order.quantity,
        symbol,
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
- [x] Get Level 2 data
- [x] Search tickers

### Streaming

- [x] Real-time quotes
- [x] Order updates
- [x] Trade executions
- [x] Level 2 updates

### Analysis

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

## Acknowledgements

- **[@tedchou12](https://github.com/tedchou12)** - Creator of the original [webull Python library](https://github.com/tedchou12/webull) which this Rust implementation is based on. The Python library's clean design and comprehensive API coverage made this port possible.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
