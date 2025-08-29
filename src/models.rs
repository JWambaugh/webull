use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Custom deserializer for fields that can be either string or number
fn deserialize_optional_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_value: Option<Value> = Option::deserialize(deserializer)?;
    match opt_value {
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(_) => Err(de::Error::custom("expected string or number")),
        None => Ok(None),
    }
}

// ============= Login Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub account: String,
    pub account_type: i32, // Changed to i32 based on API
    pub device_id: String,
    pub device_name: String,
    pub grade: i32,
    pub pwd: String,
    pub region_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext_info: Option<ExtInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_questions: Option<String>,
}

/// Builder for LoginRequest
#[derive(Debug, Clone)]
pub struct LoginRequestBuilder {
    username: Option<String>,
    password: Option<String>,
    device_name: Option<String>,
    device_id: Option<String>,
    mfa_code: Option<String>,
    question_id: Option<String>,
    question_answer: Option<String>,
    region_id: Option<i32>,
}

impl LoginRequestBuilder {
    /// Create a new login request builder
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
            device_name: None,
            device_id: None,
            mfa_code: None,
            question_id: None,
            question_answer: None,
            region_id: Some(6), // Default to US region
        }
    }

    /// Set the username (email)
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set the password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the device name
    pub fn device_name(mut self, device_name: impl Into<String>) -> Self {
        self.device_name = Some(device_name.into());
        self
    }

    /// Set the device ID
    pub fn device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    /// Set the MFA code
    pub fn mfa(mut self, code: impl Into<String>) -> Self {
        self.mfa_code = Some(code.into());
        self
    }

    /// Set security question answer
    pub fn security_question(
        mut self,
        question_id: impl Into<String>,
        answer: impl Into<String>,
    ) -> Self {
        self.question_id = Some(question_id.into());
        self.question_answer = Some(answer.into());
        self
    }

    /// Set the region (default is 6 for US)
    pub fn region(mut self, region_id: i32) -> Self {
        self.region_id = Some(region_id);
        self
    }

    /// Build the login parameters
    /// Returns tuple of (username, password, device_name, mfa, question_id, question_answer)
    pub fn build(
        self,
    ) -> Result<
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
        String,
    > {
        let username = self
            .username
            .ok_or_else(|| "username is required".to_string())?;
        let password = self
            .password
            .ok_or_else(|| "password is required".to_string())?;

        Ok((
            username,
            password,
            self.device_name,
            self.mfa_code,
            self.question_id,
            self.question_answer,
        ))
    }
}

impl Default for LoginRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtInfo {
    #[serde(default)]
    pub code_account_type: i32,
    #[serde(default)]
    pub verification_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_pwd_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(rename = "tokenExpireTime")]
    pub token_expire_time: String,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_mode_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_time_of_third: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_security_pwd: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_address: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<UserSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_account_rels: Option<Vec<UserAccountRel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub chart_option: Option<i32>,
    pub create_time: Option<String>,
    pub focus_market_id: Option<String>,
    pub font_size: Option<String>,
    pub hot_news: Option<i32>,
    pub id: Option<i64>,
    pub incre_decre_color: Option<i32>,
    pub kdata: Option<i32>,
    pub language: Option<String>,
    pub language_update_time: Option<String>,
    pub list_style: Option<i32>,
    pub operate_time: Option<String>,
    pub order_deal_remind: Option<i32>,
    pub portfolio_bulletin: Option<i32>,
    pub portfolio_display_mode: Option<i32>,
    pub portfolio_holdings_display: Option<i32>,
    pub portfolio_index_display: Option<i32>,
    pub portfolio_name_newline: Option<i32>,
    pub refresh_frequency: Option<i32>,
    pub region_id: Option<i32>,
    pub shock: Option<i32>,
    pub theme: Option<i32>,
    pub ticker_price_remind: Option<i32>,
    pub update_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountRel {
    pub broker_id: i32,
}

