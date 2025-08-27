use thiserror::Error;

pub type Result<T> = std::result::Result<T, WebullError>;

#[derive(Error, Debug)]
pub enum WebullError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("MFA required")]
    MfaRequired,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("Session expired")]
    SessionExpired,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Trade token not available")]
    TradeTokenNotAvailable,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Market closed")]
    MarketClosed,

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("MQTT error: {0}")]
    MqttError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Device ID error: {0}")]
    DeviceIdError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}