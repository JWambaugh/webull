use chrono::Local;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use webull::endpoints::Endpoints;
use webull::utils::{hash_password, get_account_type, generate_req_id};

// Basic structures we need for raw testing
#[derive(Debug)]
struct RawApiTester {
    client: Client,
    headers: HeaderMap,
    endpoints: Endpoints,
    did: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    trade_token: Option<String>,
    account_id: Option<String>,
    paper_account_id: Option<String>,
    uuid: Option<String>,
}

impl RawApiTester {
    fn new(did: String) -> Self {
        let mut headers = HeaderMap::new();
        
        // Match the exact headers from the working client
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
        
        Self {
            client: Client::new(),
            headers,
            endpoints: Endpoints::new(),
            did,
            access_token: None,
            refresh_token: None,
            trade_token: None,
            account_id: None,
            paper_account_id: None,
            uuid: None,
        }
    }
    
    fn build_headers(&self, include_trade_token: bool, include_time: bool, include_zone: bool) -> HeaderMap {
        let mut headers = self.headers.clone();
        
        // Generate reqid like the working client
        let req_id = generate_req_id();
        headers.insert("reqid", HeaderValue::from_str(&req_id).unwrap());
        
        // Always include did
        headers.insert("did", HeaderValue::from_str(&self.did).unwrap());
        
        // Add access token if we have it
        if let Some(access_token) = &self.access_token {
            headers.insert("access_token", HeaderValue::from_str(access_token).unwrap());
        }
        
        // Add trade token if requested
        if include_trade_token {
            if let Some(trade_token) = &self.trade_token {
                headers.insert("t_token", HeaderValue::from_str(trade_token).unwrap());
            }
        }
        
        // Add timestamp if requested
        if include_time {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string();
            headers.insert("t_time", HeaderValue::from_str(&timestamp).unwrap());
        }
        
        // Add lzone if requested
        if include_zone {
            headers.insert("lzone", HeaderValue::from_static("dc_core_r001"));
        }
        
        headers
    }
    
    async fn make_request(
        &self,
        method: &str,
        url: &str,
        body: Option<Value>,
        include_trade_token: bool,
        include_time: bool,
        include_zone: bool,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let headers = self.build_headers(include_trade_token, include_time, include_zone);
        
        println!("   → Making {} request to: {}", method, url);
        
        let request = match method {
            "GET" => self.client.get(url).headers(headers),
            "POST" => {
                let req = self.client.post(url).headers(headers);
                if let Some(body) = body {
                    req.json(&body)
                } else {
                    req
                }
            }
            _ => panic!("Unsupported method: {}", method),
        };
        
        let response = request
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;
        
        let status = response.status();
        let text = response.text().await?;
        
        // Try to parse as JSON, but if it fails, return the raw text
        let json_value = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| {
            json!({
                "raw_response": text,
                "status_code": status.as_u16()
            })
        });
        
        Ok(json_value)
    }
}

