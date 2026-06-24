use serde::Deserialize;

/// Non-sensitive, toml-provided settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TomlSettings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub pagination: PaginationSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtSettings {
    /// token lifetime in seconds
    pub expires_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationSettings {
    pub default_page: u64,
    pub default_page_size: u64,
    pub max_page_size: u64,
}

/// Full runtime configuration: toml-provided fields + secrets loaded from the
/// environment. Secrets are optional so the process can boot (and serve health
/// checks) without a fully configured environment; each feature validates the
/// secrets it needs at the point of use.
#[derive(Debug, Clone)]
pub struct Settings {
    pub env: String,
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub pagination: PaginationSettings,

    pub database_url: Option<String>,
    pub jwt_secret: Option<String>,
    pub wechat_appid: Option<String>,
    pub wechat_secret: Option<String>,
    pub qiniu_access_key: Option<String>,
    pub qiniu_secret_key: Option<String>,
    pub cos_secret_id: Option<String>,
    pub cos_secret_key: Option<String>,
    pub cos_bucket: Option<String>,
    pub cos_region: Option<String>,
}

impl Settings {
    /// Load toml config files (`config/default.toml` + `config/<RUN_ENV>.toml`)
    /// and merge secrets from the environment (`.env` already loaded by caller).
    pub fn load() -> anyhow::Result<Self> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(true))
            .add_source(config::File::with_name(&format!("config/{env}")).required(false))
            .build()?;

        let toml: TomlSettings = cfg.try_deserialize()?;

        Ok(Self {
            env,
            server: toml.server,
            database: toml.database,
            jwt: toml.jwt,
            pagination: toml.pagination,
            database_url: env_opt("DATABASE_URL"),
            jwt_secret: env_opt("JWT_SECRET"),
            wechat_appid: env_opt("WX_APPID"),
            wechat_secret: env_opt("WX_SECRET"),
            qiniu_access_key: env_opt("QINIU_ACCESS_KEY"),
            qiniu_secret_key: env_opt("QINIU_SECRET_KEY"),
            cos_secret_id: env_opt("COS_SECRET_ID"),
            cos_secret_key: env_opt("COS_SECRET_KEY"),
            cos_bucket: env_opt("COS_BUCKET"),
            cos_region: env_opt("COS_REGION"),
        })
    }
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}
