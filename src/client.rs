use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::{json, Value};
use crate::{
    endpoints::Endpoints,
    error::{Result, WebullError},
    models::*,
    utils::*,
};

#[derive(Debug, Clone)]
pub struct WebullClient {
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

impl WebullClient {
    /// Create a new Webull client
    pub fn new(region_code: Option<i32>) -> Result<Self> {
        let did = get_did(None)?;
        let mut headers = HeaderMap::new();
        
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:99.0) Gecko/20100101 Firefox/99.0"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.5"));
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
        self.headers.insert("did", HeaderValue::from_str(did).unwrap());
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
    fn build_req_headers(&self, include_trade_token: bool, include_time: bool, include_zone_var: bool) -> HeaderMap {
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
            return Err(WebullError::InvalidParameter("Username or password is empty".to_string()));
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
            data["accessQuestions"] = json!(format!("[{{\"questionId\":\"{}\", \"answer\":\"{}\"}}]", qid, qanswer));
        }

        println!("Login endpoint: {}", self.endpoints.login());
        println!("Login data: {}", serde_json::to_string_pretty(&data).unwrap_or_default());
        
        let response = self.client
            .post(&self.endpoints.login())
            .headers(headers)
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let status = response.status();
        let result: Value = response.json().await?;
        
        println!("Response status: {}", status);
        println!("Response body: {}", serde_json::to_string_pretty(&result).unwrap_or_default());
        
        if let Some(access_token) = result.get("accessToken").and_then(|v| v.as_str()) {
            self.access_token = Some(access_token.to_string());
            self.refresh_token = result.get("refreshToken").and_then(|v| v.as_str()).map(|s| s.to_string());
            self.token_expire = result.get("tokenExpireTime").and_then(|v| v.as_i64());
            self.uuid = result.get("uuid").and_then(|v| v.as_str()).map(|s| s.to_string());
            
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

        let response = self.client
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

        let response = self.client
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
        
        let response = self.client
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
        let refresh_token = self.refresh_token.as_ref()
            .ok_or(WebullError::SessionExpired)?;

        let response = self.client
            .post(&self.endpoints.refresh_login(refresh_token))
            .headers(self.headers.clone())
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(access_token) = result.get("accessToken").and_then(|v| v.as_str()) {
            self.access_token = Some(access_token.to_string());
            self.refresh_token = result.get("refreshToken").and_then(|v| v.as_str()).map(|s| s.to_string());
            self.token_expire = result.get("tokenExpireTime").and_then(|v| v.as_i64());
            
            Ok(serde_json::from_value(result)?)
        } else {
            Err(WebullError::SessionExpired)
        }
    }

    /// Get account ID
    pub async fn get_account_id(&mut self) -> Result<String> {
        let headers = self.build_req_headers(false, false, true);
        
        println!("Getting account ID from: {}", self.endpoints.account_id());
        
        let response = self.client
            .get(&self.endpoints.account_id())
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let status = response.status();
        let result: Value = response.json().await?;
        
        println!("Account ID response status: {}", status);
        println!("Account ID response: {}", serde_json::to_string_pretty(&result).unwrap_or_default());
        
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
        
        let response = self.client
            .post(&self.endpoints.trade_token())
            .headers(headers)
            .json(&data)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(trade_token) = result.get("data").and_then(|d| d.get("tradeToken")).and_then(|v| v.as_str()) {
            self.trade_token = Some(trade_token.to_string());
            Ok(trade_token.to_string())
        } else {
            Err(WebullError::AuthenticationError("Failed to get trade token".to_string()))
        }
    }

    /// Get account details
    pub async fn get_account(&self) -> Result<AccountDetail> {
        let account_id = self.account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(false, false, true);
        
        let response = self.client
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
        let account_id = self.account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.build_req_headers(false, false, true);
        
        let url = format!("{}/v2/home/{}", self.endpoints.base_trade_url, account_id);
        
        let response = self.client
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

    /// Get orders
    pub async fn get_orders(&self, page_size: Option<i32>) -> Result<Vec<Order>> {
        let account_id = self.account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let page_size = page_size.unwrap_or(100);
        let headers = self.build_req_headers(false, false, true);
        
        let response = self.client
            .get(&self.endpoints.orders(account_id, page_size))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(orders) = result.get("data") {
            Ok(serde_json::from_value(orders.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Place order
    pub async fn place_order(&self, order: &PlaceOrderRequest) -> Result<String> {
        let account_id = self.account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        if self.trade_token.is_none() {
            return Err(WebullError::TradeTokenNotAvailable);
        }

        let headers = self.build_req_headers(true, true, true);
        
        let response = self.client
            .post(&self.endpoints.place_orders(account_id))
            .headers(headers)
            .json(order)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(order_id) = result.get("data").and_then(|d| d.get("orderId")).and_then(|v| v.as_str()) {
            Ok(order_id.to_string())
        } else {
            Err(WebullError::ApiError("Failed to place order".to_string()))
        }
    }

    /// Cancel order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        let account_id = self.account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        if self.trade_token.is_none() {
            return Err(WebullError::TradeTokenNotAvailable);
        }

        let headers = self.build_req_headers(true, true, true);
        
        let url = format!("{}{}", self.endpoints.cancel_order(account_id), order_id);
        
        let response = self.client
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
        
        let response = self.client
            .get(&self.endpoints.quotes(ticker_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get bars/candles
    pub async fn get_bars(&self, ticker_id: &str, interval: &str, count: i32, timestamp: Option<i64>) -> Result<Vec<Bar>> {
        let interval = parse_interval(interval)?;
        let headers = self.build_req_headers(false, false, true);
        
        let response = self.client
            .get(&self.endpoints.bars(ticker_id, &interval, count, timestamp))
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

    /// Search ticker
    pub async fn find_ticker(&self, keyword: &str) -> Result<Vec<Ticker>> {
        let headers = self.build_req_headers(false, false, true);
        
        let response = self.client
            .get(&self.endpoints.stock_id(keyword, self.region_code))
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

    /// Get option chains
    pub async fn get_options(&self, ticker: &str) -> Result<Vec<OptionContract>> {
        let headers = self.build_req_headers(false, false, true);
        
        let response = self.client
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
        
        let response = self.client
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
        
        let response = self.client
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
        
        let response = self.client
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
    base_client: WebullClient,
    paper_account_id: Option<String>,
}

impl PaperWebullClient {
    /// Create a new paper trading client
    pub fn new(region_code: Option<i32>) -> Result<Self> {
        Ok(Self {
            base_client: WebullClient::new(region_code)?,
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
        let result = self.base_client.login(username, password, device_name, mfa, question_id, question_answer).await?;
        self.get_paper_account_id().await?;
        Ok(result)
    }

    /// Get paper account ID
    async fn get_paper_account_id(&mut self) -> Result<String> {
        let headers = self.base_client.build_req_headers(false, false, true);
        
        let response = self.base_client.client
            .get(&self.base_client.endpoints.paper_account_id())
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(data) = result.get("data").and_then(|v| v.as_array()) {
            if let Some(first_account) = data.first() {
                if let Some(paper_id) = first_account.get("paperId").and_then(|v| v.as_str()) {
                    self.paper_account_id = Some(paper_id.to_string());
                    return Ok(paper_id.to_string());
                }
            }
        }
        
        Err(WebullError::AccountNotFound)
    }

    /// Get paper account details
    pub async fn get_account(&self) -> Result<AccountDetail> {
        let paper_account_id = self.paper_account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, false, true);
        
        let response = self.base_client.client
            .get(&self.base_client.endpoints.paper_account(paper_account_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Place paper order
    pub async fn place_order(&self, ticker_id: &str, order: &PlaceOrderRequest) -> Result<String> {
        let paper_account_id = self.paper_account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, true, true);
        
        let response = self.base_client.client
            .post(&self.base_client.endpoints.paper_place_order(paper_account_id, ticker_id))
            .headers(headers)
            .json(order)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(order_id) = result.get("data").and_then(|d| d.get("orderId")).and_then(|v| v.as_str()) {
            Ok(order_id.to_string())
        } else {
            Err(WebullError::ApiError("Failed to place paper order".to_string()))
        }
    }

    /// Cancel paper order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        let paper_account_id = self.paper_account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, true, true);
        
        let response = self.base_client.client
            .post(&self.base_client.endpoints.paper_cancel_order(paper_account_id, order_id))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get paper orders
    pub async fn get_orders(&self, page_size: Option<i32>) -> Result<Vec<Order>> {
        let paper_account_id = self.paper_account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let page_size = page_size.unwrap_or(100);
        let headers = self.base_client.build_req_headers(false, false, true);
        
        let response = self.base_client.client
            .get(&self.base_client.endpoints.paper_orders(paper_account_id, page_size))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.base_client.timeout))
            .send()
            .await?;

        let result: Value = response.json().await?;
        
        if let Some(orders) = result.get("data") {
            Ok(serde_json::from_value(orders.clone())?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Delegate other methods to base client
    pub async fn get_quotes(&self, ticker_id: &str) -> Result<Quote> {
        self.base_client.get_quotes(ticker_id).await
    }

    pub async fn get_bars(&self, ticker_id: &str, interval: &str, count: i32, timestamp: Option<i64>) -> Result<Vec<Bar>> {
        self.base_client.get_bars(ticker_id, interval, count, timestamp).await
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

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        // For paper trading, we need to get positions from paper account
        let paper_account_id = self.paper_account_id.as_ref()
            .ok_or(WebullError::AccountNotFound)?;

        let headers = self.base_client.build_req_headers(false, false, true);
        
        let response = self.base_client.client
            .get(&format!("{}/paper/1/acc/{}/positions", self.base_client.endpoints.base_paper_url, paper_account_id))
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