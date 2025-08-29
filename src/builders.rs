// Builder patterns with client references for fluent async execution

use crate::{
    error::{Result, WebullError},
    models::*,
    WebullClient,
};
use std::future::Future;
use std::pin::Pin;

/// Login request builder that can be executed directly
pub struct LoginBuilderWithClient<'a> {
    client: &'a mut WebullClient,
    username: Option<String>,
    password: Option<String>,
    device_name: Option<String>,
    mfa_code: Option<String>,
    question_id: Option<String>,
    question_answer: Option<String>,
}

impl<'a> LoginBuilderWithClient<'a> {
    pub fn new(client: &'a mut WebullClient) -> Self {
        Self {
            client,
            username: None,
            password: None,
            device_name: None,
            mfa_code: None,
            question_id: None,
            question_answer: None,
        }
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn device_name(mut self, device_name: impl Into<String>) -> Self {
        self.device_name = Some(device_name.into());
        self
    }

    pub fn mfa(mut self, code: impl Into<String>) -> Self {
        self.mfa_code = Some(code.into());
        self
    }

    pub fn security_question(
        mut self,
        question_id: impl Into<String>,
        answer: impl Into<String>,
    ) -> Self {
        self.question_id = Some(question_id.into());
        self.question_answer = Some(answer.into());
        self
    }
}

impl<'a> std::future::IntoFuture for LoginBuilderWithClient<'a> {
    type Output = Result<LoginResponse>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let username = self
                .username
                .ok_or_else(|| WebullError::InvalidRequest("username is required".to_string()))?;
            let password = self
                .password
                .ok_or_else(|| WebullError::InvalidRequest("password is required".to_string()))?;

            self.client
                .login(
                    &username,
                    &password,
                    self.device_name.as_deref(),
                    self.mfa_code.as_deref(),
                    self.question_id.as_deref(),
                    self.question_answer.as_deref(),
                )
                .await
        })
    }
}

/// Bars request builder that can be executed directly
pub struct BarsRequestBuilderWithClient<'a> {
    client: &'a WebullClient,
    ticker_id: Option<String>,
    interval: Option<String>,
    count: Option<i32>,
    timestamp: Option<i64>,
}

impl<'a> BarsRequestBuilderWithClient<'a> {
    pub fn new(client: &'a WebullClient) -> Self {
        Self {
            client,
            ticker_id: None,
            interval: None,
            count: Some(100),
            timestamp: None,
        }
    }

    pub fn ticker_id(mut self, ticker_id: impl Into<String>) -> Self {
        self.ticker_id = Some(ticker_id.into());
        self
    }

    pub fn interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = Some(interval.into());
        self
    }

    pub fn count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn from_date(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(date.timestamp());
        self
    }
}

impl<'a> std::future::IntoFuture for BarsRequestBuilderWithClient<'a> {
    type Output = Result<Vec<Bar>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let ticker_id = self
                .ticker_id
                .ok_or_else(|| WebullError::InvalidRequest("ticker_id is required".to_string()))?;
            let interval = self
                .interval
                .ok_or_else(|| WebullError::InvalidRequest("interval is required".to_string()))?;
            let count = self.count.unwrap_or(100);

            self.client
                .get_bars(&ticker_id, &interval, count, self.timestamp)
                .await
        })
    }
}

/// News request builder that can be executed directly
pub struct NewsRequestBuilderWithClient<'a> {
    client: &'a WebullClient,
    ticker: Option<String>,
    last_id: Option<i64>,
    count: Option<i32>,
}

impl<'a> NewsRequestBuilderWithClient<'a> {
    pub fn new(client: &'a WebullClient) -> Self {
        Self {
            client,
            ticker: None,
            last_id: Some(0),
            count: Some(20),
        }
    }

    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    pub fn last_id(mut self, last_id: i64) -> Self {
        self.last_id = Some(last_id);
        self
    }

    pub fn after(mut self, news_id: i64) -> Self {
        self.last_id = Some(news_id);
        self
    }

    pub fn count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn latest(mut self, count: i32) -> Self {
        self.last_id = Some(0);
        self.count = Some(count);
        self
    }
}

