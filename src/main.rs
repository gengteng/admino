//! Admino 是一个计划使用 Actix 2.0+ 实现后端，Angular 8+ 实现前端，
//! PostgreSQL 作为数据库，Redis 作为缓存的后台管理系统。
//!
use crate::controller::LoadAllControllers;
use crate::error::Exception;
use crate::service::LoadAllServices;
use crate::util::identity::IdentityFactory;
use actix_web::{middleware, App, HttpServer};
use futures::TryFutureExt;
use opt::Opts;

mod controller;
mod error;
mod model;
mod opt;
mod service;
mod util;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate postgres_types;

#[macro_use]
extern crate log;

/// 入口函数
///
/// # 主要流程
///
/// 1. 从 config.toml 或 config.json 中读取所有配置(Opts)；
/// 2. 设置日志级别；
/// 3. 初始化 PostgreSQL 和 Redis 连接池，并取连接验证；
/// 4. 设置各种中间件，加载控制器和服务，并启动 HTTP 服务。
///
#[actix_rt::main]
async fn main() -> Result<(), Exception> {
    let Opts {
        db,
        redis,
        http,
        log,
    } = Opts::open_toml("config.toml")
        .or_else(|_e| Opts::open_json("config.json"))
        .await?;

    // 设置日志
    std::env::set_var("RUST_LOG", &log.level.to_string());
    env_logger::init();

    // 初始化连接池，并且尝试取个连接，让问题提前暴露
    // 因为连接池是懒加载的，初始化时并不会建立连接，只有在真正运行起来才会暴露连接错误
    let pg_pool = db.create_pool()?;
    drop(
        pg_pool
            .get()
            .await
            .map_err(|e| format!("Postgres 连接错误: {}", e))?,
    );

    let redis_pool = redis.create_pool()?;
    drop(
        redis_pool
            .get()
            .await
            .map_err(|e| format!("Redis 连接错误: {}", e))?,
    );

    let http_config = http.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                IdentityFactory::new(&http_config.secure_key, redis_pool.clone()).name("identity"),
            )
            .load_all_services(pg_pool.clone(), redis_pool.clone())
            .load_all_controllers()
            .service(actix_files::Files::new("/", &http_config.html).index_file("index.html"))
    })
    .bind(http.addrs.as_slice())?
    .run()
    .await?;

    Ok(())
}
