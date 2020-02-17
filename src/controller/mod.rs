pub mod role;
pub mod user;

use crate::error::Error;
use actix_web::web::Json;
use deadpool_postgres::Pool as PgPool;
use deadpool_redis::Pool as RedisPool;

pub trait IntoJsonResult<T, E: Into<Error>> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T, E: Into<Error>> IntoJsonResult<T, E> for Result<T, E> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json).map_err(E::into)
    }
}
