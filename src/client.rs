use crate::{
    endpoints::Endpoints,
    error::{Result, WebullError},
    models::*,
    utils::*,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct LiveWebullClient {
    client: Client,
    endpoints: Endpoints,
    headers: HeaderMap,

    // Session data
    pub(crate) account_id: Option<String>,
    trade_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_expire: Option<i64>,
    uuid: Option<String>,

    // Configuration
    pub(crate) did: String,
    pub(crate) region_code: i32,
    zone_var: String,
    timeout: u64,
}

impl LiveWebullClient {
    /// Create a new Webull client
    pub fn new(region_code: Option<i32>) -> Result<Self> {
        let did = get_did(None)?;
        let mut headers = HeaderMap::new();

        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:99.0) Gecko/20100101 Firefox/99.0"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate"));
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("en-US,en;q=0.5"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("platform", HeaderValue::from_static("web"));
        headers.insert("hl", HeaderValue::from_static("en"));
        headers.insert("os", HeaderValue::from_static("web"));
        headers.insert("osv", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:99.0) Gecko/20100101 Firefox/99.0"));
        headers.insert("app", HeaderValue::from_static("global"));
        headers.insert("appid", HeaderValue::from_static("webull-webapp"));
        headers.insert("ver", HeaderValue::from_static("3.39.18"));
        headers.insert("lzone", HeaderValue::from_static("dc_core_r001"));
        headers.insert("ph", HeaderValue::from_static("MacOS Firefox"));
        headers.insert("locale", HeaderValue::from_static("eng"));
        headers.insert("device-type", HeaderValue::from_static("Web"));
        headers.insert("did", HeaderValue::from_str(&did).unwrap());

        Ok(Self {
            client: Client::new(),
            endpoints: Endpoints::new(),
            headers,
            account_id: None,
            trade_token: None,
            access_token: None,
            refresh_token: None,
            token_expire: None,
            uuid: None,
            did,
            region_code: region_code.unwrap_or(6),
            zone_var: "dc_core_r001".to_string(),
            timeout: 15,
        })
    }

    /// Set device ID
    pub fn set_did(&mut self, did: &str, path: Option<&Path>) -> Result<()> {
        save_did(did, path)?;
        self.did = did.to_string();
        self.headers
            .insert("did", HeaderValue::from_str(did).unwrap());
        Ok(())
    }

    /// Get device ID
    pub fn get_did(&self) -> &str {
        &self.did
    }

    /// Get account ID
    pub fn get_account_id_str(&self) -> Option<&str> {
        self.account_id.as_deref()
    }

    /// Build request headers
    fn build_req_headers(
        &self,
        include_trade_token: bool,
        include_time: bool,
        include_zone_var: bool,
    ) -> HeaderMap {
        let mut headers = self.headers.clone();
        let req_id = generate_req_id();

        headers.insert("reqid", HeaderValue::from_str(&req_id).unwrap());
        headers.insert("did", HeaderValue::from_str(&self.did).unwrap());

        if let Some(access_token) = &self.access_token {
            headers.insert("access_token", HeaderValue::from_str(access_token).unwrap());
        }

        if include_trade_token {
            if let Some(trade_token) = &self.trade_token {
                headers.insert("t_token", HeaderValue::from_str(trade_token).unwrap());
            }
        }

        if include_time {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string();
            headers.insert("t_time", HeaderValue::from_str(&timestamp).unwrap());
        }

        if include_zone_var {
            headers.insert("lzone", HeaderValue::from_str(&self.zone_var).unwrap());
        }

        headers
    }

    /// Login to Webull
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        device_name: Option<&str>,
        mfa: Option<&str>,
        question_id: Option<&str>,
        question_answer: Option<&str>,
    ) -> Result<LoginResponse> {
        if username.is_empty() || password.is_empty() {
            return Err(WebullError::InvalidParameter(
                "Username or password is empty".to_string(),
            ));
        }

        let hashed_password = hash_password(password);
        let account_type = get_account_type(username)?;
        let device_name = device_name.unwrap_or("default_string");

        let mut data = json!({
            "account": username,
            "accountType": account_type.to_string(),
            "deviceId": self.did,
            "deviceName": device_name,
            "grade": 1,
            "pwd": hashed_password,
            "regionId": self.region_code
        });

        let headers = if let Some(mfa_code) = mfa {
            data["extInfo"] = json!({
                "codeAccountType": account_type,
                "verificationCode": mfa_code
            });
            self.build_req_headers(false, false, true)
        } else {
            self.headers.clone()
        };

        if let (Some(qid), Some(qanswer)) = (question_id, question_answer) {
            data["accessQuestions"] = json!(format!(
                "[{{\"questionId\":\"{}\", \"answer\":\"{}\"}}]",
                qid, qanswer
            ));
        }

        let response = self
            .client
            .post(&self.endpoints.login())
            .headers(headers)
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(access_token) = result.get("accessToken").and_then(|v| v.as_str()) {
            self.access_token = Some(access_token.to_string());
            self.refresh_token = result
                .get("refreshToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            // Parse tokenExpireTime - try as i64 first, then as string date
            self.token_expire = result.get("tokenExpireTime").and_then(|v| {
                v.as_i64().or_else(|| {
                    v.as_str().and_then(|s| {
                        // Try to parse ISO 8601 date string to timestamp
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.timestamp())
                    })
                })
            });
            self.uuid = result
                .get("uuid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Get account ID after successful login
            self.get_account_id().await?;

            Ok(serde_json::from_value(result)?)
        } else {
            Err(WebullError::AuthenticationError("Login failed".to_string()))
        }
    }

    /// Get MFA code
    pub async fn get_mfa(&self, username: &str) -> Result<bool> {
        let account_type = get_account_type(username)?;

        let data = json!({
            "account": username,
            "accountType": account_type.to_string(),
            "codeType": 5
        });

        let response = self
            .client
            .post(&self.endpoints.get_mfa())
            .headers(self.headers.clone())
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Check MFA code
    pub async fn check_mfa(&self, username: &str, mfa: &str) -> Result<bool> {
        let account_type = get_account_type(username)?;

        let data = json!({
            "account": username,
            "accountType": account_type.to_string(),
            "code": mfa,
            "codeType": 5
        });

        let response = self
            .client
            .post(&self.endpoints.check_mfa())
            .headers(self.headers.clone())
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Logout
    pub async fn logout(&mut self) -> Result<bool> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .post(&self.endpoints.logout())
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        if response.status().is_success() {
            self.access_token = None;
            self.refresh_token = None;
            self.trade_token = None;
            self.account_id = None;
            self.token_expire = None;
            self.uuid = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Refresh login token
    pub async fn refresh_login(&mut self) -> Result<LoginResponse> {
        let refresh_token = self
            .refresh_token
            .as_ref()
            .ok_or(WebullError::SessionExpired)?;

        let response = self
            .client
            .post(&self.endpoints.refresh_login(refresh_token))
            .headers(self.headers.clone())
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(access_token) = result.get("accessToken").and_then(|v| v.as_str()) {
            self.access_token = Some(access_token.to_string());
            self.refresh_token = result
                .get("refreshToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            // Parse tokenExpireTime - try as i64 first, then as string date
            self.token_expire = result.get("tokenExpireTime").and_then(|v| {
                v.as_i64().or_else(|| {
                    v.as_str().and_then(|s| {
                        // Try to parse ISO 8601 date string to timestamp
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.timestamp())
                    })
                })
            });

            Ok(serde_json::from_value(result)?)
        } else {
            Err(WebullError::SessionExpired)
        }
    }

    /// Get account ID
    pub async fn get_account_id(&mut self) -> Result<String> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.account_id())
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(data) = result.get("data").and_then(|v| v.as_array()) {
            if let Some(first_account) = data.first() {
                // Try to get secAccountId as either a string or number
                if let Some(account_id) = first_account.get("secAccountId") {
                    let account_id_str = match account_id {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => return Err(WebullError::AccountNotFound),
                    };
                    self.account_id = Some(account_id_str.clone());
                    return Ok(account_id_str);
                }
            }
        }

        Err(WebullError::AccountNotFound)
    }

    /// Get trade token
    pub async fn get_trade_token(&mut self, password: &str) -> Result<String> {
        let hashed_password = hash_password(password);

        let data = json!({
            "pwd": hashed_password
        });

        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .post(&self.endpoints.trade_token())
            .headers(headers)
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(trade_token) = result
            .get("data")
            .and_then(|d| d.get("tradeToken"))
            .and_then(|v| v.as_str())
        {
            self.trade_token = Some(trade_token.to_string());
            Ok(trade_token.to_string())
        } else {
            Err(WebullError::AuthenticationError(
                "Failed to get trade token".to_string(),
            ))
        }
    }

    /// Get account details
    pub async fn get_account(&self) -> Result<AccountDetail> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.account(account_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get positions
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(false, false, true);

        let url = format!("{}/v2/home/{}", self.endpoints.base_trade_url, account_id);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(positions) = result.get("positions") {
            Ok(serde_json::from_value(positions.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get current open orders (from account data)
    pub async fn get_orders(&self, _page_size: Option<i32>) -> Result<Vec<Order>> {
        // Get account data which contains openOrders
        let account_data = self.get_account_raw().await?;
        
        // Extract openOrders from the account data
        if let Some(open_orders) = account_data.get("openOrders") {
            // Parse the orders array
            if let Ok(orders) = serde_json::from_value::<Vec<Order>>(open_orders.clone()) {
                Ok(orders)
            } else {
                // If parsing fails, return empty vec
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get account data as raw JSON (for extracting openOrders)
    async fn get_account_raw(&self) -> Result<Value> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.account(account_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        Ok(response.json().await?)
    }
    
    /// Get historical orders
    pub async fn get_history_orders(&self, status: &str, count: i32) -> Result<Value> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(true, false, true);

        let url = format!(
            "{}{}",
            self.endpoints.orders(account_id, count),
            status
        );

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Place order
    pub async fn place_order(&self, order: &PlaceOrderRequest) -> Result<String> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        if self.trade_token.is_none() {
            return Err(WebullError::TradeTokenNotAvailable);
        }

        let headers = self.build_req_headers(true, true, true);

        // Create order data with proper formatting
        let mut order_data = serde_json::to_value(order)?;

        // Add required fields for live trading
        order_data["comboType"] = json!("NORMAL");

        // Add serialId if not present
        if order_data.get("serialId").is_none() {
            let uuid = uuid::Uuid::new_v4().to_string();
            order_data["serialId"] = json!(uuid);
        }

        // Handle different order types
        match order.order_type {
            OrderType::Market => {
                // Market orders do not support extended hours
                order_data["outsideRegularTradingHour"] = json!(false);
            }
            OrderType::Limit => {
                // Add lmtPrice for limit orders
                if let Some(limit_price) = order.limit_price {
                    order_data["lmtPrice"] = json!(limit_price);
                }
            }
            OrderType::Stop => {
                // Add auxPrice for stop orders
                if let Some(stop_price) = order.stop_price {
                    order_data["auxPrice"] = json!(stop_price);
                }
            }
            OrderType::StopLimit => {
                // Add both lmtPrice and auxPrice for stop limit orders
                if let Some(limit_price) = order.limit_price {
                    order_data["lmtPrice"] = json!(limit_price);
                }
                if let Some(stop_price) = order.stop_price {
                    order_data["auxPrice"] = json!(stop_price);
                }
            }
        }

        let response = self
            .client
            .post(&self.endpoints.place_orders(account_id))
            .headers(headers)
            .json(&order_data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        // Check for orderId in data field or directly in result
        let order_id = result
            .get("data")
            .and_then(|d| d.get("orderId"))
            .or_else(|| result.get("orderId"));

        if let Some(order_id_val) = order_id {
            // Handle both string and number formats
            let order_id_str = match order_id_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => return Err(WebullError::ApiError("Invalid orderId format".to_string())),
            };
            Ok(order_id_str)
        } else {
            Err(WebullError::ApiError("Failed to place order".to_string()))
        }
    }

    /// Cancel order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        if self.trade_token.is_none() {
            return Err(WebullError::TradeTokenNotAvailable);
        }

        let headers = self.build_req_headers(true, true, true);

        let url = format!("{}{}", self.endpoints.cancel_order(account_id), order_id);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get quotes
    pub async fn get_quotes(&self, ticker_id: &str) -> Result<Quote> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.quotes(ticker_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get bars/candles
    pub async fn get_bars(
        &self,
        ticker_id: &str,
        interval: &str,
        count: i32,
        timestamp: Option<i64>,
    ) -> Result<Vec<Bar>> {
        let interval = parse_interval(interval)?;
        let headers = self.build_req_headers(false, false, true);
        
        // Use current timestamp if not provided (like Python does)
        let timestamp = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        });

        let url = self.endpoints.bars(ticker_id, &interval, count, Some(timestamp));

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        // Parse bars from the response
        // The response is an array with the first element containing the data
        if let Some(result_array) = result.as_array() {
            if let Some(first_item) = result_array.first() {
                if let Some(data_array) = first_item.get("data").and_then(|v| v.as_array()) {
                    let mut bars = Vec::new();
                    for data_str in data_array {
                        if let Some(s) = data_str.as_str() {
                            // Parse comma-separated values: timestamp,open,close,high,low,?,volume,vwap
                            let parts: Vec<&str> = s.split(',').collect();
                            if parts.len() >= 7 {
                                let timestamp = parts[0].parse::<i64>().unwrap_or(0);
                                let open = parts[1].parse::<f64>().unwrap_or(0.0);
                                let close = parts[2].parse::<f64>().unwrap_or(0.0);
                                let high = parts[3].parse::<f64>().unwrap_or(0.0);
                                let low = parts[4].parse::<f64>().unwrap_or(0.0);
                                let volume = parts[6].parse::<i64>().unwrap_or(0);
                                let vwap = if parts.len() > 7 && parts[7] != "null" {
                                    parts[7].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                
                                bars.push(Bar {
                                    open,
                                    high,
                                    low,
                                    close,
                                    volume,
                                    vwap,
                                    timestamp,
                                });
                            }
                        }
                    }
                    return Ok(bars);
                }
            }
        }
        
        Ok(Vec::new())
    }

    /// Search ticker
    pub async fn find_ticker(&self, keyword: &str) -> Result<Vec<Ticker>> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.stock_id(keyword, self.region_code))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        // println!("Ticker search response: {}", serde_json::to_string_pretty(&result).unwrap_or_default());

        if let Some(data) = result.get("data") {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get option chains
    pub async fn get_options(&self, ticker: &str) -> Result<Vec<OptionContract>> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.options(ticker))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(data) = result.get("data") {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get news
    pub async fn get_news(&self, ticker: &str, last_id: i64, count: i32) -> Result<Vec<News>> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.news(ticker, last_id, count))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(data) = result.get("data") {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get fundamentals
    pub async fn get_fundamentals(&self, ticker: &str) -> Result<Fundamental> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .get(&self.endpoints.fundamentals(ticker))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Run screener
    pub async fn screener(&self, request: &ScreenerRequest) -> Result<Vec<Ticker>> {
        let headers = self.build_req_headers(false, false, true);

        let response = self
            .client
            .post(&self.endpoints.screener())
            .headers(headers)
            .json(request)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(data) = result.get("data") {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Paper trading client
#[derive(Debug, Clone)]
pub struct PaperWebullClient {
    base_client: LiveWebullClient,
    paper_account_id: Option<String>,
}

impl PaperWebullClient {
    /// Create a new paper trading client
    pub fn new(region_code: Option<i32>) -> Result<Self> {
        Ok(Self {
            base_client: LiveWebullClient::new(region_code)?,
            paper_account_id: None,
        })
    }

    /// Login (delegates to base client)
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        device_name: Option<&str>,
        mfa: Option<&str>,
        question_id: Option<&str>,
        question_answer: Option<&str>,
    ) -> Result<LoginResponse> {
        let result = self
            .base_client
            .login(
                username,
                password,
                device_name,
                mfa,
                question_id,
                question_answer,
            )
            .await?;
        self.get_paper_account_id().await?;
        Ok(result)
    }

    /// Get paper account ID
    async fn get_paper_account_id(&mut self) -> Result<String> {
        let headers = self.base_client.build_req_headers(false, false, true);

        let response = self
            .base_client
            .client
            .get(&self.base_client.endpoints.paper_account_id())
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        // The response can be either an array directly or wrapped in "data"
        let accounts = if result.is_array() {
            result.as_array()
        } else {
            result.get("data").and_then(|v| v.as_array())
        };

        if let Some(accounts_array) = accounts {
            if let Some(first_account) = accounts_array.first() {
                // Try id as string or number (matches Python implementation)
                if let Some(paper_id) = first_account.get("id") {
                    let paper_id_str = match paper_id {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => return Err(WebullError::AccountNotFound),
                    };
                    self.paper_account_id = Some(paper_id_str.clone());
                    return Ok(paper_id_str);
                }
            }
        }

        Err(WebullError::AccountNotFound)
    }

    /// Get paper account details
    pub async fn get_account(&self) -> Result<AccountDetail> {
        let paper_account_id = self
            .paper_account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, false, true);

        let response = self
            .base_client
            .client
            .get(&self.base_client.endpoints.paper_account(paper_account_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        // println!(
        //     "Paper account details response: {}",
        //     serde_json::to_string_pretty(&result).unwrap_or_default()
        // );
        Ok(serde_json::from_value(result)?)
    }

    /// Place paper order
    pub async fn place_order(&self, order: &PlaceOrderRequest) -> Result<String> {
        let paper_account_id = self
            .paper_account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        // Paper orders need trade token and time headers
        let headers = self.base_client.build_req_headers(true, true, true);

        // Create a modified order with serialId if not present and handle market orders
        let mut order_data = serde_json::to_value(order)?;

        // Add serialId if not present
        if order_data.get("serialId").is_none() {
            let uuid = uuid::Uuid::new_v4().to_string();
            order_data["serialId"] = serde_json::Value::String(uuid);
        }

        // For market orders, force outsideRegularTradingHour to false
        if matches!(order.order_type, OrderType::Market) {
            order_data["outsideRegularTradingHour"] = serde_json::Value::Bool(false);
        }

        // Add lmtPrice for limit orders
        if let Some(limit_price) = order.limit_price {
            order_data["lmtPrice"] = serde_json::Value::from(limit_price);
        }

        let response = self
            .base_client
            .client
            .post(
                &self
                    .base_client
                    .endpoints
                    .paper_place_order(paper_account_id, &order.ticker_id.to_string()),
            )
            .headers(headers)
            .json(&order_data)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        // Check for orderId directly in result or in data field
        let order_id = result
            .get("orderId")
            .or_else(|| result.get("data").and_then(|d| d.get("orderId")));

        if let Some(order_id_val) = order_id {
            // Handle both string and number formats
            let order_id_str = match order_id_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => return Err(WebullError::ApiError("Invalid orderId format".to_string())),
            };
            Ok(order_id_str)
        } else {
            Err(WebullError::ApiError(
                "Failed to place paper order".to_string(),
            ))
        }
    }

    /// Cancel paper order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        let paper_account_id = self
            .paper_account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, true, true);

        let response = self
            .base_client
            .client
            .post(
                &self
                    .base_client
                    .endpoints
                    .paper_cancel_order(paper_account_id, order_id),
            )
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get paper orders (current open orders)
    pub async fn get_orders(&self, page_size: Option<i32>) -> Result<Vec<Order>> {
        // Paper trading doesn't return openOrders in account data like live trading does
        // Instead, we need to get all orders and filter for "Working" status
        let history = self.get_history_orders("All", page_size.unwrap_or(100)).await?;
        
        // Parse the response and filter for Working orders
        if let Some(orders_array) = history.as_array() {
            let mut working_orders = Vec::new();
            
            for order_val in orders_array {
                // Check if status is "Working"
                if let Some(status) = order_val.get("status").and_then(|s| s.as_str()) {
                    if status == "Working" {
                        // Try to parse this into our Order struct
                        // For now, we'll need to manually construct it since the format is different
                        if let Ok(order) = self.parse_paper_order(order_val) {
                            working_orders.push(order);
                        }
                    }
                }
            }
            Ok(working_orders)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Helper to parse paper order from JSON
    fn parse_paper_order(&self, order_val: &Value) -> Result<Order> {
        use chrono::{DateTime, Utc};
        
        let order_id = order_val.get("orderId")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .ok_or(WebullError::ParseError("Missing orderId".to_string()))?;
            
        let ticker_data = order_val.get("ticker")
            .ok_or(WebullError::ParseError("Missing ticker".to_string()))?;
            
        let ticker = serde_json::from_value::<Ticker>(ticker_data.clone())?;
        
        let action = match order_val.get("action").and_then(|v| v.as_str()) {
            Some("BUY") => OrderAction::Buy,
            Some("SELL") => OrderAction::Sell,
            _ => return Err(WebullError::ParseError("Invalid action".to_string())),
        };
        
        let order_type = match order_val.get("orderType").and_then(|v| v.as_str()) {
            Some("MKT") => OrderType::Market,
            Some("LMT") => OrderType::Limit,
            Some("STP") => OrderType::Stop,
            Some("STP LMT") => OrderType::StopLimit,
            _ => return Err(WebullError::ParseError("Invalid order type".to_string())),
        };
        
        let status = match order_val.get("status").and_then(|v| v.as_str()) {
            Some("Working") => OrderStatus::Working,
            Some("Filled") => OrderStatus::Filled,
            Some("Canceled") | Some("Cancelled") => OrderStatus::Cancelled,
            Some("PartiallyFilled") | Some("Partial Filled") => OrderStatus::PartialFilled,
            Some("Pending") => OrderStatus::Pending,
            Some("Failed") => OrderStatus::Failed,
            _ => OrderStatus::Working,
        };
        
        let time_in_force = match order_val.get("timeInForce").and_then(|v| v.as_str()) {
            Some("DAY") => TimeInForce::Day,
            Some("GTC") => TimeInForce::GoodTillCancel,
            Some("IOC") => TimeInForce::ImmediateOrCancel,
            Some("FOK") => TimeInForce::FillOrKill,
            _ => TimeInForce::Day,
        };
        
        let quantity = order_val.get("totalQuantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
            
        let filled_quantity = order_val.get("filledQuantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
            
        let limit_price = order_val.get("lmtPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
            
        let stop_price = order_val.get("auxPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
            
        let avg_fill_price = order_val.get("avgFilledPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
            
        // Parse placed time
        let placed_time = if let Some(timestamp) = order_val.get("createTime0").and_then(|v| v.as_i64()) {
            DateTime::from_timestamp_millis(timestamp).unwrap_or(Utc::now())
        } else {
            Utc::now()
        };
        
        let filled_time = if let Some(timestamp) = order_val.get("filledTime0").and_then(|v| v.as_i64()) {
            Some(DateTime::from_timestamp_millis(timestamp).unwrap_or(Utc::now()))
        } else {
            None
        };
        
        let outside_regular_trading_hour = order_val.get("outsideRegularTradingHour")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        Ok(Order {
            order_id,
            combo_id: order_val.get("comboId").and_then(|v| v.as_str()).map(String::from),
            ticker,
            action,
            order_type,
            status,
            time_in_force,
            quantity,
            filled_quantity,
            avg_fill_price,
            limit_price,
            stop_price,
            placed_time,
            filled_time,
            outside_regular_trading_hour,
        })
    }
    
    /// Get historical paper orders
    pub async fn get_history_orders(&self, status: &str, count: i32) -> Result<Value> {
        let paper_account_id = self
            .paper_account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(true, false, true);

        let url = format!(
            "{}{}",
            self.base_client.endpoints.paper_orders(paper_account_id, count),
            status
        );

        let response = self
            .base_client
            .client
            .get(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Delegate other methods to base client
    pub async fn get_quotes(&self, ticker_id: &str) -> Result<Quote> {
        self.base_client.get_quotes(ticker_id).await
    }

    pub async fn get_bars(
        &self,
        ticker_id: &str,
        interval: &str,
        count: i32,
        timestamp: Option<i64>,
    ) -> Result<Vec<Bar>> {
        self.base_client
            .get_bars(ticker_id, interval, count, timestamp)
            .await
    }

    pub async fn find_ticker(&self, keyword: &str) -> Result<Vec<Ticker>> {
        self.base_client.find_ticker(keyword).await
    }

    pub async fn get_news(&self, ticker: &str, last_id: i64, count: i32) -> Result<Vec<News>> {
        self.base_client.get_news(ticker, last_id, count).await
    }

    pub async fn get_fundamentals(&self, ticker: &str) -> Result<Fundamental> {
        self.base_client.get_fundamentals(ticker).await
    }

    pub async fn logout(&mut self) -> Result<bool> {
        self.base_client.logout().await
    }

    pub async fn get_trade_token(&mut self, password: &str) -> Result<String> {
        self.base_client.get_trade_token(password).await
    }

    pub fn get_did(&self) -> &str {
        self.base_client.get_did()
    }

    pub fn get_account_id_str(&self) -> Option<String> {
        self.paper_account_id.clone()
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        // For paper trading, we need to get positions from paper account
        let paper_account_id = self
            .paper_account_id
            .as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, false, true);

        let response = self
            .base_client
            .client
            .get(&format!(
                "{}/paper/1/acc/{}/positions",
                self.base_client.endpoints.base_paper_url, paper_account_id
            ))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(positions) = result.get("data") {
            Ok(serde_json::from_value(positions.clone())?)
        } else {
            Ok(Vec::new())
        }
    }
}

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
                client.login(username, password, device_name, mfa, question_id, question_answer).await
            }
            WebullClient::Paper(client) => {
                client.login(username, password, device_name, mfa, question_id, question_answer).await
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
            WebullClient::Live(client) => client.get_bars(ticker_id, interval, count, timestamp).await,
            WebullClient::Paper(client) => client.get_bars(ticker_id, interval, count, timestamp).await,
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
