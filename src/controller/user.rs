//! 用户及登录相关控制器
//!
//! TODO: 待拆分成用户的增删改查、用户注册登录登出和验证码相关两部分（或三部分？）
use super::IntoJsonResult;
use crate::controller::EmptyBody;
use crate::error::{Error, Kind};
use crate::model::{
    AddPasswordParams, AuthType, GetAuthCodeParams, RegisterParams, Role, RolePermission,
    SignInParams, UserAuth, UserInfo,
};
use crate::service::user::UserService;
use crate::util::types::{AuthCode, Email, Phone, Username};
use crate::util::user::User;
use actix_web::web::Json;
use actix_web::{web, Scope};

/// 获取用户及登录相关的所有路由
pub fn get_user_scope() -> Scope {
    web::scope("/user")
        .service(web::resource("/phoneAuthCode").route(web::post().to(send_auth_code_to_phone)))
        .service(web::resource("/emailAuthCode").route(web::post().to(send_auth_code_to_email)))
        .service(web::resource("/register").route(web::post().to(register_with_phone)))
        .service(web::resource("/signIn").route(web::post().to(sign_in)))
        .service(web::resource("/signOut").route(web::post().to(sign_out)))
        .service(web::resource("/info").route(web::get().to(get_user_info)))
        .service(web::resource("/addPassword").route(web::post().to(add_password)))
        .service(web::resource("/roles").route(web::get().to(get_user_role)))
        .service(web::resource("/authentications").route(web::get().to(get_user_auth)))
        .service(web::resource("/permissions").route(web::get().to(get_user_perm)))
}

/// 发送6位数字验证码到手机号
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/phoneAuthCode
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sat, 22 Feb 2020 12:32:15 GMT
///
/// <Response body is empty>
/// ```
async fn send_auth_code_to_phone(
    get_auth_param: Json<GetAuthCodeParams>,
    user_svc: web::Data<UserService>,
) -> Result<&'static str, Error> {
    let phone = Phone::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to phone
    info!("已给手机号 {} 发送数字验证码 {}", phone, auth_code);

    user_svc
        .cache_auth_code(AuthType::Phone, &phone, &auth_code)
        .await
        .empty_body()
}

/// 发送6位数字验证码到邮箱
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/emailAuthCode
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sat, 22 Feb 2020 12:32:15 GMT
///
/// <Response body is empty>
/// ```
async fn send_auth_code_to_email(
    get_auth_param: Json<GetAuthCodeParams>,
    user_svc: web::Data<UserService>,
) -> Result<&'static str, Error> {
    let email = Email::new(&get_auth_param.identity)?;

    let auth_code = AuthCode::default();

    // TODO: send auth code to email address
    info!("已给电子邮箱 {} 发送数字验证码 {}", email, auth_code);

    user_svc
        .cache_auth_code(AuthType::Email, &email, &auth_code)
        .await
        .empty_body()
}

/// 使用手机号注册
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/register
/// Content-Type: application/json
///
/// {"username": "gengteng", "nickname": "GT", "phone": "+8615120049138", "auth_code": "165908"}
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 197
/// content-type: application/json
/// date: Sun, 23 Feb 2020 13:23:56 GMT
///
/// {
///   "id": 5,
///   "username": "gengteng",
///   "nickname": "GT",
///   "avatar": null,
///   "gender": "Unknown",
///   "birthday": null,
///   "create_time": "2020-02-23T13:23:57.305393",
///   "update_time": "2020-02-23T13:23:57.305393",
///   "max_role": null
/// }
/// ```
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

