use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub account: String,
    pub account_type: String,
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
    pub code_account_type: i32,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_expire_time: i64,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub account_id: String,
    pub account_type: String,
    pub broker_id: String,
    pub broker_name: String,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub ticker: Ticker,
    pub position: f64,
    pub cost_price: f64,
    pub cost: f64,
    pub market_value: f64,
    pub last_price: f64,
    pub unrealized_profit_loss: f64,
    pub unrealized_profit_loss_rate: f64,
    pub asset_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    pub ticker_id: i64,
    pub symbol: String,
    pub name: String,
    pub exchange_code: String,
    pub exchange_id: i32,
    pub sec_type: String,
    pub region_id: i32,
    pub currency_id: i32,
    pub currency_code: String,
    pub listing_exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_id: String,
    pub combo_id: Option<String>,
    pub ticker: Ticker,
    pub action: OrderAction,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub avg_fill_price: Option<f64>,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub placed_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "outsideRegularTradingHour")]
    pub outside_regular_trading_hour: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    #[serde(rename = "Working")]
    Working,
    #[serde(rename = "Pending")]
    Pending,
    #[serde(rename = "Filled")]
    Filled,
    #[serde(rename = "Cancelled")]
    Cancelled,
    #[serde(rename = "Failed")]
    Failed,
    #[serde(rename = "Partial Filled")]
    PartialFilled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    Day,
    #[serde(rename = "GTC")]
    GoodTillCancel,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub ticker_id: i64,
    pub symbol: String,
    pub exchange_code: String,
    pub pre_close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub vwap: f64,
    pub change: f64,
    pub change_ratio: f64,
    pub turnover_rate: f64,
    pub vibrate_ratio: f64,
    pub market_value: f64,
    pub status: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub vwap: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level2 {
    pub ask_list: Vec<PriceLevel>,
    pub bid_list: Vec<PriceLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionContract {
    pub derivative_id: i64,
    pub symbol: String,
    pub underlying_symbol: String,
    pub expire_date: String,
    pub strike_price: f64,
    pub direction: OptionDirection,
    pub open_interest: i64,
    pub volume: i64,
    pub ask_list: Vec<PriceLevel>,
    pub bid_list: Vec<PriceLevel>,
    pub latest_price: f64,
    pub change: f64,
    pub change_ratio: f64,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionDirection {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct News {
    pub id: i64,
    pub title: String,
    pub source: String,
    pub summary: String,
    pub news_url: String,
    pub news_time: DateTime<Utc>,
    pub collect_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_id: String,
    pub ticker_id: i64,
    pub symbol: String,
    pub alert_type: AlertType,
    pub trigger_value: f64,
    pub frequency: AlertFrequency,
    pub status: AlertStatus,
    pub created_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    #[serde(rename = "PRICE_UP")]
    PriceUp,
    #[serde(rename = "PRICE_DOWN")]
    PriceDown,
    #[serde(rename = "VOLUME")]
    Volume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertFrequency {
    #[serde(rename = "ONCE")]
    Once,
    #[serde(rename = "DAILY")]
    Daily,
    #[serde(rename = "ALWAYS")]
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "TRIGGERED")]
    Triggered,
    #[serde(rename = "EXPIRED")]
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fundamental {
    pub ticker_id: i64,
    pub pe: Option<f64>,
    pub forward_pe: Option<f64>,
    pub pb: Option<f64>,
    pub ps: Option<f64>,
    pub peg: Option<f64>,
    pub eps: Option<f64>,
    pub bvps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub total_shares: Option<i64>,
    pub float_shares: Option<i64>,
    pub market_cap: Option<f64>,
    pub fifty_two_wk_high: Option<f64>,
    pub fifty_two_wk_low: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetail {
    pub account_id: String,
    pub net_liquidation: f64,
    pub total_cash: f64,
    pub total_market_value: f64,
    pub total_profit_loss: f64,
    pub total_profit_loss_rate: f64,
    pub day_profit_loss: f64,
    pub day_profit_loss_rate: f64,
    pub buying_power: f64,
    pub cash_balance: f64,
    pub margin: f64,
    pub unsettled_cash: f64,
    pub unsettled_funds: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub id: String,
    pub account_id: String,
    pub direction: TransferDirection,
    pub amount: f64,
    pub status: TransferStatus,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferDirection {
    #[serde(rename = "IN")]
    In,
    #[serde(rename = "OUT")]
    Out,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerRequest {
    pub region_id: i32,
    pub plate_list: Vec<String>,
    pub sort_field: String,
    pub sort_type: String,
    pub page_index: i32,
    pub page_size: i32,
    pub filters: HashMap<String, ScreenerFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenerFilter {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub topic: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}