pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub max_pool_size: usize,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let database_url = std::env::var("DATABASE_URL")?;
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8080);
        let max_pool_size = std::env::var("DATABASE_POOL_SIZE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(10);

        Ok(Self {
            database_url,
            port,
            max_pool_size,
        })
    }
}