impl<'a> std::future::IntoFuture for NewsRequestBuilderWithClient<'a> {
    type Output = Result<Vec<News>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let ticker = self
                .ticker
                .ok_or_else(|| WebullError::InvalidRequest("ticker is required".to_string()))?;
            let last_id = self.last_id.unwrap_or(0);
            let count = self.count.unwrap_or(20);

            self.client.get_news(&ticker, last_id, count).await
        })
    }
}

/// Options request builder that can be executed directly
pub struct OptionsRequestBuilderWithClient<'a> {
    client: &'a WebullClient,
    ticker: Option<String>,
    expiration_date: Option<String>,
    option_type: Option<String>,
    min_strike: Option<f64>,
    max_strike: Option<f64>,
}

impl<'a> OptionsRequestBuilderWithClient<'a> {
    pub fn new(client: &'a WebullClient) -> Self {
        Self {
            client,
            ticker: None,
            expiration_date: None,
            option_type: None,
            min_strike: None,
            max_strike: None,
        }
    }

    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    pub fn expiration(mut self, date: impl Into<String>) -> Self {
        self.expiration_date = Some(date.into());
        self
    }

    pub fn calls_only(mut self) -> Self {
        self.option_type = Some("CALL".to_string());
        self
    }

    pub fn puts_only(mut self) -> Self {
        self.option_type = Some("PUT".to_string());
        self
    }

    pub fn min_strike(mut self, price: f64) -> Self {
        self.min_strike = Some(price);
        self
    }

    pub fn max_strike(mut self, price: f64) -> Self {
        self.max_strike = Some(price);
        self
    }

    pub fn strike_range(mut self, min: f64, max: f64) -> Self {
        self.min_strike = Some(min);
        self.max_strike = Some(max);
        self
    }

    pub fn near_the_money(mut self, current_price: f64, percent_range: f64) -> Self {
        let range = current_price * (percent_range / 100.0);
        self.min_strike = Some(current_price - range);
        self.max_strike = Some(current_price + range);
        self
    }
}

impl<'a> std::future::IntoFuture for OptionsRequestBuilderWithClient<'a> {
    type Output = Result<Vec<OptionContract>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let ticker = self
                .ticker
                .ok_or_else(|| WebullError::InvalidRequest("ticker is required".to_string()))?;

            self.client.get_options(&ticker).await
        })
    }
}

/// Order builder that can be executed directly
pub struct PlaceOrderBuilderWithClient<'a> {
    client: &'a WebullClient,
    ticker_id: Option<i64>,
    action: Option<OrderAction>,
    order_type: Option<OrderType>, // Made optional for auto-detection
    time_in_force: TimeInForce,
    quantity: Option<f64>,
    limit_price: Option<f64>,
    stop_price: Option<f64>,
    outside_regular_trading_hour: bool,
    serial_id: Option<String>,
    combo_type: Option<String>,
}

impl<'a> PlaceOrderBuilderWithClient<'a> {
    /// Create a new order builder with automatic type detection
    pub fn new(client: &'a WebullClient) -> Self {
        Self {
            client,
            ticker_id: None,
            action: None,
            order_type: None, // Will be auto-detected
            time_in_force: TimeInForce::Day,
            quantity: None,
            limit_price: None,
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        }
    }

    pub fn market(client: &'a WebullClient) -> Self {
        Self::new_with_type(client, OrderType::Market)
    }

    pub fn limit_order(client: &'a WebullClient, price: f64) -> Self {
        Self::new_with_type(client, OrderType::Limit).limit(price)
    }

    pub fn stop_order(client: &'a WebullClient, price: f64) -> Self {
        Self::new_with_type(client, OrderType::Stop).stop(price)
    }

    pub fn stop_limit_order(client: &'a WebullClient, stop_price: f64, limit_price: f64) -> Self {
        Self::new_with_type(client, OrderType::StopLimit)
            .stop(stop_price)
            .limit(limit_price)
    }

    fn new_with_type(client: &'a WebullClient, order_type: OrderType) -> Self {
        Self {
            client,
            ticker_id: None,
            action: None,
            order_type: Some(order_type),
            time_in_force: TimeInForce::Day,
            quantity: None,
            limit_price: None,
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        }
    }

    pub fn ticker_id(mut self, ticker_id: i64) -> Self {
        self.ticker_id = Some(ticker_id);
        self
    }

