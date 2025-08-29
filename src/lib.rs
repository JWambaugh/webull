pub mod builders;
pub mod client;
pub mod endpoints;
pub mod error;
pub mod live_client;
pub mod models;
pub mod paper_client;
pub mod stream;
pub mod unified_client;
pub mod utils;

pub use client::{LiveWebullClient, PaperWebullClient, WebullClient};
pub use error::{Result, WebullError};
pub use models::{
    BarsRequestBuilder, LoginRequestBuilder, NewsRequestBuilder, OptionsRequestBuilder,
    PlaceOrderRequest, PlaceOrderRequestBuilder, ScreenerRequestBuilder,
};
pub use stream::StreamConn;

#[cfg(test)]
mod tests;