// ============= Account Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(rename = "secAccountId")]
    pub account_id: String,
    pub account_type: String,
    pub broker_id: String,
    pub broker_name: String,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetail {
    #[serde(
        rename = "secAccountId",
        deserialize_with = "deserialize_optional_string_or_number"
    )]
    pub account_id: Option<String>,
    pub account_type: Option<String>,
    pub broker_account_id: Option<String>,
    pub broker_id: Option<i32>,
    pub currency: Option<String>,
    pub currency_id: Option<i32>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub net_liquidation: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub total_cost: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub unrealized_profit_loss: Option<f64>,
    #[serde(
        rename = "unrealizedProfitLossBase",
        deserialize_with = "deserialize_f64_from_string_opt",
        default
    )]
    pub unrealized_profit_loss_base: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub unrealized_profit_loss_rate: Option<f64>,
    pub pdt: Option<bool>,
    pub professional: Option<bool>,
    pub warning: Option<bool>,
    pub remind_modify_pwd: Option<bool>,
    pub show_upgrade: Option<bool>,
    pub open_order_size: Option<i32>,

    // These come from accountMembers array
    pub account_members: Option<Vec<AccountMember>>,

    // Computed fields from accountMembers
    #[serde(skip)]
    pub total_market_value: Option<f64>,
    #[serde(skip)]
    pub cash_balance: Option<f64>,
    #[serde(skip)]
    pub total_cash: Option<f64>,
    #[serde(skip)]
    pub buying_power: Option<f64>,
    #[serde(skip)]
    pub settled_funds: Option<f64>,
    #[serde(skip)]
    pub unsettled_funds: Option<f64>,

    pub positions: Option<Vec<Position>>,
    pub positions2: Option<Vec<Position>>,
    pub open_orders: Option<Vec<Order>>,
    pub open_orders2: Option<Vec<Order>>,
    pub open_ipo_orders: Option<Vec<serde_json::Value>>,
    pub banners: Option<Vec<Banner>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMember {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub img_url: Option<String>,
    pub link: Option<String>,
    pub sub_title: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub banner_type: Option<String>,
}

// ============= Paper Account Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperAccount {
    pub id: i64,
    pub paper_id: i32,
    pub paper_name: String,
    pub paper_type: i32,
    pub currency: String,
    pub currency_id: i32,
    pub status: i32,
    pub paper_ticker_pool_code: Option<String>,
    pub support_account_type: Option<String>,
    pub support_outside_rth: Option<bool>,
    pub order_types: Option<Vec<String>>,
    pub time_in_forces: Option<Vec<String>>,
}

// ============= Position Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub ticker: Option<Ticker>,
    #[serde(alias = "position", deserialize_with = "deserialize_f64_from_string")]
    pub quantity: f64,
    #[serde(alias = "costPrice", deserialize_with = "deserialize_f64_from_string")]
    pub avg_cost: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub cost: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub market_value: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub last_price: f64,
    #[serde(default, deserialize_with = "deserialize_f64_from_string_opt")]
    pub unrealized_profit_loss: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_f64_from_string_opt")]
    pub unrealized_profit_loss_rate: Option<f64>,
    pub asset_type: Option<String>,
}

// ============= Ticker Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    pub ticker_id: i64,
    #[serde(rename = "disSymbol")]
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub tiny_name: Option<String>,
    #[serde(default)]
    pub exchange_code: String,
    #[serde(rename = "disExchangeCode", default)]
    pub dis_exchange_code: Option<String>,
    #[serde(default)]
    pub exchange_id: i32,
    #[serde(default)]
    pub sec_type: Vec<i32>,
    #[serde(default)]
    pub sec_type2: Option<i32>,
    #[serde(default)]
    pub security_type: Option<i32>,
    #[serde(default)]
    pub security_sub_type: Option<i32>,
    #[serde(default)]
    pub region_id: i32,
    #[serde(default)]
    pub region_code: Option<String>,
    #[serde(default)]
    pub currency_id: i32,
    #[serde(default)]
    pub currency_code: String,
    #[serde(rename = "type", default)]
    pub ticker_type: Option<i32>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub list_date: Option<String>,
    #[serde(default)]
    pub list_status: Option<i32>,
    #[serde(default)]
    pub bond_support: Option<i32>,
    #[serde(default)]
    pub derivative_support: Option<i32>,
    #[serde(default)]
    pub is_adr: Option<i32>,
    #[serde(rename = "isPTP", default)]
    pub is_ptp: Option<i32>,
    #[serde(default)]
    pub issuer_region_id: Option<i32>,
    #[serde(default)]
    pub night_trade_session: Option<i32>,
    #[serde(default)]
    pub odd_lot_support: Option<bool>,
    #[serde(default)]
    pub overnight_trade_flag: Option<i32>,
    #[serde(default)]
    pub shariah_flag: Option<i32>,
}

