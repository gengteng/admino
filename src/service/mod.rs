pub mod role;
pub mod user;

pub type RedisClient =
    deadpool::managed::Object<deadpool_redis::ConnectionWrapper, redis::RedisError>;
pub type PgClient = deadpool_postgres::Client;
