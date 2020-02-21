//! 服务（Service）的实现，使用 deadpool 连接池访问 PostgreSQL / Redis
use crate::opt::{PgPool, RedisPool};
use crate::service::permission::PermissionService;
use crate::service::role::RoleService;
use crate::service::user::UserService;
use actix_service::ServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::App;

pub(crate) mod permission;
pub(crate) mod role;
pub(crate) mod user;

/// 加载所有服务，已为 `actix_web::app:App` 实现这个 `trait`，
/// 详见 `main.rs` 中对 `load_all_services` 函数的调用
pub trait LoadAllServices {
    fn load_all_services(self, pg_pool: PgPool, redis_pool: RedisPool) -> Self;
}

impl<T, B> LoadAllServices for App<T, B>
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
    fn load_all_services(self, pg_pool: PgPool, redis_pool: RedisPool) -> Self {
        self.data(UserService::new(pg_pool.clone(), redis_pool))
            .data(RoleService::new(pg_pool.clone()))
            .data(PermissionService::new(pg_pool))
    }
}
