pub mod client;
pub mod live_client;
pub mod paper_client;
pub mod unified_client;
pub mod endpoints;
pub mod stream;
pub mod error;
pub mod models;
pub mod utils;

pub use client::{WebullClient, LiveWebullClient, PaperWebullClient};
pub use stream::StreamConn;
pub use error::{WebullError, Result};

#[cfg(test)]
mod tests;