// ============= Quote Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub close: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub change: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub change_ratio: f64,
    #[serde(rename = "preClose", deserialize_with = "deserialize_f64_from_string")]
    pub pre_close: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub open: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub high: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub low: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub volume: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub avg_vol10_d: Option<f64>,
    #[serde(
        rename = "avgVol3M",
        deserialize_with = "deserialize_f64_from_string_opt",
        default
    )]
    pub avg_vol3_m: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub market_value: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub pe: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub forward_pe: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub bps: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub ask: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub bid: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub ask_size: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_string_opt", default)]
    pub bid_size: Option<f64>,
    pub currency_code: Option<String>,
    pub currency_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<Depth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    pub ntv_agg_ask_list: Option<Vec<PriceLevel>>,
    pub ntv_agg_bid_list: Option<Vec<PriceLevel>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLevel {
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub volume: f64,
}

// ============= Order Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_id: String,
    pub combo_id: Option<String>,
    pub ticker: Option<Ticker>,
    pub action: OrderAction,
    pub order_type: OrderType,
    #[serde(rename = "status", alias = "statusCode")]
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(
        alias = "totalQuantity",
        deserialize_with = "deserialize_f64_from_string"
    )]
    pub quantity: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub filled_quantity: f64,
    #[serde(
        alias = "avgFilledPrice",
        default,
        deserialize_with = "deserialize_f64_from_string_opt"
    )]
    pub avg_fill_price: Option<f64>,
    #[serde(
        alias = "lmtPrice",
        default,
        deserialize_with = "deserialize_f64_from_string_opt"
    )]
    pub limit_price: Option<f64>,
    #[serde(
        alias = "stopPrice",
        alias = "auxPrice",
        default,
        deserialize_with = "deserialize_f64_from_string_opt"
    )]
    pub stop_price: Option<f64>,
    #[serde(rename = "outsideRegularTradingHour")]
    pub outside_regular_trading_hour: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    #[serde(rename = "MKT")]
    Market,
    #[serde(rename = "LMT")]
    Limit,
    #[serde(rename = "STP")]
    Stop,
    #[serde(rename = "STP_LMT")]
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    #[serde(rename = "Working")]
    Working,
    #[serde(rename = "Pending")]
    Pending,
    #[serde(rename = "Submitted")]
    Submitted,
    #[serde(rename = "PartialFilled")]
    PartialFilled,
    #[serde(rename = "Filled")]
    Filled,
    #[serde(rename = "Cancelled")]
    Cancelled,
    #[serde(rename = "Failed")]
    Failed,
    #[serde(rename = "Rejected")]
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    #[serde(rename = "DAY")]
    Day,
    #[serde(rename = "GTC")]
    GoodTillCancel,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
}

// ============= Place Order Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    pub ticker_id: i64,
    pub action: OrderAction,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    pub outside_regular_trading_hour: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combo_type: Option<String>,
}

impl PlaceOrderRequest {
    /// Create a builder for a market order
    pub fn market() -> PlaceOrderRequestBuilder {
        PlaceOrderRequestBuilder::new(OrderType::Market)
    }

    /// Create a builder for a limit order
    pub fn limit(price: f64) -> PlaceOrderRequestBuilder {
        PlaceOrderRequestBuilder::new(OrderType::Limit).limit_price(price)
    }

    /// Create a builder for a stop order
    pub fn stop(price: f64) -> PlaceOrderRequestBuilder {
        PlaceOrderRequestBuilder::new(OrderType::Stop).stop_price(price)
    }

    /// Create a builder for a stop-limit order
    pub fn stop_limit(stop_price: f64, limit_price: f64) -> PlaceOrderRequestBuilder {
        PlaceOrderRequestBuilder::new(OrderType::StopLimit)
            .stop_price(stop_price)
            .limit_price(limit_price)
    }

    /// Create a custom builder with a specific order type
    pub fn builder(order_type: OrderType) -> PlaceOrderRequestBuilder {
        PlaceOrderRequestBuilder::new(order_type)
    }
}

