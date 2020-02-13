use crate::error::Detail;
use deadpool_postgres::Config as PgConfig;
use deadpool_redis::Config as RedisConfig;
use log::Level;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Opts {
    pub db: DbOpts,
    pub redis: RedisOpts,
    pub http: HttpOpts,
    pub log: LogOpts,
}

impl Opts {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self, Detail> {
        let mut file = File::open(path.as_ref()).await?;
        let mut vec = Vec::with_capacity(file.metadata().await?.len() as usize);
        file.read_to_end(&mut vec).await?;

        Ok(serde_json::from_slice(vec.as_slice())?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbOpts {
    host: String,
    database: String,
    username: String,
    password: String,
}

impl From<DbOpts> for PgConfig {
    fn from(opts: DbOpts) -> Self {
        let mut pg_config = PgConfig::default();
        pg_config.host = Some(opts.host);
        pg_config.dbname = Some(opts.database);
        pg_config.user = Some(opts.username);
        pg_config.password = Some(opts.password);
        pg_config
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisOpts {
    pub url: String,
}

impl From<RedisOpts> for RedisConfig {
    fn from(opts: RedisOpts) -> Self {
        let mut redis_config = RedisConfig::default();
        redis_config.url = Some(opts.url);
        redis_config
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpOpts {
    pub addrs: Vec<SocketAddr>,
    pub html: PathBuf,
    #[serde(rename = "secure-key", with = "hex_serde")]
    pub secure_key: [u8; 32],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogOpts {
    pub level: Level,
}
