pub mod role;
pub mod user;

use crate::error::Error;
use actix_web::web::Json;

pub trait IntoJsonResult<T, E: Into<Error>> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T, E: Into<Error>> IntoJsonResult<T, E> for Result<T, E> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json).map_err(E::into)
    }
}
