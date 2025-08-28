#[cfg(test)]
mod tests {
    use crate::error::WebullError;
    use crate::models::*;
    use crate::utils::*;
    use crate::WebullClient;
    // use std::collections::HashMap; // Not needed after screener simplification

    #[test]
    fn test_password_hashing() {
        let password = "testpassword123";
        let hashed = hash_password(password);

        assert!(!hashed.is_empty());
        assert_eq!(hashed.len(), 32); // MD5 produces 32 hex characters

        // Test consistency
        let hashed2 = hash_password(password);
        assert_eq!(hashed, hashed2);
    }

    #[test]
    fn test_account_type_detection() {
        // Email accounts
        assert_eq!(get_account_type("test@example.com").unwrap(), 2);
        assert_eq!(get_account_type("user.name+tag@example.co.uk").unwrap(), 2);

        // Phone accounts
        assert_eq!(get_account_type("+1-2345678901").unwrap(), 1);
        assert_eq!(get_account_type("+86-13812345678").unwrap(), 1);

        // Default fallback
        assert_eq!(get_account_type("username").unwrap(), 2);
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.co.uk"));
        assert!(validate_email("test+tag@subdomain.example.org"));

        assert!(!validate_email("invalid"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("test@"));
        assert!(!validate_email("test@.com"));
        assert!(!validate_email("test@domain"));
    }

    #[test]
    fn test_interval_parsing() {
        // Valid intervals
        assert!(parse_interval("1m").is_ok());
        assert!(parse_interval("5m").is_ok());
        assert!(parse_interval("1h").is_ok());
        assert!(parse_interval("1d").is_ok());
        assert!(parse_interval("d1").is_ok());

        // Invalid intervals
        assert!(parse_interval("invalid").is_err());
        assert!(parse_interval("10s").is_err());
        assert!(parse_interval("2y").is_err());
    }

    #[test]
    fn test_region_code_mapping() {
        assert_eq!(get_region_code(Some("US")), 6);
        assert_eq!(get_region_code(Some("us")), 6);
        assert_eq!(get_region_code(Some("CN")), 1);
        assert_eq!(get_region_code(Some("cn")), 1);
        assert_eq!(get_region_code(Some("HK")), 2);
        assert_eq!(get_region_code(Some("hk")), 2);
        assert_eq!(get_region_code(None), 6); // Default to US
        assert_eq!(get_region_code(Some("unknown")), 6); // Default to US
    }

    #[test]
    fn test_price_formatting() {
        assert_eq!(format_price(123.456789, 2), "123.46");
        assert_eq!(format_price(0.001234, 4), "0.0012");
        assert_eq!(format_price(1000.0, 0), "1000");
        assert_eq!(format_price(99.999, 2), "100.00");
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_req_id();
        let id2 = generate_req_id();

        assert_eq!(id1.len(), 32); // UUID without hyphens
        assert_eq!(id2.len(), 32);
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_base64_encoding_decoding() {
        let original = b"Hello, Webull!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_timestamp_conversion() {
        let timestamp = 1609459200000i64; // 2021-01-01 00:00:00 UTC
        let formatted = timestamp_to_string(timestamp);

        assert!(formatted.contains("2021-01-01"));
        assert!(formatted.contains("00:00:00"));
    }

    #[tokio::test]
    async fn test_live_client_creation() {
        let client = WebullClient::new_live(Some(6));
        assert!(client.is_ok());
        // Region code is tested internally in the client
    }

    #[tokio::test]
    async fn test_paper_client_creation() {
        let client = WebullClient::new_paper(Some(6));
        assert!(client.is_ok());
    }

    #[test]
    fn test_order_action_serialization() {
        let buy = OrderAction::Buy;
        let sell = OrderAction::Sell;

        let buy_json = serde_json::to_string(&buy).unwrap();
        let sell_json = serde_json::to_string(&sell).unwrap();

        assert_eq!(buy_json, "\"BUY\"");
        assert_eq!(sell_json, "\"SELL\"");
    }

    #[test]
    fn test_order_type_serialization() {
        let market = OrderType::Market;
        let limit = OrderType::Limit;

        let market_json = serde_json::to_string(&market).unwrap();
        let limit_json = serde_json::to_string(&limit).unwrap();

        assert_eq!(market_json, "\"MKT\"");
        assert_eq!(limit_json, "\"LMT\"");
    }

    #[test]
    fn test_time_in_force_serialization() {
        let day = TimeInForce::Day;
        let gtc = TimeInForce::GoodTillCancel;

        let day_json = serde_json::to_string(&day).unwrap();
        let gtc_json = serde_json::to_string(&gtc).unwrap();

        assert_eq!(day_json, "\"DAY\"");
        assert_eq!(gtc_json, "\"GTC\"");
    }

    #[test]
    fn test_endpoints_generation() {
        let endpoints = crate::endpoints::Endpoints::new();

        // Test account endpoints
        let account_url = endpoints.account("12345");
        assert!(account_url.contains("12345"));
        assert!(account_url.contains("/v3/home/"));

        // Test stock endpoints
        let quote_url = endpoints.quotes("AAPL");
        assert!(quote_url.contains("AAPL"));
        assert!(quote_url.contains("getTickerRealTime"));

        // Test order endpoints
        let order_url = endpoints.place_orders("67890");
        assert!(order_url.contains("67890"));
        assert!(order_url.contains("placeStockOrder"));
    }

    #[test]
    fn test_error_types() {
        let auth_error = WebullError::AuthenticationError("Test".to_string());
        assert_eq!(auth_error.to_string(), "Authentication failed: Test");

        let mfa_error = WebullError::MfaRequired;
        assert_eq!(mfa_error.to_string(), "MFA required");

        let session_error = WebullError::SessionExpired;
        assert_eq!(session_error.to_string(), "Session expired");
    }

    #[test]
    fn test_place_order_request_serialization() {
        let order = PlaceOrderRequest {
            ticker_id: 913256135,
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            quantity: 10.0,
            limit_price: Some(150.50),
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        };

        let json = serde_json::to_value(&order).unwrap();

        assert_eq!(json["tickerId"], 913256135);
        assert_eq!(json["action"], "BUY");
        assert_eq!(json["orderType"], "LMT");
        assert_eq!(json["timeInForce"], "DAY");
        assert_eq!(json["quantity"], 10.0);
        assert_eq!(json["limitPrice"], 150.50);
        assert_eq!(json["outsideRegularTradingHour"], false);
    }

    #[test]
    fn test_screener_request() {
        // Test simplified screener request
        let request = ScreenerRequest {
            region_id: 6,
            plate_id: 1,
            rank_id: 0,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["regionId"], 6);
        assert_eq!(json["plateId"], 1);
        assert_eq!(json["rankId"], 0);
    }
}
