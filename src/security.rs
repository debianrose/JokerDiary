use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::net::IpAddr;
use std::str::FromStr;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtToken {
    pub user_id: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

const JWT_SECRET: &[u8] = b"shhh";

pub fn create_jwt(user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Invalid timestamp")
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )?;
    
    Ok(token)
}

pub fn verify_jwt(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

pub fn is_ip_allowed(ip_str: &str) -> bool {
    if let Ok(ip) = IpAddr::from_str(ip_str) {
        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4.is_loopback() {
                    return true;
                }
                
                if ipv4.is_private() {
                    return true;
                }
                
                let octets = ipv4.octets();
                if octets[0] == 192 && octets[1] == 168 {
                    return true;
                }
                
                false
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback()
            }
        }
    } else {
        false
    }
}