async fn save_response(
    dir: &str,
    name: &str,
    response: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}.json", dir, name);
    let pretty_json = serde_json::to_string_pretty(response)?;
    fs::write(&path, pretty_json)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    // Create output directory
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = format!("api_responses_raw_{}", timestamp);
    fs::create_dir_all(&output_dir)?;
    
    // Load credentials
    let username = env::var("WEBULL_USERNAME")
        .expect("WEBULL_USERNAME not set");
    let password = env::var("WEBULL_PASSWORD")
        .expect("WEBULL_PASSWORD not set");
    let trading_pin = env::var("WEBULL_TRADING_PIN")
        .expect("WEBULL_TRADING_PIN not set for live trading");
    
    // Load or generate device ID (using bincode like the main library does)
    let did = if let Ok(did_bytes) = fs::read("did.bin") {
        // Try to deserialize with bincode first
        match bincode::deserialize::<String>(&did_bytes) {
            Ok(did) => did,
            Err(_) => {
                // Fallback: treat as raw string
                match String::from_utf8(did_bytes) {
                    Ok(did) => did.trim().to_string(),
                    Err(_) => {
                        // Generate a new one if we can't read it
                        let new_did = Uuid::new_v4().to_string().replace("-", "");
                        let serialized = bincode::serialize(&new_did)?;
                        fs::write("did.bin", &serialized)?;
                        new_did
                    }
                }
            }
        }
    } else {
        let new_did = Uuid::new_v4().to_string().replace("-", "");
        let serialized = bincode::serialize(&new_did)?;
        fs::write("did.bin", &serialized)?;
        new_did
    };
    
    println!("Using device ID: {}", did);
    
    // Test both paper and live (paper first)
    for is_paper in &[true, false] {
        let mode = if *is_paper { "paper" } else { "live" };
        println!("\n{}", "=".repeat(60));
        println!("Testing {} mode", mode);
        println!("{}\n", "=".repeat(60));
        
        let mode_dir = format!("{}/{}", output_dir, mode);
        fs::create_dir_all(&mode_dir)?;
        
        let mut tester = RawApiTester::new(did.clone());
        
        // 1. Login - both use the same endpoint
        println!("1. Testing login...");
        let login_url = tester.endpoints.login();
        let account_type = get_account_type(&username)?;
        let login_data = json!({
            "account": username,
            "accountType": account_type,
            "deviceId": tester.did,
            "deviceName": "API",
            "grade": 1,
            "pwd": hash_password(&password),
            "regionId": 6
        });
        
        let login_response = tester.make_request("POST", &login_url, Some(login_data), false, false, true).await?;
        save_response(&mode_dir, "01_login", &login_response).await?;
        
        // Extract tokens
        if let Some(access_token) = login_response.get("accessToken").and_then(|v| v.as_str()) {
            tester.access_token = Some(access_token.to_string());
            println!("   ✓ Login successful, got access token");
        } else {
            println!("   ✗ Login failed: {:?}", login_response);
            continue;
        }
        
        if let Some(refresh_token) = login_response.get("refreshToken").and_then(|v| v.as_str()) {
            tester.refresh_token = Some(refresh_token.to_string());
        }
        
        if let Some(uuid) = login_response.get("uuid").and_then(|v| v.as_str()) {
            tester.uuid = Some(uuid.to_string());
        }
        
        // 2. Get account ID - different endpoints for paper and live
        if *is_paper {
            println!("2. Getting paper account ID...");
            let paper_account_url = tester.endpoints.paper_account_id();
            let paper_response = tester.make_request("GET", &paper_account_url, None, false, false, true).await?;
            save_response(&mode_dir, "02_paper_account_id", &paper_response).await?;
            
            if let Some(data) = paper_response.as_array() {
                if let Some(first_account) = data.first() {
                    if let Some(paper_id) = first_account.get("paperId") {
                        let paper_id_str = match paper_id {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => String::new(),
                        };
                        if !paper_id_str.is_empty() {
                            tester.paper_account_id = Some(paper_id_str.clone());
                            println!("   ✓ Got paper account ID: {}", paper_id_str);
                        }
                    }
                }
            }
        } else {
            println!("2. Getting account ID...");
            let account_id_url = tester.endpoints.account_id();
            let account_response = tester.make_request("GET", &account_id_url, None, false, false, true).await?;
            save_response(&mode_dir, "02_account_id", &account_response).await?;
            
            if let Some(data) = account_response.get("data").and_then(|v| v.as_array()) {
                if let Some(first_account) = data.first() {
                    if let Some(account_id) = first_account.get("secAccountId") {
                        let account_id_str = match account_id {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => String::new(),
                        };
                        if !account_id_str.is_empty() {
                            tester.account_id = Some(account_id_str.clone());
                            println!("   ✓ Got account ID: {}", account_id_str);
                        }
                    }
                }
            }
            
            // 3. Get trade token (live only)
            println!("3. Getting trade token...");
            let trade_token_url = tester.endpoints.trade_token();
            let trade_token_data = json!({
                "pwd": hash_password(&trading_pin)
            });
            
            let trade_token_response = tester.make_request("POST", &trade_token_url, Some(trade_token_data), false, false, true).await?;
            save_response(&mode_dir, "03_trade_token", &trade_token_response).await?;
            
            let trade_token = trade_token_response
                .get("tradeToken")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    trade_token_response
                        .get("data")
                        .and_then(|d| d.get("tradeToken"))
                        .and_then(|v| v.as_str())
                });
                
            if let Some(token) = trade_token {
                tester.trade_token = Some(token.to_string());
                println!("   ✓ Got trade token");
            }
        }
        
        // 4. Get account details
        println!("4. Getting account details...");
        let account_url = if *is_paper {
            if let Some(paper_id) = &tester.paper_account_id {
                tester.endpoints.paper_account(paper_id)
            } else {
                println!("   ✗ No paper account ID available");
                continue;
            }
        } else {
            if let Some(account_id) = &tester.account_id {
                tester.endpoints.account(account_id)
            } else {
                println!("   ✗ No account ID available");
                continue;
            }
        };
        
        let account_details = tester.make_request("GET", &account_url, None, false, false, true).await?;
        save_response(&mode_dir, "04_account_details", &account_details).await?;
        println!("   ✓ Got account details");
        
        // 5. Get positions (included in account details)
        println!("5. Extracting positions from account details...");
        let empty_array = json!([]);
        let positions = account_details.get("positions").unwrap_or(&empty_array);
        save_response(&mode_dir, "05_positions", &json!({
            "positions": positions
        })).await?;
        println!("   ✓ Extracted positions");
        
        // 6. Find ticker (PROK) - using stock_id endpoint
        println!("6. Finding ticker PROK...");
        let ticker_url = tester.endpoints.stock_id("PROK", 6);
        let ticker_response = tester.make_request("GET", &ticker_url, None, false, false, true).await?;
        save_response(&mode_dir, "06_find_ticker", &ticker_response).await?;
        
        let mut prok_ticker_id: Option<String> = None;
        if let Some(data) = ticker_response.get("data").and_then(|v| v.as_array()) {
            for ticker in data {
                if ticker.get("disSymbol").and_then(|v| v.as_str()) == Some("PROK") 
                    || ticker.get("symbol").and_then(|v| v.as_str()) == Some("PROK") {
                    if let Some(ticker_id) = ticker.get("tickerId") {
                        prok_ticker_id = Some(match ticker_id {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => String::new(),
                        });
                        println!("   ✓ Found PROK, ticker_id: {}", prok_ticker_id.as_ref().unwrap());
                        break;
                    }
                }
            }
        }
        
        if let Some(ticker_id) = &prok_ticker_id {
            // 7. Get quotes
            println!("7. Getting quotes for PROK...");
            let quotes_url = tester.endpoints.quotes(ticker_id);
            let quotes_response = tester.make_request("GET", &quotes_url, None, false, false, true).await?;
            save_response(&mode_dir, "07_quotes", &quotes_response).await?;
            println!("   ✓ Got quotes");
            
            // 8. Get bars/candles
            println!("8. Getting bars for PROK...");
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let bars_url = tester.endpoints.bars(ticker_id, "m1", 50, Some(timestamp));
            let bars_response = tester.make_request("GET", &bars_url, None, false, false, true).await?;
            save_response(&mode_dir, "08_bars", &bars_response).await?;
            println!("   ✓ Got bars");
            
            // 9. Get existing orders
            println!("9. Getting existing orders...");
            let orders_url = if *is_paper {
                if let Some(paper_id) = &tester.paper_account_id {
                    tester.endpoints.paper_orders(paper_id, 100)
                } else {
                    continue;
                }
            } else {
                if let Some(account_id) = &tester.account_id {
                    tester.endpoints.orders(account_id, 100)
                } else {
                    continue;
                }
            };
            
            let orders_response = tester.make_request("GET", &orders_url, None, !is_paper, false, true).await?;
            save_response(&mode_dir, "09_orders_before", &orders_response).await?;
            println!("   ✓ Got existing orders");
            
            // 10. Place buy limit order at $1
            println!("10. Placing buy limit order at $1...");
            let place_order_url = if *is_paper {
                if let Some(paper_id) = &tester.paper_account_id {
                    tester.endpoints.paper_place_order(paper_id, ticker_id)
                } else {
                    continue;
                }
            } else {
                if let Some(account_id) = &tester.account_id {
                    tester.endpoints.place_orders(account_id)
                } else {
                    continue;
                }
            };
            
            let serial_id = Uuid::new_v4().to_string();
            let buy_limit_data = json!({
                "action": "BUY",
                "comboType": "NORMAL",
                "lmtPrice": 1.0,
                "orderType": "LMT",
                "outsideRegularTradingHour": false,
                "quantity": 1,
                "serialId": serial_id,
                "tickerId": ticker_id.parse::<i64>().unwrap_or(0),
                "timeInForce": "DAY"
            });
            
            let buy_limit_response = tester.make_request("POST", &place_order_url, Some(buy_limit_data), !is_paper, !is_paper, true).await?;
            save_response(&mode_dir, "10_place_buy_limit", &buy_limit_response).await?;
            println!("   ✓ Buy limit order response saved");
            
            // Wait a moment
            println!("   Waiting 2 seconds...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // 11. Get orders after placement
            println!("11. Getting orders after placements...");
            let orders_after_response = tester.make_request("GET", &orders_url, None, !is_paper, false, true).await?;
            save_response(&mode_dir, "11_orders_after", &orders_after_response).await?;
            println!("   ✓ Got orders after placements");
            
            // 12. Cancel any pending orders
            println!("12. Cancelling pending orders...");
            let orders_data = if *is_paper {
                orders_after_response.clone()
            } else {
                orders_after_response.get("data").cloned().unwrap_or(json!([]))
            };
            
            if let Some(orders) = orders_data.as_array() {
                for order in orders {
                    if let Some(order_id) = order.get("orderId") {
                        let order_id_str = match order_id {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        
                        let cancel_url = if *is_paper {
                            if let Some(paper_id) = &tester.paper_account_id {
                                tester.endpoints.paper_cancel_order(paper_id, &order_id_str)
                            } else {
                                continue;
                            }
                        } else {
                            if let Some(account_id) = &tester.account_id {
                                let uuid = Uuid::new_v4();
                                format!("{}{}/{}", 
                                    tester.endpoints.cancel_order(account_id), 
                                    order_id_str,
                                    uuid
                                )
                            } else {
                                continue;
                            }
                        };
                        
                        let cancel_response = tester.make_request("POST", &cancel_url, Some(json!({})), !is_paper, !is_paper, true).await?;
                        save_response(&mode_dir, &format!("12_cancel_{}", order_id_str), &cancel_response).await?;
                        println!("   ✓ Cancelled order: {}", order_id_str);
                    }
                }
            }
            
            // 13. Get news
            println!("13. Getting news for PROK...");
            let news_url = tester.endpoints.news(ticker_id, 0, 5);
            let news_response = tester.make_request("GET", &news_url, None, false, false, true).await?;
            save_response(&mode_dir, "13_news", &news_response).await?;
            println!("   ✓ Got news");
            
            // 14. Get options chain
            println!("14. Getting options for PROK...");
            let options_url = tester.endpoints.options("PROK");
            let options_response = tester.make_request("GET", &options_url, None, false, false, true).await?;
            save_response(&mode_dir, "14_options", &options_response).await?;
            println!("   ✓ Got options");
            
            // 15. Get fundamentals
            println!("15. Getting fundamentals for PROK...");
            let fundamentals_url = tester.endpoints.fundamentals(ticker_id);
            let fundamentals_response = tester.make_request("GET", &fundamentals_url, None, false, false, true).await?;
            save_response(&mode_dir, "15_fundamentals", &fundamentals_response).await?;
            println!("   ✓ Got fundamentals");
        }
        
        // 16. Logout
        println!("16. Logging out...");
        let logout_url = tester.endpoints.logout();
        let logout_response = tester.make_request("POST", &logout_url, None, false, false, true).await?;
        save_response(&mode_dir, "16_logout", &logout_response).await?;
        println!("   ✓ Logged out");
    }
    
    println!("\n{}", "=".repeat(60));
    println!("Testing complete! Raw responses saved to: {}", output_dir);
    println!("{}\n", "=".repeat(60));
    
    Ok(())
}