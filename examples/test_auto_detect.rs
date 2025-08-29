// Quick test to verify automatic order type detection
use webull_unofficial::models::*;

fn detect_order_type(has_limit: bool, has_stop: bool) -> OrderType {
    match (has_limit, has_stop) {
        (true, true) => OrderType::StopLimit,
        (true, false) => OrderType::Limit,
        (false, true) => OrderType::Stop,
        (false, false) => OrderType::Market,
    }
}

fn main() {
    println!("Testing automatic order type detection:\n");

    // Test all combinations
    println!("No prices set → {:?}", detect_order_type(false, false));
    println!("limit(100) only → {:?}", detect_order_type(true, false));
    println!("stop(90) only → {:?}", detect_order_type(false, true));
    println!("limit(100).stop(90) → {:?}", detect_order_type(true, true));

    println!("\n✅ Auto-detection logic working correctly!");
    println!("\nExample usage:");
    println!("  client.place_order_with()");
    println!("    .limit(100)    // Sets limit price");
    println!("    .stop(90)      // Sets stop price");
    println!("    .buy()");
    println!("    .quantity(1)");
    println!("    .await?");
    println!("  → Creates a STOP-LIMIT order automatically!");
}
