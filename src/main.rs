use crate::controller::role::get_role_scope;
use crate::controller::user::get_user_scope;
use crate::error::Exception;
use crate::service::LoadAllService;
use crate::util::identity::IdentityFactory;
use actix_web::{middleware, App, HttpServer};
use deadpool_postgres::Config as PgConfig;
use deadpool_redis::Config as RedisConfig;
use opt::Opts;
use tokio_postgres::NoTls;

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

#[actix_rt::main]
async fn main() -> Result<(), Exception> {
    let Opts {
        db,
        redis,
        http,
        log,
    } = Opts::open("config.json").await?;

    // 设置日志
    std::env::set_var("RUST_LOG", &log.level.to_string());
    env_logger::init();

    // 初始化连接池，并且尝试取个连接
    let pg_pool = PgConfig::from(db).create_pool(NoTls)?;
    drop(
        pg_pool
            .get()
            .await
            .map_err(|e| format!("Postgres 连接错误: {}", e))?,
    );

    let redis_pool = RedisConfig::from(redis).create_pool()?;
    drop(
        redis_pool
            .get()
            .await
            .map_err(|e| format!("Redis 连接错误: {}", e))?,
    );

    let http_config = http.clone();
    HttpServer::new(move || {
        App::new()
            .load_all_service(pg_pool.clone(), redis_pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                IdentityFactory::new(&http_config.secure_key, redis_pool.clone()).name("identity"),
            )
            .service(get_user_scope())
            .service(get_role_scope())
            .service(actix_files::Files::new("/", &http_config.html).index_file("index.html"))
    })
    .bind(http.addrs.as_slice())?
    .run()
    .await?;

    Ok(())
}
