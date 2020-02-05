use crate::error::XqlError;
use actix_web::web::Json;
use deadpool_redis::ConnectionWrapper;

pub mod user;

pub type RedisClient = deadpool::managed::Object<ConnectionWrapper, redis::RedisError>;

pub trait IntoJsonResult<T> {
    fn json(self) -> Result<Json<T>, XqlError>;
}

impl<T> IntoJsonResult<T> for Result<T, XqlError> {
    fn json(self) -> Result<Json<T>, XqlError> {
        self.map(Json)
    }
}
