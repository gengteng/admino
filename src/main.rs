use crate::controller::LoadAllController;
use crate::error::Exception;
use crate::service::LoadAllService;
use crate::util::identity::IdentityFactory;
use actix_web::{middleware, App, HttpServer};
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
            .load_all_service(pg_pool.clone(), redis_pool.clone())
            .load_all_controller()
            .service(actix_files::Files::new("/", &http_config.html).index_file("index.html"))
    })
    .bind(http.addrs.as_slice())?
    .run()
    .await?;

    Ok(())
}
