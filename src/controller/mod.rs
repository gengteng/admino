//! 控制器（Controller）的实现
//!
mod permission;
mod role;
mod user;

use crate::error::Error;
use actix_service::ServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::Json;
use actix_web::App;

/// 将一个 `Result<T, E>` 类型（且满足 `E: Into<Error>`）转换为 Result<Json<T>, Error> 类型
///
/// 调用 Service 时返回的结果通常为 `Result<T, E>`，而 Controller 的返回值通常为 Result<Json<T>, Error>,
/// 使用这个 trait 可以很方便的从前者转换到后者。
///
/// # Example
///
/// ```no_run
/// async fn get_role(
///     role_svc: web::Data<RoleService>,
///     id: web::Path<Id>,
/// ) -> Result<Json<Role>, Error> {
///     role_svc.query_role(id.into_inner()).await.json()
/// }
/// ```
///
pub(self) trait IntoJsonResult<T, E: Into<Error>> {
    fn json(self) -> Result<Json<T>, Error>;
}

impl<T, E: Into<Error>> IntoJsonResult<T, E> for Result<T, E> {
    fn json(self) -> Result<Json<T>, Error> {
        self.map(Json).map_err(E::into)
    }
}

/// 加载所有控制器，已为 `actix_web::app:App` 实现这个 `trait`，
/// 详见 `main.rs` 中对 `load_all_controllers` 函数的调用
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
