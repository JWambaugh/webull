use crate::{
    error::{Result, WebullError},
    live_client::LiveWebullClient,
    models::{AccountDetail, AccountMember, *},
};
use serde_json::Value;

/// Paper trading client
#[derive(Debug, Clone)]
pub struct PaperWebullClient {
    pub(crate) base_client: LiveWebullClient,
    pub(crate) paper_account_id: Option<String>,
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

    /// Login using builder pattern
    pub async fn login_with(
        &mut self,
        builder: crate::models::LoginRequestBuilder,
    ) -> Result<LoginResponse> {
        let (username, password, device_name, mfa, question_id, question_answer) = builder
            .build()
            .map_err(|e| WebullError::InvalidRequest(e))?;
        self.login(
            &username,
            &password,
            device_name.as_deref(),
            mfa.as_deref(),
            question_id.as_deref(),
            question_answer.as_deref(),
        )
        .await
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
        // Debug: Remove or comment out in production
        // println!("Paper account raw response: {:?}", result);

        // Create a new AccountDetail with paper account values
        let mut account = AccountDetail {
            // Paper accounts use "accountId" instead of "secAccountId"
            account_id: result
                .get("accountId")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string())
                .or_else(|| Some(paper_account_id.to_string())),

            // Parse netLiquidation from string
            net_liquidation: result
                .get("netLiquidation")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| result.get("netLiquidation").and_then(|v| v.as_f64())),

            // Parse totalProfitLoss
            unrealized_profit_loss: result
                .get("totalProfitLoss")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| result.get("totalProfitLoss").and_then(|v| v.as_f64())),

            // Parse totalProfitLossRate
            unrealized_profit_loss_rate: result
                .get("totalProfitLossRate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| result.get("totalProfitLossRate").and_then(|v| v.as_f64())),

            // Extract currency
            currency: result
                .get("currency")
                .and_then(|v| v.as_str())
                .map(String::from),

            // Initialize other fields as None (will be populated from accountMembers)
            total_cost: None,
            account_type: None,
            broker_account_id: None,
            broker_id: None,
            currency_id: None,
            unrealized_profit_loss_base: None,
            pdt: None,
            professional: None,
            warning: None,
            remind_modify_pwd: None,
            show_upgrade: None,
            open_order_size: None,
            account_members: None,
            open_orders: None,
            open_orders2: None,
            open_ipo_orders: None,
            positions: None,
            positions2: None,
            banners: None,
            total_cash: None,
            total_market_value: None,
            buying_power: None,
            cash_balance: None,
            settled_funds: None,
            unsettled_funds: None,
        };

        // Try to extract account members
        if let Some(members) = result.get("accountMembers") {
            if let Ok(parsed_members) =
                serde_json::from_value::<Vec<AccountMember>>(members.clone())
            {
                account.account_members = Some(parsed_members);
            }
        }

        // Process accountMembers to extract key financial values
        if let Some(ref members) = account.account_members {
            for member in members {
                match member.key.as_str() {
                    "totalMarketValue" => {
                        account.total_market_value = member.value.parse::<f64>().ok();
                    }
                    "usableCash" => {
                        // Paper trading uses "usableCash" instead of "cashBalance"
                        let usable_cash = member.value.parse::<f64>().ok();
                        account.cash_balance = usable_cash;
                        account.buying_power = usable_cash; // Use usable cash as buying power
                    }
                    "dayProfitLoss" => {
                        // We could add a day_profit_loss field if needed
                        // For now, this is informational only
                    }
                    "cashBalance" => {
                        // Fall back to cashBalance if available
                        if account.cash_balance.is_none() {
                            account.cash_balance = member.value.parse::<f64>().ok();
                        }
                    }
                    "dayBuyingPower" | "overnightBuyingPower" => {
                        // Use dayBuyingPower as the primary buying power if not already set
                        if member.key == "dayBuyingPower" && account.buying_power.is_none() {
                            account.buying_power = member.value.parse::<f64>().ok();
                        }
                    }
                    "unsettledFunds" => {
                        account.unsettled_funds = member.value.parse::<f64>().ok();
                    }
                    _ => {}
                }
            }

            // If total_cash wasn't in the members, try to use cash_balance
            if account.total_cash.is_none() && account.cash_balance.is_some() {
                account.total_cash = account.cash_balance;
            }
        }

        // Extract positions array - paper trading returns this directly in the response
        if let Some(positions) = result.get("positions") {
            if let Ok(pos_array) = serde_json::from_value::<Vec<Position>>(positions.clone()) {
                account.positions = Some(pos_array);
            }
        }

        // Extract openOrders array - paper trading returns this directly in the response
        if let Some(open_orders) = result.get("openOrders") {
            if let Ok(orders_array) = serde_json::from_value::<Vec<Order>>(open_orders.clone()) {
                account.open_orders = Some(orders_array);
            }
        }

        // Set account type to CASH (paper accounts are typically cash accounts)
        // The actual account type info is in the "accounts" array if needed
        account.account_type = Some("CASH".to_string());

        Ok(account)
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
        let history = self
            .get_history_orders("All", page_size.unwrap_or(100))
            .await?;

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

        let order_id = order_val
            .get("orderId")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .ok_or(WebullError::ParseError("Missing orderId".to_string()))?;

        let ticker_data = order_val
            .get("ticker")
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

        let quantity = order_val
            .get("totalQuantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let filled_quantity = order_val
            .get("filledQuantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let limit_price = order_val
            .get("lmtPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let stop_price = order_val
            .get("auxPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let avg_fill_price = order_val
            .get("avgFilledPrice")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        // Parse placed time
        let placed_time =
            if let Some(timestamp) = order_val.get("createTime0").and_then(|v| v.as_i64()) {
                DateTime::from_timestamp_millis(timestamp).unwrap_or(Utc::now())
            } else {
                Utc::now()
            };

        let filled_time =
            if let Some(timestamp) = order_val.get("filledTime0").and_then(|v| v.as_i64()) {
                Some(DateTime::from_timestamp_millis(timestamp).unwrap_or(Utc::now()))
            } else {
                None
            };

        let outside_regular_trading_hour = order_val
            .get("outsideRegularTradingHour")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Order {
            order_id,
            combo_id: order_val
                .get("comboId")
                .and_then(|v| v.as_str())
                .map(String::from),
            ticker: Some(ticker),
            action,
            order_type,
            status,
            time_in_force,
            quantity,
            filled_quantity,
            avg_fill_price,
            limit_price,
            stop_price,
            create_time: None,
            placed_time: Some(placed_time.to_rfc3339()),
            filled_time: filled_time.map(|t| t.to_rfc3339()),
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
            self.base_client
                .endpoints
                .paper_orders(paper_account_id, count),
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
        // For paper trading, positions are included in the account details
        // This matches the Python implementation which calls get_account()['positions']
        let account = self.get_account().await?;

        // Return positions from account details, defaulting to empty vec if None
        Ok(account.positions.unwrap_or_default())
    }
}
