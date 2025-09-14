use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: Uuid,
    pub user_id: Uuid,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, 
    pub exp: i64,       
    pub iat: i64,  
    pub jti: String,    
    pub iss: String,   
    pub aud: String,   
    pub typ: String,   
    pub nonce: String, 
    pub nbf: i64,    
}

pub fn generate_jwt_token(
    user_id: Uuid,
    max_age: i64,
    private_key: String,
    token_type: &str,
    config: &Config,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_in = now + Duration::minutes(max_age);
    let token_uuid = Uuid::new_v4();

    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp: expires_in.timestamp(),
        iat: now.timestamp(),
        jti: token_uuid.to_string(),
        iss: config.token_issuer.clone(),
        aud: config.token_audience.clone(),
        typ: token_type.to_string(),
        nonce: Uuid::new_v4().to_string(), 
        nbf: now.timestamp(),              
    };

    let header = Header::new(Algorithm::RS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(private_key.as_bytes())?,
    )?;

    Ok(TokenDetails {
        token: Some(token),
        token_uuid,
        user_id,
        expires_in: expires_in.timestamp(),
    })
}

pub async fn verify_jwt_token(
    public_key: String,
    token: &str,
    config: &Config,
    redis_client: &mut redis::aio::Connection,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_nbf = true; 
    validation.set_issuer(&[config.token_issuer.as_str()]);
    validation.set_audience(&[config.token_audience.as_str()]);
    validation.leeway = 60;

    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())?;
    let decoded = decode::<TokenClaims>(token, &decoding_key, &validation)?;

    if decoded.claims.typ != "access" && decoded.claims.typ != "refresh" {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    let nonce_key = format!("nonce:{}", decoded.claims.nonce);
    if redis_client
        .exists(&nonce_key)
        .await
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?
    {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    let token_lifetime = (decoded.claims.exp - decoded.claims.iat) as usize;
    redis_client
        .set_ex(
            &nonce_key,
            "used",
            token_lifetime + 300,
        )
        .await
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?;

    let user_id = Uuid::parse_str(&decoded.claims.sub)
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidSubject)?;

    let token_uuid = Uuid::parse_str(&decoded.claims.jti)
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?;

    Ok(TokenDetails {
        token: None,
        token_uuid,
        user_id,
        expires_in: decoded.claims.exp,
    })
}
