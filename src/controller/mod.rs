mod permission;
mod role;
mod user;

use crate::error::Error;
use actix_service::ServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::Json;
use actix_web::App;

pub(self) trait IntoJsonResult<T, E: Into<Error>> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T, E: Into<Error>> IntoJsonResult<T, E> for Result<T, E> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json).map_err(E::into)
    }
}

pub trait LoadAllControllers {
    fn load_all_controllers(self) -> Self;
}

impl<T, B> LoadAllControllers for App<T, B>
where
    B: MessageBody,
    T: ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = actix_web::Error,
        InitError = (),
    >,
{
    fn load_all_controllers(self) -> Self {
        self.service(user::get_user_scope())
            .service(role::get_role_scope())
            .service(permission::get_permission_scope())
    }
}