    pub fn action(mut self, action: OrderAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn buy(mut self) -> Self {
        self.action = Some(OrderAction::Buy);
        self
    }

    pub fn sell(mut self) -> Self {
        self.action = Some(OrderAction::Sell);
        self
    }

    pub fn quantity(mut self, quantity: f64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set limit price (for limit and stop-limit orders)
    pub fn limit(mut self, price: f64) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set stop price (for stop and stop-limit orders)
    pub fn stop(mut self, price: f64) -> Self {
        self.stop_price = Some(price);
        self
    }

    /// Alias for limit() - for backwards compatibility
    pub fn limit_price(self, price: f64) -> Self {
        self.limit(price)
    }

    /// Alias for stop() - for backwards compatibility
    pub fn stop_price(self, price: f64) -> Self {
        self.stop(price)
    }

    pub fn extended_hours(mut self) -> Self {
        self.outside_regular_trading_hour = true;
        self
    }

    pub fn serial_id(mut self, id: String) -> Self {
        self.serial_id = Some(id);
        self
    }

    pub fn combo_type(mut self, combo_type: String) -> Self {
        self.combo_type = Some(combo_type);
        self
    }
}

impl<'a> std::future::IntoFuture for PlaceOrderBuilderWithClient<'a> {
    type Output = Result<String>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let ticker_id = self
                .ticker_id
                .ok_or_else(|| WebullError::InvalidRequest("ticker_id is required".to_string()))?;
            let action = self
                .action
                .ok_or_else(|| WebullError::InvalidRequest("action is required".to_string()))?;
            let quantity = self
                .quantity
                .ok_or_else(|| WebullError::InvalidRequest("quantity is required".to_string()))?;

            // Auto-detect order type if not explicitly set
            let order_type = if let Some(order_type) = self.order_type {
                order_type
            } else {
                // Detect based on which prices are set
                match (self.limit_price.is_some(), self.stop_price.is_some()) {
                    (true, true) => OrderType::StopLimit,
                    (true, false) => OrderType::Limit,
                    (false, true) => OrderType::Stop,
                    (false, false) => OrderType::Market,
                }
            };

            // Validate order type specific requirements
            match order_type {
                OrderType::Limit => {
                    if self.limit_price.is_none() {
                        return Err(WebullError::InvalidRequest(format!(
                            "{:?} order requires limit_price",
                            order_type
                        )));
                    }
                }
                OrderType::Stop => {
                    if self.stop_price.is_none() {
                        return Err(WebullError::InvalidRequest(
                            "Stop order requires stop_price".to_string(),
                        ));
                    }
                }
                OrderType::StopLimit => {
                    if self.limit_price.is_none() {
                        return Err(WebullError::InvalidRequest(
                            "StopLimit order requires limit_price".to_string(),
                        ));
                    }
                    if self.stop_price.is_none() {
                        return Err(WebullError::InvalidRequest(
                            "StopLimit order requires stop_price".to_string(),
                        ));
                    }
                }
                _ => {}
            }

            let order = PlaceOrderRequest {
                ticker_id,
                action,
                order_type,
                time_in_force: self.time_in_force,
                quantity,
                limit_price: self.limit_price,
                stop_price: self.stop_price,
                outside_regular_trading_hour: self.outside_regular_trading_hour,
                serial_id: self.serial_id,
                combo_type: self.combo_type,
            };

            self.client.place_order(&order).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WebullClient;

    #[test]
    fn test_login_builder_with_client() {
        let mut client = WebullClient::new_paper(Some(6)).unwrap();
        let builder = LoginBuilderWithClient::new(&mut client);

        // Test builder pattern
        let builder = builder
            .username("test@example.com")
            .password("testpass")
            .device_name("test_device")
            .mfa("123456");

        // Verify fields are set
        assert_eq!(builder.username, Some("test@example.com".to_string()));
        assert_eq!(builder.password, Some("testpass".to_string()));
        assert_eq!(builder.device_name, Some("test_device".to_string()));
        assert_eq!(builder.mfa_code, Some("123456".to_string()));
    }

    #[test]
    fn test_bars_request_builder() {
        let client = WebullClient::new_paper(Some(6)).unwrap();
        let builder = BarsRequestBuilderWithClient::new(&client);

        // Test builder pattern
        let builder = builder
            .ticker_id("913256135")
            .interval("5m")
            .count(100)
            .timestamp(1234567890);

        // Verify fields are set
        assert_eq!(builder.ticker_id, Some("913256135".to_string()));
        assert_eq!(builder.interval, Some("5m".to_string()));
        assert_eq!(builder.count, Some(100));
        assert_eq!(builder.timestamp, Some(1234567890));
    }

    #[test]
    fn test_bars_builder_from_date() {
        let client = WebullClient::new_paper(Some(6)).unwrap();
        let builder = BarsRequestBuilderWithClient::new(&client);

        // Test from_date conversion
        let date = chrono::Utc::now();
        let builder = builder.from_date(date);

        // Verify timestamp is set
        assert!(builder.timestamp.is_some());
        assert_eq!(builder.timestamp.unwrap(), date.timestamp());
    }

    #[test]
    fn test_news_request_builder() {
        let client = WebullClient::new_paper(Some(6)).unwrap();
        let builder = NewsRequestBuilderWithClient::new(&client);

        // Test builder pattern
        let builder = builder.ticker("AAPL").latest(20).after(9876543210);

        // Verify fields are set
        assert_eq!(builder.ticker, Some("AAPL".to_string()));
        assert_eq!(builder.count, Some(20));
        assert_eq!(builder.last_id, Some(9876543210));
    }

    #[test]
    fn test_news_builder_convenience_methods() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test latest() convenience method
        let builder = NewsRequestBuilderWithClient::new(&client).latest(10);
        assert_eq!(builder.count, Some(10));
        assert_eq!(builder.last_id, Some(0));

        // Test after() method
        let builder = NewsRequestBuilderWithClient::new(&client).after(123);
        assert_eq!(builder.last_id, Some(123));
    }

    #[test]
    fn test_options_request_builder() {
        let client = WebullClient::new_paper(Some(6)).unwrap();
        let builder = OptionsRequestBuilderWithClient::new(&client);

        // Test builder pattern
        let builder = builder
            .ticker("SPY")
            .calls_only()
            .near_the_money(450.0, 5.0);

        // Verify fields are set
        assert_eq!(builder.ticker, Some("SPY".to_string()));
        // Options builder uses option_type field instead of filter_calls/puts
        assert_eq!(builder.option_type, Some("CALL".to_string()));
        assert_eq!(builder.min_strike, Some(427.5)); // 450 * 0.95
        assert_eq!(builder.max_strike, Some(472.5)); // 450 * 1.05
    }

    #[test]
    fn test_options_builder_filters() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test calls_only
        let builder = OptionsRequestBuilderWithClient::new(&client).calls_only();
        assert_eq!(builder.option_type, Some("CALL".to_string()));

        // Test puts_only
        let builder = OptionsRequestBuilderWithClient::new(&client).puts_only();
        assert_eq!(builder.option_type, Some("PUT".to_string()));

        // Test near_the_money calculation
        let builder = OptionsRequestBuilderWithClient::new(&client).near_the_money(100.0, 10.0);
        assert_eq!(builder.min_strike, Some(90.0));
        assert_eq!(builder.max_strike, Some(110.0));
    }

