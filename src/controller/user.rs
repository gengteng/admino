use crate::error::XqlError;
use crate::model::{
    AuthParams, GetAuthCodeParams, Id, RegisterParams, RolePerm, UserAuth, UserInfo,
};
use crate::service::{user::*, IntoJsonResult};
use crate::util::identity::Identity;
use crate::util::AuthCode;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, Scope};
use deadpool_postgres::Pool as PgPool;
use deadpool_redis::Pool as RedisPool;

pub fn get_user_scope() -> Scope {
    web::scope("/user")
        .service(web::resource("/authCode").route(web::post().to(get_auth_code)))
        .service(web::resource("/register").route(web::post().to(register)))
        .service(web::resource("/signIn").route(web::post().to(sign_in)))
        .service(web::resource("/signOut").route(web::post().to(sign_out)))
        .service(web::resource("/info").route(web::get().to(get_user_info)))
        .service(web::resource("/role").route(web::get().to(get_user_role)))
        .service(web::resource("/perm").route(web::get().to(get_user_perm)))
        .service(web::resource("/auth").route(web::get().to(get_user_auth)))
}

async fn get_auth_code(
    get_auth_param: Json<GetAuthCodeParams>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<AuthCode>, XqlError> {
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
) -> Result<Json<UserInfo>, XqlError> {
    let reg_param = reg_param.into_inner();

    let mut redis_client = redis_pool.get().await?;

    if !check_auth_code(&mut redis_client, &reg_param).await? {
        return Err(XqlError::static_custom(
            StatusCode::BAD_REQUEST,
            "验证码错误",
        ));
    }

    let pg_client = pg_pool.get().await?;
    create_user(&pg_client, &reg_param).await.json()
}

async fn sign_in(
    auth_param: Json<AuthParams>,
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, XqlError> {
    let pg_client = pg_pool.get().await?;
    let user_info = login(&pg_client, auth_param.into_inner())
        .await
        .or_else(|_| Err(XqlError::Status(StatusCode::UNAUTHORIZED)))?;

    identity.sign_in(user_info.id)?;

    Ok(Json(user_info))
}

async fn sign_out(identity: Identity) -> Result<&'static str, XqlError> {
    if identity.is_user() {
        identity.sign_out();
        Ok("logged out")
    } else {
        Err(XqlError::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_info(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, XqlError> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_by_id(&pg_client, user_id).await.json()
    } else {
        Err(XqlError::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_role(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<Id>>, XqlError> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_roles(&pg_client, user_id).await.json()
    } else {
        Err(XqlError::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_auth(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<UserAuth>>, XqlError> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_auth(&pg_client, user_id).await.json()
    } else {
        Err(XqlError::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get_user_perm(
    identity: Identity,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<Vec<RolePerm>>, XqlError> {
    if let Some(user_id) = identity.get() {
        let pg_client = pg_pool.get().await?;
        query_user_perm(&pg_client, user_id).await.json()
    } else {
        Err(XqlError::Status(StatusCode::UNAUTHORIZED))
    }
}
