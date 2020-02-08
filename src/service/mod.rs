pub mod user;
pub mod role;

pub type RedisClient = deadpool::managed::Object<deadpool_redis::ConnectionWrapper, redis::RedisError>;
pub type PgClient = deadpool_postgres::Client;
