use rand::thread_rng;
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};

fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set", var_name))
}

fn get_env_var_or_default(var_name: &str, default: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| default.to_string())
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub client_origin: String,

    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: i64,
    pub access_token_max_age: i64,

    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: i64,
    pub refresh_token_max_age: i64,

    pub token_issuer: String,
    pub token_audience: String,

    pub https_only: bool,
    pub cookie_secure: bool,
}

impl Config {
    pub fn init() -> Config {
        let database_url = get_env_var("DATABASE_URL");
        let redis_url = get_env_var("REDIS_URL");
        let client_origin = get_env_var("CLIENT_ORIGIN");

        let (access_private, access_public, refresh_private, refresh_public) =
            Self::get_env_var_or_generate_rsa_keys();

        let access_token_expires_in = get_env_var_or_default("ACCESS_TOKEN_EXPIRES_IN", "15")
            .parse::<i64>()
            .expect("ACCESS_TOKEN_EXPIRES_IN must be a number");

        let access_token_max_age = get_env_var_or_default("ACCESS_TOKEN_MAX_AGE", "900")
            .parse::<i64>()
            .expect("ACCESS_TOKEN_MAX_AGE must be a number");

        let refresh_token_expires_in = get_env_var_or_default("REFRESH_TOKEN_EXPIRES_IN", "43200")
            .parse::<i64>()
            .expect("REFRESH_TOKEN_EXPIRES_IN must be a number");

        let refresh_token_max_age = get_env_var_or_default("REFRESH_TOKEN_MAX_AGE", "2592000")
            .parse::<i64>()
            .expect("REFRESH_TOKEN_MAX_AGE must be a number");

        let token_issuer = get_env_var_or_default("TOKEN_ISSUER", "secure-app");
        let token_audience = get_env_var_or_default("TOKEN_AUDIENCE", "secure-app-users");

        let https_only = get_env_var_or_default("HTTPS_ONLY", "true")
            .parse::<bool>()
            .unwrap_or(true);

        let cookie_secure = get_env_var_or_default("COOKIE_SECURE", "true")
            .parse::<bool>()
            .unwrap_or(true);

        Config {
            database_url,
            redis_url,
            client_origin,

            access_token_private_key: access_private,
            access_token_public_key: access_public,
            access_token_expires_in,
            access_token_max_age,

            refresh_token_private_key: refresh_private,
            refresh_token_public_key: refresh_public,
            refresh_token_expires_in,
            refresh_token_max_age,

            token_issuer,
            token_audience,

            https_only,
            cookie_secure,
        }
    }

    fn get_env_var_or_generate_rsa_keys() -> (String, String, String, String) {
        match (
            std::env::var("ACCESS_TOKEN_PRIVATE_KEY"),
            std::env::var("ACCESS_TOKEN_PUBLIC_KEY"),
            std::env::var("REFRESH_TOKEN_PRIVATE_KEY"),
            std::env::var("REFRESH_TOKEN_PUBLIC_KEY"),
        ) {
            (Ok(acc_priv), Ok(acc_pub), Ok(ref_priv), Ok(ref_pub)) => {
                println!("✅ Loaded RSA keys from environment");
                (acc_priv, acc_pub, ref_priv, ref_pub)
            }
            _ => {
                println!("🔄 Generating new RSA key pair for all tokens");
                let (private, public) = Self::generate_rsa_key();
                (private.clone(), public.clone(), private, public)
            }
        }
    }

    fn generate_rsa_key() -> (String, String) {
        let mut rng = thread_rng();
        let bits = 2048;

        let private_key =
            RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate RSA private key");

        let public_key = RsaPublicKey::from(&private_key);

        (
            private_key
                .to_pkcs8_pem(LineEnding::LF)
                .expect("Failed to encode private key")
                .to_string(),
            public_key
                .to_public_key_pem(LineEnding::LF)
                .expect("Failed to encode public key")
                .to_string(),
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.access_token_expires_in <= 0 {
            return Err("ACCESS_TOKEN_EXPIRES_IN must be positive".to_string());
        }

        if self.refresh_token_expires_in <= 0 {
            return Err("REFRESH_TOKEN_EXPIRES_IN must be positive".to_string());
        }

        if self.access_token_max_age <= 0 {
            return Err("ACCESS_TOKEN_MAX_AGE must be positive".to_string());
        }

        if self.refresh_token_max_age <= 0 {
            return Err("REFRESH_TOKEN_MAX_AGE must be positive".to_string());
        }

        if !self.access_token_private_key.contains("PRIVATE KEY") {
            return Err("Invalid ACCESS_TOKEN_PRIVATE_KEY format".to_string());
        }

        if !self.access_token_public_key.contains("PUBLIC KEY") {
            return Err("Invalid ACCESS_TOKEN_PUBLIC_KEY format".to_string());
        }

        if !self.refresh_token_private_key.contains("PRIVATE KEY") {
            return Err("Invalid REFRESH_TOKEN_PRIVATE_KEY format".to_string());
        }

        if !self.refresh_token_public_key.contains("PUBLIC KEY") {
            return Err("Invalid REFRESH_TOKEN_PUBLIC_KEY format".to_string());
        }

        if self.access_token_private_key != self.refresh_token_private_key {
            return Err("Access and refresh token private keys must be identical".to_string());
        }

        if self.access_token_public_key != self.refresh_token_public_key {
            return Err("Access and refresh token public keys must be identical".to_string());
        }

        println!("✅ RSA key pairs are identical (access == refresh)");
        Ok(())
    }
}
