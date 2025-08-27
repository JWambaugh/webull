use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use serde_json::Value;
use tokio::time::{sleep, Duration};
use log::{debug, error, info, warn};
use crate::error::{Result, WebullError};

/// Callback for handling price updates
pub type PriceCallback = Arc<dyn Fn(Value, Value) + Send + Sync>;

/// Callback for handling order updates  
pub type OrderCallback = Arc<dyn Fn(Value, Value) + Send + Sync>;

/// Stream connection configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub host: String,
    pub port: u16,
    pub use_ssl: bool,
    pub client_id: String,
    pub keep_alive: Duration,
    pub debug: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            host: "wss://wspush.webullfintech.com:443/mqtt".to_string(),
            port: 443,
            use_ssl: true,
            client_id: format!("rust_client_{}", uuid::Uuid::new_v4()),
            keep_alive: Duration::from_secs(30),
            debug: false,
        }
    }
}

/// WebSocket/MQTT streaming connection
pub struct StreamConn {
    config: StreamConfig,
    client: Option<AsyncClient>,
    price_callback: Option<PriceCallback>,
    order_callback: Option<OrderCallback>,
    total_volume: Arc<RwLock<HashMap<String, i64>>>,
    subscriptions: Arc<RwLock<Vec<String>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl StreamConn {
    /// Create a new streaming connection
    pub fn new(config: Option<StreamConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            client: None,
            price_callback: None,
            order_callback: None,
            total_volume: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Set price update callback
    pub fn set_price_callback<F>(&mut self, callback: F)
    where
        F: Fn(Value, Value) + Send + Sync + 'static,
    {
        self.price_callback = Some(Arc::new(callback));
    }

    /// Set order update callback
    pub fn set_order_callback<F>(&mut self, callback: F)
    where
        F: Fn(Value, Value) + Send + Sync + 'static,
    {
        self.order_callback = Some(Arc::new(callback));
    }

    /// Connect to the streaming service
    pub async fn connect(&mut self, access_token: &str, did: &str) -> Result<()> {
        let mut mqtt_options = MqttOptions::new(
            &self.config.client_id,
            &self.config.host,
            self.config.port,
        );

        mqtt_options.set_keep_alive(self.config.keep_alive);
        
        // Set authentication
        mqtt_options.set_credentials(access_token, did);

        // Create MQTT client
        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
        self.client = Some(client.clone());

        // Spawn event loop handler
        let is_connected = Arc::clone(&self.is_connected);
        let price_callback = self.price_callback.clone();
        let order_callback = self.order_callback.clone();
        let debug = self.config.debug;
        let total_volume = Arc::clone(&self.total_volume);

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        if debug {
                            debug!("MQTT Event: {:?}", event);
                        }

                        match event {
                            Event::Incoming(Packet::ConnAck(_)) => {
                                info!("Connected to streaming service");
                                *is_connected.write() = true;
                            }
                            Event::Incoming(Packet::Publish(publish)) => {
                                Self::handle_message(
                                    &publish.topic,
                                    &publish.payload,
                                    &price_callback,
                                    &order_callback,
                                    &total_volume,
                                    debug,
                                );
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                warn!("Disconnected from streaming service");
                                *is_connected.write() = false;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("MQTT Error: {:?}", e);
                        *is_connected.write() = false;
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        // Wait for connection
        let mut attempts = 0;
        while !*self.is_connected.read() && attempts < 10 {
            sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }

        if *self.is_connected.read() {
            Ok(())
        } else {
            Err(WebullError::WebSocketError("Failed to connect to streaming service".to_string()))
        }
    }

    /// Handle incoming messages
    fn handle_message(
        topic: &str,
        payload: &[u8],
        price_callback: &Option<PriceCallback>,
        order_callback: &Option<OrderCallback>,
        total_volume: &Arc<RwLock<HashMap<String, i64>>>,
        debug: bool,
    ) {
        // Try to parse the message
        let topic_json = match serde_json::from_str::<Value>(topic) {
            Ok(v) => v,
            Err(e) => {
                if debug {
                    debug!("Failed to parse topic: {}, error: {}", topic, e);
                }
                return;
            }
        };

        let payload_json = match serde_json::from_slice::<Value>(payload) {
            Ok(v) => v,
            Err(e) => {
                if debug {
                    debug!("Failed to parse payload: {:?}, error: {}", payload, e);
                }
                return;
            }
        };

        if debug {
            debug!("Topic: {}, Payload: {}", topic_json, payload_json);
        }

        // Check if it's an order message (from platpush)
        if topic.contains("platpush") {
            if let Some(callback) = order_callback {
                callback(topic_json, payload_json);
            }
        } 
        // Check if it's a price message (from wspush)
        else if topic.contains("wspush") || topic.contains("ticker") {
            // Update total volume if applicable
            if let Some(ticker_id) = topic_json.get("tickerId").and_then(|v| v.as_str()) {
                if let Some(volume) = payload_json.get("volume").and_then(|v| v.as_i64()) {
                    total_volume.write().insert(ticker_id.to_string(), volume);
                }
            }

            if let Some(callback) = price_callback {
                callback(topic_json, payload_json);
            }
        }
    }

    /// Subscribe to ticker updates
    pub async fn subscribe_ticker(&mut self, ticker_id: &str, topics: Vec<i32>) -> Result<()> {
        if let Some(client) = &self.client {
            for topic_type in topics {
                let topic = format!("{{\"tickerId\":\"{}\",\"type\":{}}}", ticker_id, topic_type);
                
                client.subscribe(&topic, QoS::AtLeastOnce).await
                    .map_err(|e| WebullError::MqttError(e.to_string()))?;
                
                self.subscriptions.write().push(topic.clone());
                
                if self.config.debug {
                    debug!("Subscribed to: {}", topic);
                }
            }
            Ok(())
        } else {
            Err(WebullError::WebSocketError("Not connected".to_string()))
        }
    }

    /// Subscribe to order updates
    pub async fn subscribe_orders(&mut self, account_id: &str) -> Result<()> {
        if let Some(client) = &self.client {
            let topic = format!("{{\"secAccountId\":\"{}\"}}", account_id);
            
            client.subscribe(&topic, QoS::AtLeastOnce).await
                .map_err(|e| WebullError::MqttError(e.to_string()))?;
            
            self.subscriptions.write().push(topic.clone());
            
            if self.config.debug {
                debug!("Subscribed to orders: {}", topic);
            }
            Ok(())
        } else {
            Err(WebullError::WebSocketError("Not connected".to_string()))
        }
    }

    /// Unsubscribe from ticker updates
    pub async fn unsubscribe_ticker(&mut self, ticker_id: &str, topics: Vec<i32>) -> Result<()> {
        if let Some(client) = &self.client {
            for topic_type in topics {
                let topic = format!("{{\"tickerId\":\"{}\",\"type\":{}}}", ticker_id, topic_type);
                
                client.unsubscribe(&topic).await
                    .map_err(|e| WebullError::MqttError(e.to_string()))?;
                
                self.subscriptions.write().retain(|t| t != &topic);
                
                if self.config.debug {
                    debug!("Unsubscribed from: {}", topic);
                }
            }
            Ok(())
        } else {
            Err(WebullError::WebSocketError("Not connected".to_string()))
        }
    }

    /// Unsubscribe from all topics
    pub async fn unsubscribe_all(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            let subscriptions = self.subscriptions.read().clone();
            for topic in subscriptions {
                client.unsubscribe(&topic).await
                    .map_err(|e| WebullError::MqttError(e.to_string()))?;
            }
            self.subscriptions.write().clear();
            Ok(())
        } else {
            Err(WebullError::WebSocketError("Not connected".to_string()))
        }
    }

    /// Disconnect from the streaming service
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.client.is_some() {
            self.unsubscribe_all().await?;
            if let Some(client) = self.client.take() {
                client.disconnect().await
                    .map_err(|e| WebullError::MqttError(e.to_string()))?;
            }
            *self.is_connected.write() = false;
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.is_connected.read()
    }

    /// Get current subscriptions
    pub fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().clone()
    }

    /// Get total volume for a ticker
    pub fn get_total_volume(&self, ticker_id: &str) -> Option<i64> {
        self.total_volume.read().get(ticker_id).copied()
    }
}

/// Topic types for streaming subscriptions
pub struct TopicTypes;

impl TopicTypes {
    pub const TICKER_STATUS: i32 = 101;
    pub const TICKER_QUOTE: i32 = 102;
    pub const TICKER_TRADE: i32 = 103;
    pub const TICKER_BOOK: i32 = 104;
    pub const TICKER_QUOTE_AND_TRADE: i32 = 105;
    pub const TICKER_QUOTE_TRADE_OPTIONAL: i32 = 106;
    pub const TICKER_TRADE_AND_BOOK: i32 = 107;
    pub const TICKER_FULL: i32 = 108;
    
    /// Get all available topic types
    pub fn all() -> Vec<i32> {
        vec![
            Self::TICKER_STATUS,
            Self::TICKER_QUOTE,
            Self::TICKER_TRADE,
            Self::TICKER_BOOK,
            Self::TICKER_QUOTE_AND_TRADE,
            Self::TICKER_QUOTE_TRADE_OPTIONAL,
            Self::TICKER_TRADE_AND_BOOK,
            Self::TICKER_FULL,
        ]
    }
    
    /// Get basic subscription topics
    pub fn basic() -> Vec<i32> {
        vec![
            Self::TICKER_QUOTE,
            Self::TICKER_TRADE,
            Self::TICKER_BOOK,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert!(config.use_ssl);
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_topic_types() {
        let all_topics = TopicTypes::all();
        assert_eq!(all_topics.len(), 8);
        
        let basic_topics = TopicTypes::basic();
        assert_eq!(basic_topics.len(), 3);
    }
}