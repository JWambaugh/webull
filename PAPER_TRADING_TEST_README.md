# Paper Trading Test Program

An interactive test program for the Webull Rust client's paper trading API with user prompts and confirmations.

## Features

The test program provides an interactive menu-driven interface with:

1. **Secure Authentication**: 
   - Prompts for credentials if not in environment
   - Secure password input (hidden characters)
   
2. **Interactive Trading**:
   - User prompts for stock symbols and quantities
   - Confirmation before placing any trades
   - Buy/Sell selection for orders
   
3. **Order Types**:
   - Market orders (with confirmation)
   - Limit orders (with custom price)
   - Stop-loss orders (with trigger price)
   
4. **Account Management**:
   - View account information
   - Check portfolio performance
   - Monitor P&L in real-time
   
5. **Market Data**:
   - Interactive quote lookup
   - Historical data with custom date ranges
   - Latest market news by symbol
   
6. **Order Management**:
   - View all current orders
   - Cancel specific orders with confirmation
   - Track order status

## Setup

### Option 1: Environment Variables (Optional)
Create a `.env` file in the project root:
```
WEBULL_USERNAME=your_email@example.com
WEBULL_PASSWORD=your_password
```

### Option 2: Interactive Login
If credentials are not in the environment, the program will prompt you to enter them securely.

## Running the Test

```bash
# Run the interactive test program
cargo run --example paper_trading_test

# With logging enabled
RUST_LOG=info cargo run --example paper_trading_test
```

## Interactive Menu

When you run the program, you'll see:

```
=====================================
           MAIN MENU                 
=====================================
1.  View Account Information
2.  Get Stock Quote
3.  Get Historical Data
4.  Place Market Order
5.  Place Limit Order
6.  Place Stop-Loss Order
7.  View Current Orders
8.  Cancel Order
9.  Analyze Portfolio
10. Get Market News
11. Run Automated Test Suite
0.  Exit
=====================================
Enter your choice:
```

## Usage Examples

### Placing an Order
```
Enter your choice: 4

🛒 Place Market Order
─────────────────────
Enter stock symbol: AAPL
Buy or Sell? (B/S): B
Enter quantity: 10

📋 Order Summary:
  Action: BUY 10 shares of AAPL
  Current Price: $175.50
  Estimated Total: $1755.00

⚠️  Place this MARKET order for 10 shares of AAPL Confirm? (y/n): y
✅ Market order placed successfully!
   Order ID: 123456789
```

### Getting a Quote
```
Enter your choice: 2
Enter stock symbol (e.g., AAPL): MSFT

🔍 Fetching quote for MSFT...

📊 Quote for MSFT - Microsoft Corporation
────────────────────────────
Current Price: $378.45
Change: $5.23 (1.40%)
Volume: 23456789
Day Range: $375.00 - $380.00
Previous Close: $373.22
Market Cap: $2815432.50M
```

## Safety Features

- **Confirmation Prompts**: Every trade requires explicit confirmation
- **Secure Password Input**: Passwords are never displayed on screen
- **Clear Order Summaries**: Shows exactly what will be executed before confirmation
- **Cancel Protection**: Confirms before cancelling any orders

## Error Handling

The program includes comprehensive error handling:
- Login failures with clear messages
- Network errors with retry guidance
- Invalid input validation
- Insufficient buying power warnings
- Market closed notifications

## Important Notes

- **Paper Trading Only**: No real money is involved
- **Educational Purpose**: Perfect for learning the API
- **Safe Testing**: All trades are simulated
- **Real-time Data**: Uses actual market data (may be delayed)

## Tips for Usage

1. **Start Small**: Test with small quantities first
2. **Check Account**: View account info before trading
3. **Use Limits**: Practice with limit orders to control price
4. **Monitor Orders**: Check order status regularly
5. **Cancel Practice**: Learn to cancel orders quickly

## Keyboard Shortcuts

- Enter `0`, `q`, or `Q` to exit
- Press Enter to continue after each operation
- Use `y` or `yes` to confirm actions
- Use `n` or `no` to cancel actions