use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};
use serde_json::Value;

// Custom deserializer for fields that can be either string or number
fn deserialize_optional_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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
    pub account_type: i32,  // Changed to i32 based on API
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
    #[serde(rename = "secAccountId", deserialize_with = "deserialize_optional_string_or_number")]
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
    #[serde(rename = "unrealizedProfitLossBase", deserialize_with = "deserialize_f64_from_string_opt", default)]
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
    #[serde(rename = "avgVol3M", deserialize_with = "deserialize_f64_from_string_opt", default)]
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
    #[serde(alias = "totalQuantity", deserialize_with = "deserialize_f64_from_string")]
    pub quantity: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    pub filled_quantity: f64,
    #[serde(alias = "avgFilledPrice", default, deserialize_with = "deserialize_f64_from_string_opt")]
    pub avg_fill_price: Option<f64>,
    #[serde(alias = "lmtPrice", default, deserialize_with = "deserialize_f64_from_string_opt")]
    pub limit_price: Option<f64>,
    #[serde(alias = "stopPrice", alias = "auxPrice", default, deserialize_with = "deserialize_f64_from_string_opt")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub news_time: String,  // API returns as string like "2025-08-27T11:35:08.000+0000"
    #[serde(rename = "newsUrl")]
    pub news_url: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: Option<i32>,
    pub translated: Option<bool>,
    #[serde(rename = "mainPic")]
    pub main_pic: Option<String>,
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

// ============= Helper Functions =============

/// Custom deserializer for f64 from string
pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match s {
        serde_json::Value::String(s) => s.parse::<f64>().map_err(de::Error::custom),
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| de::Error::custom("Invalid number")),
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