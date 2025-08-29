use crate::{
    builders::*,
    error::{Result, WebullError},
    live_client::LiveWebullClient,
    models::*,
    paper_client::PaperWebullClient,
};
use serde_json::Value;

/// Unified Webull client that can work with both live and paper trading
#[derive(Debug, Clone)]
pub enum WebullClient {
    Live(LiveWebullClient),
    Paper(PaperWebullClient),
}

impl WebullClient {
    /// Create a new live trading client
    pub fn new_live(region_code: Option<i32>) -> Result<Self> {
        Ok(WebullClient::Live(LiveWebullClient::new(region_code)?))
    }

    /// Create a new paper trading client
    pub fn new_paper(region_code: Option<i32>) -> Result<Self> {
        Ok(WebullClient::Paper(PaperWebullClient::new(region_code)?))
    }

    /// Check if this is a paper trading client
    pub fn is_paper(&self) -> bool {
        matches!(self, WebullClient::Paper(_))
    }

    /// Login to the account
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        device_name: Option<&str>,
        mfa: Option<&str>,
        question_id: Option<&str>,
        question_answer: Option<&str>,
    ) -> Result<LoginResponse> {
        match self {
            WebullClient::Live(client) => {
                client
                    .login(
                        username,
                        password,
                        device_name,
                        mfa,
                        question_id,
                        question_answer,
                    )
                    .await
            }
            WebullClient::Paper(client) => {
                client
                    .login(
                        username,
                        password,
                        device_name,
                        mfa,
                        question_id,
                        question_answer,
                    )
                    .await
            }
        }
    }

    /// Logout from the account
    pub async fn logout(&mut self) -> Result<bool> {
        match self {
            WebullClient::Live(client) => client.logout().await,
            WebullClient::Paper(client) => client.logout().await,
        }
    }

    /// Get MFA code
    pub async fn get_mfa(&self, username: &str) -> Result<bool> {
        match self {
            WebullClient::Live(client) => client.get_mfa(username).await,
            WebullClient::Paper(client) => client.base_client.get_mfa(username).await,
        }
    }

    /// Check MFA code
    pub async fn check_mfa(&self, username: &str, mfa: &str) -> Result<bool> {
        match self {
            WebullClient::Live(client) => client.check_mfa(username, mfa).await,
            WebullClient::Paper(client) => client.base_client.check_mfa(username, mfa).await,
        }
    }

    /// Refresh login token
    pub async fn refresh_login(&mut self) -> Result<LoginResponse> {
        match self {
            WebullClient::Live(client) => client.refresh_login().await,
            WebullClient::Paper(client) => client.base_client.refresh_login().await,
        }
    }

    /// Get account ID
    pub async fn get_account_id(&mut self) -> Result<String> {
        match self {
            WebullClient::Live(client) => client.get_account_id().await,
            WebullClient::Paper(client) => {
                // Paper trading returns paper account ID
                if let Some(ref id) = client.paper_account_id {
                    Ok(id.clone())
                } else {
                    Err(WebullError::AccountNotFound)
                }
            }
        }
    }

    /// Get trade token
    pub async fn get_trade_token(&mut self, password: &str) -> Result<String> {
        match self {
            WebullClient::Live(client) => client.get_trade_token(password).await,
            WebullClient::Paper(client) => client.get_trade_token(password).await,
        }
    }

    /// Get account details
    pub async fn get_account(&self) -> Result<AccountDetail> {
        match self {
            WebullClient::Live(client) => client.get_account().await,
            WebullClient::Paper(client) => client.get_account().await,
        }
    }

    /// Get positions
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        match self {
            WebullClient::Live(client) => client.get_positions().await,
            WebullClient::Paper(client) => client.get_positions().await,
        }
    }

    /// Get current orders
    pub async fn get_orders(&self, page_size: Option<i32>) -> Result<Vec<Order>> {
        match self {
            WebullClient::Live(client) => client.get_orders(page_size).await,
            WebullClient::Paper(client) => client.get_orders(page_size).await,
        }
    }

    /// Get historical orders
    pub async fn get_history_orders(&self, status: &str, count: i32) -> Result<Value> {
        match self {
            WebullClient::Live(client) => client.get_history_orders(status, count).await,
            WebullClient::Paper(client) => client.get_history_orders(status, count).await,
        }
    }

    /// Place an order
    pub async fn place_order(&self, order: &PlaceOrderRequest) -> Result<String> {
        match self {
            WebullClient::Live(client) => client.place_order(order).await,
            WebullClient::Paper(client) => client.place_order(order).await,
        }
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        match self {
            WebullClient::Live(client) => client.cancel_order(order_id).await,
            WebullClient::Paper(client) => client.cancel_order(order_id).await,
        }
    }

    /// Get quotes for a ticker
    pub async fn get_quotes(&self, ticker_id: &str) -> Result<Quote> {
        match self {
            WebullClient::Live(client) => client.get_quotes(ticker_id).await,
            WebullClient::Paper(client) => client.get_quotes(ticker_id).await,
        }
    }

    /// Get historical bars
    pub async fn get_bars(
        &self,
        ticker_id: &str,
        interval: &str,
        count: i32,
        timestamp: Option<i64>,
    ) -> Result<Vec<Bar>> {
        match self {
            WebullClient::Live(client) => {
                client.get_bars(ticker_id, interval, count, timestamp).await
            }
            WebullClient::Paper(client) => {
                client.get_bars(ticker_id, interval, count, timestamp).await
            }
        }
    }

    /// Find ticker by keyword
    pub async fn find_ticker(&self, keyword: &str) -> Result<Vec<Ticker>> {
        match self {
            WebullClient::Live(client) => client.find_ticker(keyword).await,
            WebullClient::Paper(client) => client.find_ticker(keyword).await,
        }
    }

    /// Get options chain
    pub async fn get_options(&self, ticker: &str) -> Result<Vec<OptionContract>> {
        match self {
            WebullClient::Live(client) => client.get_options(ticker).await,
            WebullClient::Paper(client) => client.base_client.get_options(ticker).await,
        }
    }

    /// Get news for a ticker
    pub async fn get_news(&self, ticker: &str, last_id: i64, count: i32) -> Result<Vec<News>> {
        match self {
            WebullClient::Live(client) => client.get_news(ticker, last_id, count).await,
            WebullClient::Paper(client) => client.get_news(ticker, last_id, count).await,
        }
    }

    /// Get bars with builder (new fluent API)
    pub fn get_bars_with(&self) -> BarsRequestBuilderWithClient<'_> {
        BarsRequestBuilderWithClient::new(self)
    }

    /// Get news with builder (new fluent API)
    pub fn get_news_with(&self) -> NewsRequestBuilderWithClient<'_> {
        NewsRequestBuilderWithClient::new(self)
    }

    /// Get options with builder (new fluent API)
    pub fn get_options_with(&self) -> OptionsRequestBuilderWithClient<'_> {
        OptionsRequestBuilderWithClient::new(self)
    }

    /// Place order with builder (auto-detects order type based on parameters)
    pub fn place_order_with(&self) -> PlaceOrderBuilderWithClient<'_> {
        PlaceOrderBuilderWithClient::new(self)
    }

    /// Place market order with builder (new fluent API)
    pub fn place_market_order_with(&self) -> PlaceOrderBuilderWithClient<'_> {
        PlaceOrderBuilderWithClient::market(self)
    }

    /// Place limit order with builder (new fluent API)
    pub fn place_limit_order_with(&self, price: f64) -> PlaceOrderBuilderWithClient<'_> {
        PlaceOrderBuilderWithClient::limit_order(self, price)
    }

    /// Place stop order with builder (new fluent API)
    pub fn place_stop_order_with(&self, price: f64) -> PlaceOrderBuilderWithClient<'_> {
        PlaceOrderBuilderWithClient::stop_order(self, price)
    }

    /// Place stop-limit order with builder (new fluent API)
    pub fn place_stop_limit_order_with(
        &self,
        stop_price: f64,
        limit_price: f64,
    ) -> PlaceOrderBuilderWithClient<'_> {
        PlaceOrderBuilderWithClient::stop_limit_order(self, stop_price, limit_price)
    }

    /// Login with builder (new fluent API)
    pub fn login_with(&mut self) -> LoginBuilderWithClient<'_> {
        LoginBuilderWithClient::new(self)
    }

    /// Get fundamentals for a ticker
    pub async fn get_fundamentals(&self, ticker: &str) -> Result<Fundamental> {
        match self {
            WebullClient::Live(client) => client.get_fundamentals(ticker).await,
            WebullClient::Paper(client) => client.get_fundamentals(ticker).await,
        }
    }

    /// Run screener
    pub async fn screener(&self, request: &ScreenerRequest) -> Result<Vec<Ticker>> {
        match self {
            WebullClient::Live(client) => client.screener(request).await,
            WebullClient::Paper(client) => client.base_client.screener(request).await,
        }
    }
}
