use crate::error::Error;
use crate::model::*;
use crate::service::user::*;
use crate::util::identity::Identity;
use crate::util::types::AuthCode;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, Scope};
use super::{PgPool, RedisPool, IntoJsonResult};

pub fn get_user_scope() -> Scope {
    web::scope("/user")
        .service(web::resource("/authCode").route(web::post().to(get_auth_code)))
        .service(web::resource("/register").route(web::post().to(register)))
        .service(web::resource("/signIn").route(web::post().to(sign_in)))
        .service(web::resource("/signOut").route(web::post().to(sign_out)))
        .service(web::resource("/info").route(web::get().to(get_user_info)))
        .service(web::resource("/roles").route(web::get().to(get_user_role)))
        .service(web::resource("/permissions").route(web::get().to(get_user_perm)))
        .service(web::resource("/authentications").route(web::get().to(get_user_auth)))
}

async fn get_auth_code(
    get_auth_param: Json<GetAuthCodeParams>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<AuthCode>, Error> {
    let auth_code = AuthCode::default();

    // TODO: send sms to phone

    let mut redis_client = redis_pool.get().await?;
    cache_auth_code(&mut redis_client, &get_auth_param.phone, &auth_code).await?;

    Ok(Json(auth_code)) // TODO: just send ok
}

async fn register(
    reg_param: Json<RegisterParams>,
    redis_pool: web::Data<RedisPool>,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, Error> {
    let reg_param = reg_param.into_inner();

    let mut redis_client = redis_pool.get().await?;

    if !check_auth_code(&mut redis_client, &reg_param).await? {
        return Err(Error::static_custom(StatusCode::BAD_REQUEST, "验证码错误"));
    }

    let pg_client = pg_pool.get().await?;
    create_user(&pg_client, &reg_param).await.json()
}

async fn sign_in(
    sign_in_params: Json<SignInParams>,
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, Error> {
    let pg_client = pg_pool.get().await?;
    let user_info = login(&pg_client, sign_in_params.into_inner())
        .await
        .or_else(|_| Err(Error::Status(StatusCode::UNAUTHORIZED)))?;

    identity.sign_in(user_info.id)?;

    Ok(Json(user_info))
}

async fn sign_out(identity: Identity) -> Result<&'static str, Error> {
    if identity.is_user() {
        identity.sign_out();
        Ok("logged out")
    } else {
        Err(Error::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_info(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, Error> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_by_id(&pg_client, user_id).await.json()
    } else {
        Err(Error::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_role(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<Id>>, Error> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_roles(&pg_client, user_id).await.json()
    } else {
        Err(Error::Status(StatusCode::UNAUTHORIZED))
    }
}
//
//async fn get_roles(identity: Identity, pager: web::Json<Condition>, pg_pool: web::Data<PgPool>) -> Result<Json<Role>, Error> {
//    Err(Error::Status(StatusCode::UNAUTHORIZED))
//}

async fn get_user_auth(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<UserAuth>>, Error> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_auth(&pg_client, user_id).await.json()
    } else {
        Err(Error::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_perm(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<RolePermission>>, Error> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_perm(&pg_client, user_id).await.json()
    } else {
        Err(Error::Status(StatusCode::UNAUTHORIZED))
    }
}
