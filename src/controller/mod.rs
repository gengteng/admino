pub mod user;
pub mod role;

use deadpool_postgres::Pool as PgPool;
use deadpool_redis::Pool as RedisPool;
use actix_web::web::Json;
use crate::error::Error;

pub trait IntoJsonResult<T> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T> IntoJsonResult<T> for Result<T, Error> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json)
    }
}