/// 登录
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/signIn
/// Content-Type: application/json
///
/// {"identity": "+8615120049138","auth_type": "Phone","credential1": "490604"}
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 197
/// content-type: application/json
/// set-cookie: identity=XtltrLDnewNbRrCBFYd5ZVDdeu3+kNwf/L28W28tsdY=D6lDb0WskxWKSHUczlNJHo3aUi4QEy8n; HttpOnly; Max-Age=86400
/// date: Sun, 23 Feb 2020 13:27:25 GMT
///
/// {
///   "id": 5,
///   "username": "gengteng",
///   "nickname": "GT",
///   "avatar": null,
///   "gender": "Unknown",
///   "birthday": null,
///   "create_time": "2020-02-23T13:23:57.305393",
///   "update_time": "2020-02-23T13:23:57.305393",
///   "max_role": null
/// }
/// ```
///
async fn sign_in(
    sign_in_params: Json<SignInParams>,
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<Json<UserInfo>, Error> {
    let sign_in_params = sign_in_params.into_inner();

    let user_info = match sign_in_params.auth_type {
        AuthType::Username => {
            let username = Username::new(&sign_in_params.identity)?;

            user_svc
                .sign_in_with_username(&username, &sign_in_params.credential1)
                .await?
        }
        AuthType::Phone => {
            let phone = Phone::new(&sign_in_params.identity)?;
            let auth_code = AuthCode::new(&sign_in_params.credential1)?;

            user_svc.sign_in_with_phone(&phone, &auth_code).await?
        }
        AuthType::Email => unimplemented!(),
    };

    user.sign_in(user_info.id)?;

    Ok(Json(user_info))
}

/// 登出
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/signOut
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// set-cookie: identity=; Max-Age=0; Expires=Sat, 23 Feb 2019 13:30:39 GMT
/// date: Sun, 23 Feb 2020 13:30:39 GMT
///
/// <Response body is empty>
/// ```
///
async fn sign_out(user: User) -> Result<&'static str, Error> {
    if user.is_user() {
        user.sign_out();
        Ok("")
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

/// 获取当前用户信息
///
/// # Example
///
/// HTTP 请求:
/// ```
/// GET /user/info
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 197
/// content-type: application/json
/// date: Sun, 23 Feb 2020 13:44:30 GMT
///
/// {
///   "id": 5,
///   "username": "gengteng",
///   "nickname": "GT",
///   "avatar": null,
///   "gender": "Unknown",
///   "birthday": null,
///   "create_time": "2020-02-23T13:23:57.305393",
///   "update_time": "2020-02-23T13:23:57.305393",
///   "max_role": null
/// }
/// ```
async fn get_user_info(
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<Json<UserInfo>, Error> {
    if let Some(user_id) = user.get() {
        user_svc.query_user_by_id(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

/// 为当前用户新增登录密码
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// POST /user/addPassword
/// Content-Type: application/json
///
/// {"password": "!23QweAsd"}
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// set-cookie: identity=; Max-Age=0; Expires=Sat, 23 Feb 2019 13:30:39 GMT
/// date: Sun, 23 Feb 2020 13:30:39 GMT
///
/// <Response body is empty>
/// ```
///
async fn add_password(
    add_pwd_params: Json<AddPasswordParams>,
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<&'static str, Error> {
    if let Some(user_id) = user.get() {
        user_svc
            .add_password(user_id, &add_pwd_params.password)
            .await
            .empty_body()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

/// 获取当前用户的所有角色
///
/// # Example
///
/// HTTP 请求:
/// ```
/// GET /user/roles
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 138
/// content-type: application/json
/// date: Sun, 23 Feb 2020 13:48:07 GMT
///
/// [
///   {
///     "id": 1,
///     "name": "超级管理员",
///     "max_user": 1,
///     "max_permission": null
///   },
///   {
///     "id": 5,
///     "name": "角色名",
///     "max_user": 121212,
///     "max_permission": null
///   }
/// ]
/// ```
async fn get_user_role(
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<Role>>, Error> {
    if let Some(user_id) = user.get() {
        user_svc.query_user_roles(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

/// 获取当前用户所有登录方式
///
/// # Example
///
/// HTTP 请求:
/// ```
/// GET /user/authentications
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 185
/// content-type: application/json
/// date: Sun, 23 Feb 2020 13:48:50 GMT
///
/// [
///   {
///     "user_id": 5,
///     "auth_type": "Phone",
///     "identity": "+8615120049138",
///     "credential1": "",
///     "credential2": null,
///     "create_time": "2020-02-23T13:23:57.305393",
///     "update_time": "2020-02-23T13:23:57.305393"
///   }
/// ]
/// ```
async fn get_user_auth(
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<UserAuth>>, Error> {
    if let Some(user_id) = user.get() {
        user_svc.query_user_auth(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}

/// 获取用户所有权限
///
/// TODO: 应该返回 `Vec<Permission>`，待修改
async fn get_user_perm(
    user: User,
    user_svc: web::Data<UserService>,
) -> Result<Json<Vec<RolePermission>>, Error> {
    if let Some(user_id) = user.get() {
        user_svc.query_user_perm(user_id).await.json()
    } else {
        Err(Kind::USER_NOT_SIGNED_IN.into())
    }
}
