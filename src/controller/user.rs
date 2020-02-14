use super::{IntoJsonResult, PgPool, RedisPool};
use crate::error::{Error, Kind};
use crate::model::*;
use crate::service::user::*;
use crate::util::identity::Identity;
use crate::util::types::{AuthCode, Email, Phone};
use actix_web::web::Json;
use actix_web::{web, Scope};

pub fn get_user_scope() -> Scope {
    web::scope("/user")
        .service(web::resource("/phoneAuthCode").route(web::post().to(send_auth_code_to_phone)))
        .service(web::resource("/emailAuthCode").route(web::post().to(send_auth_code_to_email)))
        .service(web::resource("/register").route(web::post().to(register_with_phone)))
        .service(web::resource("/signIn").route(web::post().to(sign_in)))
        .service(web::resource("/signOut").route(web::post().to(sign_out)))
        .service(web::resource("/info").route(web::get().to(get_user_info)))
        .service(web::resource("/roles").route(web::get().to(get_user_role)))
        .service(web::resource("/permissions").route(web::get().to(get_user_perm)))
        .service(web::resource("/authentications").route(web::get().to(get_user_auth)))
}

async fn send_auth_code_to_phone(
    get_auth_param: Json<GetAuthCodeParams>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<AuthCode>, Error> {
    let phone = Phone::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to phone

    let mut redis_client = redis_pool.get().await?;
    cache_auth_code(&mut redis_client, AuthType::Phone, &phone, &auth_code).await?;

    Ok(Json(auth_code)) // TODO: just send ok
}

async fn send_auth_code_to_email(
    get_auth_param: Json<GetAuthCodeParams>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<AuthCode>, Error> {
    let email = Email::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to email address

    let mut redis_client = redis_pool.get().await?;
    cache_auth_code(&mut redis_client, AuthType::Email, &email, &auth_code).await?;

    Ok(Json(auth_code)) // TODO: just send ok
}

async fn register_with_phone(
    reg_param: Json<RegisterParams>,
    redis_pool: web::Data<RedisPool>,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserInfo>, Error> {
    let reg_param = reg_param.into_inner();

    let phone = Phone::new(&reg_param.phone)?;
    let auth_code = AuthCode::new(&reg_param.auth_code)?;

    let mut redis_client = redis_pool.get().await?;
    if !check_auth_code(&mut redis_client, AuthType::Phone, &phone, &auth_code).await? {
        return Err(Kind::INVALID_AUTH_CODE.into());
    }

    let mut pg_client = pg_pool.get().await?;
    create_user_with_phone(
        &mut pg_client,
        &reg_param.username,
        &reg_param.nickname,
        &phone,
    )
    .await
    .json()
}

async fn sign_in(
    sign_in_params: Json<SignInParams>,
    identity: Identity,
    pg_pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<UserInfo>, Error> {
    let sign_in_params = sign_in_params.into_inner();

    let pg_client = pg_pool.get().await?;
    let user_info = match sign_in_params.auth_type {
        AuthType::Username => sign_in_with_username(
            &pg_client,
            &sign_in_params.identity,
            &sign_in_params.credential1,
        )
        .await
        .or_else(|_| Err(Error::kind(Kind::INVALID_USERNAME_PASSWORD)))?,
        AuthType::Phone => {
            let phone = Phone::new(&sign_in_params.identity)?;
            let auth_code = AuthCode::new(&sign_in_params.credential1)?;

            let mut redis_client = redis_pool.get().await?;
            sign_in_with_phone(&pg_client, &mut redis_client, &phone, &auth_code).await?
        }
        AuthType::Email => unimplemented!(),
    };

    identity.sign_in(user_info.id)?;

    Ok(Json(user_info))
}

async fn sign_out(identity: Identity) -> Result<&'static str, Error> {
    if identity.is_user() {
        identity.sign_out();
        Ok("logged out")
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
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
        Err(Kind::USER_NOT_SIGNED_IN.into())
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
        Err(Kind::USER_NOT_SIGNED_IN.into())
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
        Err(Kind::USER_NOT_SIGNED_IN.into())
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
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}