    #[test]
    fn test_place_order_builder_auto_detect() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test auto-detection with new()
        let builder = PlaceOrderBuilderWithClient::new(&client);
        assert_eq!(builder.order_type, None);

        // Market order (no prices)
        let builder = PlaceOrderBuilderWithClient::new(&client)
            .ticker_id(123)
            .buy()
            .quantity(10.0);
        assert_eq!(builder.limit_price, None);
        assert_eq!(builder.stop_price, None);

        // Limit order (limit price only)
        let builder = PlaceOrderBuilderWithClient::new(&client)
            .ticker_id(123)
            .limit(150.0)
            .buy()
            .quantity(10.0);
        assert_eq!(builder.limit_price, Some(150.0));
        assert_eq!(builder.stop_price, None);

        // Stop order (stop price only)
        let builder = PlaceOrderBuilderWithClient::new(&client)
            .ticker_id(123)
            .stop(145.0)
            .sell()
            .quantity(10.0);
        assert_eq!(builder.limit_price, None);
        assert_eq!(builder.stop_price, Some(145.0));

        // Stop-limit order (both prices)
        let builder = PlaceOrderBuilderWithClient::new(&client)
            .ticker_id(123)
            .limit(144.0)
            .stop(145.0)
            .sell()
            .quantity(10.0);
        assert_eq!(builder.limit_price, Some(144.0));
        assert_eq!(builder.stop_price, Some(145.0));
    }

    #[test]
    fn test_place_order_builder_explicit_types() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test market order constructor
        let builder = PlaceOrderBuilderWithClient::market(&client);
        assert_eq!(builder.order_type, Some(OrderType::Market));

        // Test limit order constructor
        let builder = PlaceOrderBuilderWithClient::limit_order(&client, 150.0);
        assert_eq!(builder.order_type, Some(OrderType::Limit));
        assert_eq!(builder.limit_price, Some(150.0));

        // Test stop order constructor
        let builder = PlaceOrderBuilderWithClient::stop_order(&client, 145.0);
        assert_eq!(builder.order_type, Some(OrderType::Stop));
        assert_eq!(builder.stop_price, Some(145.0));

        // Test stop-limit order constructor
        let builder = PlaceOrderBuilderWithClient::stop_limit_order(&client, 145.0, 144.0);
        assert_eq!(builder.order_type, Some(OrderType::StopLimit));
        assert_eq!(builder.stop_price, Some(145.0));
        assert_eq!(builder.limit_price, Some(144.0));
    }

    #[test]
    fn test_place_order_builder_actions() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test buy() method
        let builder = PlaceOrderBuilderWithClient::new(&client).buy();
        assert_eq!(builder.action, Some(OrderAction::Buy));

        // Test sell() method
        let builder = PlaceOrderBuilderWithClient::new(&client).sell();
        assert_eq!(builder.action, Some(OrderAction::Sell));

        // Test action() method
        let builder = PlaceOrderBuilderWithClient::new(&client).action(OrderAction::Buy);
        assert_eq!(builder.action, Some(OrderAction::Buy));
    }

    #[test]
    fn test_place_order_builder_options() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        let builder = PlaceOrderBuilderWithClient::new(&client)
            .ticker_id(123)
            .quantity(10.0)
            .time_in_force(TimeInForce::GoodTillCancel)
            .extended_hours()
            .serial_id("test_serial".to_string())
            .combo_type("test_combo".to_string());

        assert_eq!(builder.ticker_id, Some(123));
        assert_eq!(builder.quantity, Some(10.0));
        assert_eq!(builder.time_in_force, TimeInForce::GoodTillCancel);
        assert_eq!(builder.outside_regular_trading_hour, true);
        assert_eq!(builder.serial_id, Some("test_serial".to_string()));
        assert_eq!(builder.combo_type, Some("test_combo".to_string()));
    }

    #[test]
    fn test_place_order_builder_aliases() {
        let client = WebullClient::new_paper(Some(6)).unwrap();

        // Test that limit_price and limit do the same thing
        let builder1 = PlaceOrderBuilderWithClient::new(&client).limit(150.0);
        let builder2 = PlaceOrderBuilderWithClient::new(&client).limit_price(150.0);
        assert_eq!(builder1.limit_price, builder2.limit_price);

        // Test that stop_price and stop do the same thing
        let builder1 = PlaceOrderBuilderWithClient::new(&client).stop(145.0);
        let builder2 = PlaceOrderBuilderWithClient::new(&client).stop_price(145.0);
        assert_eq!(builder1.stop_price, builder2.stop_price);
    }

    #[test]
    fn test_order_type_detection_logic() {
        // This tests the exact logic that would be used in IntoFuture
        fn detect_order_type(has_limit: bool, has_stop: bool) -> OrderType {
            match (has_limit, has_stop) {
                (true, true) => OrderType::StopLimit,
                (true, false) => OrderType::Limit,
                (false, true) => OrderType::Stop,
                (false, false) => OrderType::Market,
            }
        }

        assert_eq!(detect_order_type(false, false), OrderType::Market);
        assert_eq!(detect_order_type(true, false), OrderType::Limit);
        assert_eq!(detect_order_type(false, true), OrderType::Stop);
        assert_eq!(detect_order_type(true, true), OrderType::StopLimit);
    }
}