/// Builder for PlaceOrderRequest
#[derive(Debug, Clone)]
pub struct PlaceOrderRequestBuilder {
    ticker_id: Option<i64>,
    action: Option<OrderAction>,
    order_type: OrderType,
    time_in_force: TimeInForce,
    quantity: Option<f64>,
    limit_price: Option<f64>,
    stop_price: Option<f64>,
    outside_regular_trading_hour: bool,
    serial_id: Option<String>,
    combo_type: Option<String>,
}

impl PlaceOrderRequestBuilder {
    /// Create a new builder with the given order type
    pub fn new(order_type: OrderType) -> Self {
        Self {
            ticker_id: None,
            action: None,
            order_type,
            time_in_force: TimeInForce::Day, // Default to Day
            quantity: None,
            limit_price: None,
            stop_price: None,
            outside_regular_trading_hour: false,
            serial_id: None,
            combo_type: None,
        }
    }

    /// Set the ticker ID
    pub fn ticker_id(mut self, ticker_id: i64) -> Self {
        self.ticker_id = Some(ticker_id);
        self
    }

    /// Set the ticker by symbol (requires looking up the ticker_id separately)
    /// Note: This is a convenience method for documentation, actual lookup must be done separately
    pub fn symbol(self, _symbol: &str) -> Self {
        // Note: The actual ticker_id must be set using ticker_id() method
        // This is here for API consistency
        self
    }

    /// Set the order action (Buy or Sell)
    pub fn action(mut self, action: OrderAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Convenience method for buy orders
    pub fn buy(mut self) -> Self {
        self.action = Some(OrderAction::Buy);
        self
    }

    /// Convenience method for sell orders
    pub fn sell(mut self) -> Self {
        self.action = Some(OrderAction::Sell);
        self
    }

    /// Set the quantity
    pub fn quantity(mut self, quantity: f64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Set the time in force
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set the limit price (for limit and stop-limit orders)
    pub fn limit_price(mut self, price: f64) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set the stop price (for stop and stop-limit orders)
    pub fn stop_price(mut self, price: f64) -> Self {
        self.stop_price = Some(price);
        self
    }

    /// Enable or disable outside regular trading hours
    pub fn outside_regular_trading_hour(mut self, enabled: bool) -> Self {
        self.outside_regular_trading_hour = enabled;
        self
    }

    /// Enable outside regular trading hours (convenience method)
    pub fn extended_hours(mut self) -> Self {
        self.outside_regular_trading_hour = true;
        self
    }

    /// Set the serial ID
    pub fn serial_id(mut self, id: String) -> Self {
        self.serial_id = Some(id);
        self
    }

    /// Set the combo type
    pub fn combo_type(mut self, combo_type: String) -> Self {
        self.combo_type = Some(combo_type);
        self
    }

    /// Build the PlaceOrderRequest
    /// Returns an error if required fields are missing
    pub fn build(self) -> Result<PlaceOrderRequest, String> {
        let ticker_id = self
            .ticker_id
            .ok_or_else(|| "ticker_id is required".to_string())?;
        let action = self
            .action
            .ok_or_else(|| "action is required".to_string())?;
        let quantity = self
            .quantity
            .ok_or_else(|| "quantity is required".to_string())?;

        // Validate order type specific requirements
        match self.order_type {
            OrderType::Limit => {
                if self.limit_price.is_none() {
                    return Err(format!("{:?} order requires limit_price", self.order_type));
                }
            }
            OrderType::Stop => {
                if self.stop_price.is_none() {
                    return Err("Stop order requires stop_price".to_string());
                }
            }
            OrderType::StopLimit => {
                if self.limit_price.is_none() {
                    return Err("StopLimit order requires limit_price".to_string());
                }
                if self.stop_price.is_none() {
                    return Err("StopLimit order requires stop_price".to_string());
                }
            }
            _ => {}
        }

        Ok(PlaceOrderRequest {
            ticker_id,
            action,
            order_type: self.order_type,
            time_in_force: self.time_in_force,
            quantity,
            limit_price: self.limit_price,
            stop_price: self.stop_price,
            outside_regular_trading_hour: self.outside_regular_trading_hour,
            serial_id: self.serial_id,
            combo_type: self.combo_type,
        })
    }
}

// ============= Bar/Candle Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub vwap: f64,
}

/// Request builder for fetching bars/candles
#[derive(Debug, Clone)]
pub struct BarsRequestBuilder {
    ticker_id: Option<String>,
    interval: Option<String>,
    count: Option<i32>,
    timestamp: Option<i64>,
}

impl BarsRequestBuilder {
    /// Create a new bars request builder
    pub fn new() -> Self {
        Self {
            ticker_id: None,
            interval: None,
            count: Some(100), // Default count
            timestamp: None,
        }
    }

