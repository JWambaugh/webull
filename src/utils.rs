use crate::error::{Result, WebullError};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Generate or load a device ID from file
pub fn get_did(path: Option<&Path>) -> Result<String> {
    let filename = if let Some(p) = path {
        p.join("did.bin")
    } else {
        PathBuf::from("did.bin")
    };

    if filename.exists() {
        let mut file = File::open(&filename)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        // Try to deserialize with bincode
        match bincode::deserialize::<String>(&contents) {
            Ok(did) => Ok(did),
            Err(_) => {
                // Fallback: treat as raw string
                String::from_utf8(contents).map_err(|e| WebullError::DeviceIdError(e.to_string()))
            }
        }
    } else {
        let did = Uuid::new_v4().to_string().replace("-", "");
        save_did(&did, Some(&filename))?;
        Ok(did)
    }
}

/// Save device ID to file
pub fn save_did(did: &str, path: Option<&Path>) -> Result<()> {
    let filename = if let Some(p) = path {
        p.to_path_buf()
    } else {
        PathBuf::from("did.bin")
    };

    // Ensure parent directory exists
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized =
        bincode::serialize(did).map_err(|e| WebullError::SerializationError(e.to_string()))?;

    let mut file = File::create(&filename)?;
    file.write_all(&serialized)?;
    Ok(())
}

/// Hash password with Webull's salt
pub fn hash_password(password: &str) -> String {
    let salted = format!("wl_app-a&b@!423^{}", password);
    format!("{:x}", md5::compute(salted.as_bytes()))
}

/// Determine account type from username
pub fn get_account_type(username: &str) -> Result<i32> {
    // Check if it's an email
    if username.contains('@') {
        if validate_email(username) {
            return Ok(2); // Email account
        } else {
            return Err(WebullError::InvalidParameter(
                "Invalid email format".to_string(),
            ));
        }
    }

    // Check if it's a phone number
    if username.starts_with('+') {
        return Ok(1); // Phone account
    }

    // Default to email type if no clear indicator
    Ok(2)
}

/// Simple email validation
pub fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    if domain_parts.len() < 2 {
        return false;
    }

    // Basic checks
    !parts[0].is_empty() && domain_parts.iter().all(|p| !p.is_empty())
}

/// Convert timestamp to human-readable format
pub fn timestamp_to_string(timestamp: i64) -> String {
    use chrono::DateTime;

    let datetime = DateTime::from_timestamp(timestamp / 1000, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Parse time interval string (e.g., "1m", "5m", "1h", "1d")
pub fn parse_interval(interval: &str) -> Result<String> {
    let valid_intervals = vec![
        "1m", "3m", "5m", "15m", "30m", "60m", "120m", "240m", "1h", "2h", "4h", "1d", "1w", "1M",
        "d1", "d5", "m1", "m5", "m15", "m30", "m60", "m120", "m240", "h1", "h2", "h4", "w1", "mo1",
    ];

    if valid_intervals.contains(&interval) {
        Ok(interval.to_string())
    } else {
        Err(WebullError::InvalidParameter(format!(
            "Invalid interval: {}",
            interval
        )))
    }
}

/// Format a float to a string with specified decimal places
pub fn format_price(price: f64, decimals: usize) -> String {
    format!("{:.prec$}", price, prec = decimals)
}

/// Convert region string to region code
pub fn get_region_code(region: Option<&str>) -> i32 {
    match region {
        Some("us") | Some("US") => 6,
        Some("cn") | Some("CN") => 1,
        Some("hk") | Some("HK") => 2,
        _ => 6, // Default to US
    }
}

/// Generate a unique request ID
pub fn generate_req_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

/// Base64 encode
pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Base64 decode
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>> {
    general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| WebullError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test123";
        let hashed = hash_password(password);
        assert!(!hashed.is_empty());
        assert_eq!(hashed.len(), 32); // MD5 hash is 32 characters
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.co.uk"));
        assert!(!validate_email("invalid"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("test@"));
    }

    #[test]
    fn test_get_account_type() {
        assert_eq!(get_account_type("test@example.com").unwrap(), 2);
        assert_eq!(get_account_type("+1234567890").unwrap(), 1);
    }

    #[test]
    fn test_parse_interval() {
        assert!(parse_interval("1m").is_ok());
        assert!(parse_interval("1d").is_ok());
        assert!(parse_interval("invalid").is_err());
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(123.456789, 2), "123.46");
        assert_eq!(format_price(0.001234, 4), "0.0012");
    }
}
