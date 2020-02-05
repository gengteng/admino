use crate::error::XqlError;
use crate::model::{AuthParams, Id, RegisterParams, RolePerm, UserAuth, UserInfo, UserRole};
use crate::service::RedisClient;
use crate::util::{AuthCode, Phone};
use deadpool_postgres::Client as PgClient;
use deadpool_redis::cmd;
use tokio_pg_mapper::FromTokioPostgresRow;

const AUTH_CODE_KEY: &str = "user:authCode:";
const AUTH_CODE_EXPIRE: &str = "300";

pub async fn cache_auth_code(
    redis: &mut RedisClient,
    phone: &Phone,
    auth_code: &AuthCode,
) -> Result<(), XqlError> {
    Ok(cmd("SETEX")
        .arg(format!("{}{}", AUTH_CODE_KEY, phone))
        .arg(AUTH_CODE_EXPIRE)
        .arg(&auth_code.code)
        .execute_async(redis)
        .await?)
}

pub async fn check_auth_code(
    redis: &mut RedisClient,
    register: &RegisterParams,
) -> Result<bool, XqlError> {
    let auth_code: String = cmd("GET")
        .arg(format!("{}{}", AUTH_CODE_KEY, register.phone))
        .query_async(redis)
        .await?;

    Ok(auth_code == register.auth_code)
}

pub async fn create_user(pg: &PgClient, register: &RegisterParams) -> Result<UserInfo, XqlError> {
    Ok(UserInfo::from_row(
        pg.query_one(
            "insert into user_info(nickname, phone) values($1, $2) returning *",
            &[&register.nickname, &register.phone],
        )
        .await?,
    )?)
}

pub async fn login(pg: &PgClient, auth_info: AuthParams) -> Result<UserInfo, XqlError> {
    let row = pg
        .query_one("select * from user_info where id in (select user_id from user_auth where auth_type = $1 and identity = $2)",
                   &[&auth_info.auth_type, &auth_info.identity]).await?;

    Ok(UserInfo::from_row(row)?)
}

pub async fn query_user_by_id(pg: &PgClient, id: Id) -> Result<UserInfo, XqlError> {
    let row = pg
        .query_one("select * from user_info where id = $1", &[&id])
        .await?;
    Ok(UserInfo::from_row(row)?)
}

pub async fn query_user_roles(pg: &PgClient, user_id: Id) -> Result<Vec<Id>, XqlError> {
    let rows = pg
        .query("select * from user_role where user_id = $1", &[&user_id])
        .await?;

    let mut user_roles = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        user_roles.push(UserRole::from_row_ref(row)?.role_id);
    }

    Ok(user_roles)
}

pub async fn query_user_auth(pg: &PgClient, user_id: Id) -> Result<Vec<UserAuth>, XqlError> {
    let rows = pg
        .query("select * from user_auth where user_id = $1", &[&user_id])
        .await?;

    let mut auth = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        auth.push(UserAuth::from_row_ref(row)?);
    }

    Ok(auth)
}

pub async fn query_user_perm(pg: &PgClient, user_id: Id) -> Result<Vec<RolePerm>, XqlError> {
    let rows = pg
        .query("select * from role_perm where role_id in (select role_id from user_role where user_id = $1)", &[&user_id])
        .await?;

    let mut perms = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        perms.push(RolePerm::from_row_ref(row)?);
    }

    Ok(perms)
}
