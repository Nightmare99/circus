pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    // Wired up in the auth phase (JWT signing/verification).
    #[allow(dead_code)]
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}
