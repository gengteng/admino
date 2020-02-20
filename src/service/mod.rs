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

pub trait LoadAllService {
    fn load_all_service(self, pg_pool: PgPool, redis_pool: RedisPool) -> Self;
}

impl<T, B> LoadAllService for App<T, B>
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
    fn load_all_service(self, pg_pool: PgPool, redis_pool: RedisPool) -> Self {
        self.data(UserService::new(pg_pool.clone(), redis_pool))
            .data(RoleService::new(pg_pool.clone()))
            .data(PermissionService::new(pg_pool))
    }
}
