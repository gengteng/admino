//! 用户及登录相关控制器
//!
use super::IntoJsonResult;
use crate::error::{Error, Kind};
use crate::model::{
    AuthType, GetAuthCodeParams, Id, RegisterParams, RolePermission, SignInParams, UserAuth,
    UserInfo,
};
use crate::service::user::UserService;
use crate::util::identity::Identity;
use crate::util::types::{AuthCode, Email, Phone, Username};
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
    user_svc: web::Data<UserService>,
) -> Result<Json<AuthCode>, Error> {
    let phone = Phone::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to phone

    user_svc
        .cache_auth_code(AuthType::Phone, &phone, &auth_code)
        .await?;

    Ok(Json(auth_code)) // TODO: just send ok
}

async fn send_auth_code_to_email(
    get_auth_param: Json<GetAuthCodeParams>,
    user_svc: web::Data<UserService>,
) -> Result<Json<AuthCode>, Error> {
    let email = Email::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to email address

    user_svc
        .cache_auth_code(AuthType::Email, &email, &auth_code)
        .await?;

    Ok(Json(auth_code)) // TODO: just send ok
}

async fn register_with_phone(
    reg_param: Json<RegisterParams>,
    user_svc: web::Data<UserService>,
) -> Result<Json<UserInfo>, Error> {
    let reg_param = reg_param.into_inner();

    let username = Username::new(&reg_param.username)?;
    let phone = Phone::new(&reg_param.phone)?;
    let auth_code = AuthCode::new(&reg_param.auth_code)?;

    if !user_svc
        .check_auth_code(AuthType::Phone, &phone, &auth_code)
        .await?
    {
        return Err(Kind::INVALID_AUTH_CODE.into());
    }

    user_svc
        .create_user_with_phone(&username, &reg_param.nickname, &phone)
        .await
        .json()
}

async fn sign_in(
    sign_in_params: Json<SignInParams>,
    identity: Identity,
    user_svc: web::Data<UserService>,
) -> Result<Json<UserInfo>, Error> {
    let sign_in_params = sign_in_params.into_inner();

    let username = Username::new(&sign_in_params.identity)?;

    let user_info = match sign_in_params.auth_type {
        AuthType::Username => user_svc
            .sign_in_with_username(&username, &sign_in_params.credential1)
            .await
            .or_else(|_| Err(Error::simple(Kind::LOGIN_FAILED)))?,
        AuthType::Phone => {
            let phone = Phone::new(&sign_in_params.identity)?;
            let auth_code = AuthCode::new(&sign_in_params.credential1)?;

            user_svc.sign_in_with_phone(&phone, &auth_code).await?
        }
        AuthType::Email => unimplemented!(),
    };

    identity.sign_in(user_info.id)?;

    Ok(Json(user_info))
}

async fn sign_out(identity: Identity) -> Result<&'static str, Error> {
    if identity.is_user() {
        identity.sign_out();
        Ok("")
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

async fn get_user_info(
    identity: Identity,
    user_svc: web::Data<UserService>,
) -> Result<Json<UserInfo>, Error> {
    if let Some(user_id) = identity.get() {
        user_svc.query_user_by_id(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

async fn get_user_role(
    identity: Identity,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<Id>>, Error> {
    if let Some(user_id) = identity.get() {
        user_svc.query_user_roles(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

async fn get_user_auth(
    identity: Identity,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<UserAuth>>, Error> {
    if let Some(user_id) = identity.get() {
        user_svc.query_user_auth(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

async fn get_user_perm(
    identity: Identity,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<RolePermission>>, Error> {
    if let Some(user_id) = identity.get() {
        user_svc.query_user_perm(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}