    /// Set the ticker ID
    pub fn ticker_id(mut self, ticker_id: impl Into<String>) -> Self {
        self.ticker_id = Some(ticker_id.into());
        self
    }

    /// Set the interval (e.g., "1m", "5m", "1d")
    pub fn interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = Some(interval.into());
        self
    }

    /// Set the number of bars to fetch
    pub fn count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }

    /// Set the timestamp to fetch bars from
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the date to fetch bars from
    pub fn from_date(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(date.timestamp());
        self
    }

    /// Build the request parameters
    pub fn build(self) -> Result<(String, String, i32, Option<i64>), String> {
        let ticker_id = self
            .ticker_id
            .ok_or_else(|| "ticker_id is required".to_string())?;
        let interval = self
            .interval
            .ok_or_else(|| "interval is required".to_string())?;
        let count = self.count.unwrap_or(100);

        Ok((ticker_id, interval, count, self.timestamp))
    }
}

impl Default for BarsRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============= News Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct News {
    pub id: i64,
    pub title: String,
    pub summary: Option<String>,
    #[serde(rename = "sourceName")]
    pub source_name: Option<String>,
    #[serde(rename = "collectSource")]
    pub collect_source: Option<String>,
    #[serde(rename = "newsTime")]
    pub news_time: String, // API returns as string like "2025-08-27T11:35:08.000+0000"
    #[serde(rename = "newsUrl")]
    pub news_url: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: Option<i32>,
    pub translated: Option<bool>,
    #[serde(rename = "mainPic")]
    pub main_pic: Option<String>,
}

/// Request builder for fetching news
#[derive(Debug, Clone)]
pub struct NewsRequestBuilder {
    ticker: Option<String>,
    last_id: Option<i64>,
    count: Option<i32>,
}

impl NewsRequestBuilder {
    /// Create a new news request builder
    pub fn new() -> Self {
        Self {
            ticker: None,
            last_id: Some(0), // Default to fetch from beginning
            count: Some(20),  // Default count
        }
    }

    /// Set the ticker/symbol
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Set the last news ID for pagination
    pub fn last_id(mut self, last_id: i64) -> Self {
        self.last_id = Some(last_id);
        self
    }

    /// Fetch news after a specific news ID
    pub fn after(mut self, news_id: i64) -> Self {
        self.last_id = Some(news_id);
        self
    }

    /// Set the number of news items to fetch
    pub fn count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }

    /// Set to fetch the latest N news items
    pub fn latest(mut self, count: i32) -> Self {
        self.last_id = Some(0);
        self.count = Some(count);
        self
    }

    /// Build the request parameters
    pub fn build(self) -> Result<(String, i64, i32), String> {
        let ticker = self
            .ticker
            .ok_or_else(|| "ticker is required".to_string())?;
        let last_id = self.last_id.unwrap_or(0);
        let count = self.count.unwrap_or(20);

        Ok((ticker, last_id, count))
    }
}

impl Default for NewsRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============= Options Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionContract {
    pub ticker_id: i64,
    pub symbol: String,
    pub strike_price: f64,
    pub expiration_date: String,
    pub option_type: String, // CALL or PUT
}

/// Builder for requesting options data
#[derive(Debug, Clone)]
pub struct OptionsRequestBuilder {
    ticker: Option<String>,
    expiration_date: Option<String>,
    option_type: Option<String>,
    min_strike: Option<f64>,
    max_strike: Option<f64>,
}

impl OptionsRequestBuilder {
    /// Create a new options request builder
    pub fn new() -> Self {
        Self {
            ticker: None,
            expiration_date: None,
            option_type: None,
            min_strike: None,
            max_strike: None,
        }
    }

    /// Set the ticker symbol
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Set the expiration date (format: YYYY-MM-DD)
    pub fn expiration(mut self, date: impl Into<String>) -> Self {
        self.expiration_date = Some(date.into());
        self
    }

