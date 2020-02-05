use crate::error::Error;
use actix_web::web::Json;
use deadpool_redis::ConnectionWrapper;

pub mod user;

pub type RedisClient = deadpool::managed::Object<ConnectionWrapper, redis::RedisError>;

pub trait IntoJsonResult<T> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T> IntoJsonResult<T> for Result<T, Error> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json)
    }
}
