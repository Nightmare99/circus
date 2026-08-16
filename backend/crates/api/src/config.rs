pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub storage_dir: String,
    pub max_upload_mb: usize,
    /// Directory containing the built frontend (`index.html` + assets). When
    /// set, the API also serves the SPA — this is what the production Docker
    /// image does. Left unset in local dev, where Vite serves the frontend.
    pub static_dir: Option<String>,
    pub bootstrap_admin_email: Option<String>,
    pub bootstrap_admin_password: Option<String>,
    pub bootstrap_admin_name: String,
    /// Mark the refresh-token cookie `Secure`. Must be true in production
    /// (served over TLS); default false so local http dev isn't broken.
    pub cookie_secure: bool,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            bind_addr: env_or("BIND_ADDR", "0.0.0.0:8080"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_ttl_minutes: env_or("ACCESS_TOKEN_TTL_MINUTES", "15")
                .parse()
                .expect("ACCESS_TOKEN_TTL_MINUTES must be an integer"),
            refresh_token_ttl_days: env_or("REFRESH_TOKEN_TTL_DAYS", "7")
                .parse()
                .expect("REFRESH_TOKEN_TTL_DAYS must be an integer"),
            storage_dir: env_or("STORAGE_DIR", "./data/attachments"),
            max_upload_mb: env_or("MAX_UPLOAD_MB", "25")
                .parse()
                .expect("MAX_UPLOAD_MB must be an integer"),
            static_dir: std::env::var("STATIC_DIR").ok(),
            bootstrap_admin_email: std::env::var("BOOTSTRAP_ADMIN_EMAIL").ok(),
            bootstrap_admin_password: std::env::var("BOOTSTRAP_ADMIN_PASSWORD").ok(),
            bootstrap_admin_name: env_or("BOOTSTRAP_ADMIN_NAME", "Admin"),
            cookie_secure: env_or("COOKIE_SECURE", "false")
                .parse()
                .expect("COOKIE_SECURE must be true or false"),
        }
    }
}