    /// Request only call options
    pub fn calls_only(mut self) -> Self {
        self.option_type = Some("CALL".to_string());
        self
    }

    /// Request only put options
    pub fn puts_only(mut self) -> Self {
        self.option_type = Some("PUT".to_string());
        self
    }

    /// Set minimum strike price filter
    pub fn min_strike(mut self, price: f64) -> Self {
        self.min_strike = Some(price);
        self
    }

    /// Set maximum strike price filter
    pub fn max_strike(mut self, price: f64) -> Self {
        self.max_strike = Some(price);
        self
    }

    /// Set strike price range
    pub fn strike_range(mut self, min: f64, max: f64) -> Self {
        self.min_strike = Some(min);
        self.max_strike = Some(max);
        self
    }

    /// Request options near the money (within a percentage of current price)
    pub fn near_the_money(mut self, current_price: f64, percent_range: f64) -> Self {
        let range = current_price * (percent_range / 100.0);
        self.min_strike = Some(current_price - range);
        self.max_strike = Some(current_price + range);
        self
    }

    /// Build the options request parameters
    pub fn build(self) -> Result<String, String> {
        let ticker = self
            .ticker
            .ok_or_else(|| "ticker is required".to_string())?;

        // In a real implementation, this would build appropriate query parameters
        // For now, we just return the ticker as the main required parameter
        Ok(ticker)
    }
}

impl Default for OptionsRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============= Fundamentals Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fundamental {
    pub ticker_id: Option<i64>,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
}

// ============= Screener Models =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerRequest {
    pub region_id: i32,
    pub plate_id: i32,
    pub rank_id: i32,
}

/// Builder for ScreenerRequest
#[derive(Debug, Clone)]
pub struct ScreenerRequestBuilder {
    region_id: Option<i32>,
    plate_id: Option<i32>,
    rank_id: Option<i32>,
}

impl ScreenerRequestBuilder {
    /// Create a new screener request builder
    pub fn new() -> Self {
        Self {
            region_id: Some(6), // Default to US region
            plate_id: None,
            rank_id: None,
        }
    }

    /// Set the region ID (default is 6 for US)
    pub fn region(mut self, region_id: i32) -> Self {
        self.region_id = Some(region_id);
        self
    }

    /// Set the plate/category ID
    pub fn plate(mut self, plate_id: i32) -> Self {
        self.plate_id = Some(plate_id);
        self
    }

    /// Set the rank/sort ID
    pub fn rank(mut self, rank_id: i32) -> Self {
        self.rank_id = Some(rank_id);
        self
    }

    /// Use preset for top gainers
    pub fn top_gainers(mut self) -> Self {
        self.plate_id = Some(1); // Typical ID for gainers
        self.rank_id = Some(1); // Sort by gain percentage
        self
    }

    /// Use preset for top losers
    pub fn top_losers(mut self) -> Self {
        self.plate_id = Some(2); // Typical ID for losers
        self.rank_id = Some(2); // Sort by loss percentage
        self
    }

    /// Use preset for most active (by volume)
    pub fn most_active(mut self) -> Self {
        self.plate_id = Some(3); // Typical ID for active stocks
        self.rank_id = Some(3); // Sort by volume
        self
    }

    /// Build the screener request
    pub fn build(self) -> Result<ScreenerRequest, String> {
        let region_id = self.region_id.unwrap_or(6);
        let plate_id = self
            .plate_id
            .ok_or_else(|| "plate_id is required".to_string())?;
        let rank_id = self
            .rank_id
            .ok_or_else(|| "rank_id is required".to_string())?;

        Ok(ScreenerRequest {
            region_id,
            plate_id,
            rank_id,
        })
    }
}

impl Default for ScreenerRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============= Helper Functions =============

/// Custom deserializer for f64 from string
pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match s {
        serde_json::Value::String(s) => s.parse::<f64>().map_err(de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| de::Error::custom("Invalid number")),
        _ => Err(de::Error::custom("Expected string or number")),
    }
}

/// Custom deserializer for optional f64 from string
pub fn deserialize_f64_from_string_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match s {
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            s.parse::<f64>().map(Some).map_err(de::Error::custom)
        }
        Some(serde_json::Value::Number(n)) => Ok(n.as_f64()),
        _ => Ok(None),
    }
}
