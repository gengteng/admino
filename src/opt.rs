use crate::error::Exception;
use deadpool_postgres::Config as PgConfig;
use deadpool_redis::Config as RedisConfig;
use log::Level;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::prelude::*;
use tokio_postgres::NoTls;

pub type RedisPool = deadpool_redis::Pool;
pub type PgPool = deadpool_postgres::Pool;

/// 所有配置项
#[derive(Debug, Serialize, Deserialize)]
pub struct Opts {
    pub db: DbOpts,
    pub redis: RedisOpts,
    pub http: HttpOpts,
    pub log: LogOpts,
}

impl Opts {
    /// 打开配置文件
    pub async fn open_json<P: AsRef<Path>>(path: P) -> Result<Self, Exception> {
        let vec = Self::open(path).await?;
        Ok(serde_json::from_slice(vec.as_slice())?)
    }

    pub async fn open_toml<P: AsRef<Path>>(path: P) -> Result<Self, Exception> {
        let vec = Self::open(path).await?;
        Ok(toml::from_slice(vec.as_slice())?)
    }

    async fn open<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Exception> {
        let mut file = File::open(path.as_ref()).await?;
        let mut vec = Vec::with_capacity(file.metadata().await?.len() as usize);
        file.read_to_end(&mut vec).await?;
        Ok(vec)
    }
}

/// 数据库配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbOpts {
    host: String,
    database: String,
    username: String,
    password: String,
}

impl DbOpts {
    /// 使用数据库配置直接创建连接池
    pub fn create_pool(self) -> Result<PgPool, Exception> {
        Ok(PgConfig::from(self).create_pool(NoTls)?)
    }
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

/// Redis配置项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisOpts {
    pub url: String,
}

impl RedisOpts {
    /// 使用 Redis 配置直接创建连接池
    pub fn create_pool(self) -> Result<RedisPool, Exception> {
        Ok(RedisConfig::from(self).create_pool()?)
    }
}

impl From<RedisOpts> for RedisConfig {
    fn from(opts: RedisOpts) -> Self {
        let mut redis_config = RedisConfig::default();
        redis_config.url = Some(opts.url);
        redis_config
    }
}

/// http 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpOpts {
    pub addrs: Vec<SocketAddr>,
    pub html: PathBuf,
    #[serde(rename = "secure-key", with = "hex_serde")]
    pub secure_key: [u8; 32],
}

/// 日志配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogOpts {
    pub level: Level,
}
