use super::{PgClient, RedisClient};
use crate::error::{Error, Kind};
use crate::model::*;
use crate::util::types::{AuthCode, Phone};
use deadpool_redis::cmd;
use log::*;
use std::fmt::Display;
use tokio_pg_mapper::FromTokioPostgresRow;

const AUTH_CODE_KEY: &str = "user:authCode";
const AUTH_CODE_EXPIRE: &str = "300";

fn gen_auth_code_key<T: Display>(auth_type: AuthType, identity: &T) -> String {
    format!("{}:{}:{}", AUTH_CODE_KEY, auth_type, identity)
}

pub async fn cache_auth_code<T: Display>(
    redis: &mut RedisClient,
    auth_type: AuthType,
    identity: &T,
    auth_code: &AuthCode,
) -> Result<(), Error> {
    Ok(cmd("SETEX")
        .arg(gen_auth_code_key(auth_type, identity))
        .arg(AUTH_CODE_EXPIRE)
        .arg(&auth_code.code)
        .execute_async(redis)
        .await?)
}

pub async fn check_auth_code<T: Display>(
    redis: &mut RedisClient,
    auth_type: AuthType,
    identity: &T,
    auth_code: &AuthCode,
) -> Result<bool, Error> {
    let key = gen_auth_code_key(auth_type, identity);

    let get_auth_code: Option<String> = cmd("GET").arg(&key).query_async(redis).await?;

    let cached_auth_code = match get_auth_code {
        Some(cached_auth_code) => AuthCode::new(&cached_auth_code)?,
        None => return Err(Kind::INVALID_AUTH_CODE.into()),
    };

    if let Err(e) = cmd("DEL").arg(&key).execute_async(redis).await {
        error!("从 Redis 中删除 {} 时发生错误: {}", key, e);
    }

    Ok(auth_code == &cached_auth_code)
}

pub async fn create_user_with_phone(
    pg: &mut PgClient,
    username: &str,
    nickname: &str,
    phone: &Phone,
) -> Result<UserInfo, Error> {
    let transaction = pg.transaction().await?;

    let user_info = UserInfo::from_row(
        transaction
            .query_one(
                "insert into user_info(username, nickname) values($1, $2) returning *",
                &[&username, &nickname],
            )
            .await?,
    )?;

    transaction.execute("insert into user_auth(user_id, auth_type, identity, credential1) values($1, $2, $3, $4)",
        &[&user_info.id, &AuthType::Phone, phone, &""]).await?;

    transaction.commit().await?;

    Ok(user_info)
}

pub async fn sign_in_with_username(
    pg: &PgClient,
    username: &str,
    _password: &str,
) -> Result<UserInfo, Error> {
    if let Some(row) = pg
        .query_opt("select * from user_info where id in (select user_id from user_auth where auth_type = $1 and identity = $2)",
                   &[&AuthType::Username, &username]).await? {
        Ok(UserInfo::from_row(row)?)
    } else {
        Err(Kind::INVALID_USERNAME_PASSWORD.into())
    }
}

pub async fn sign_in_with_phone(
    pg: &PgClient,
    redis: &mut RedisClient,
    phone: &Phone,
    auth_code: &AuthCode,
) -> Result<UserInfo, Error> {
    if !check_auth_code(redis, AuthType::Phone, phone, auth_code).await? {
        return Err(Kind::INVALID_AUTH_CODE.into());
    }

    let row = pg
        .query_one("select * from user_info where id in (select user_id from user_auth where auth_type = $1 and identity = $2)",
                   &[&AuthType::Phone, phone]).await?;

    Ok(UserInfo::from_row(row)?)
}

pub async fn query_user_by_id(pg: &PgClient, id: Id) -> Result<UserInfo, Error> {
    if let Some(row) = pg
        .query_opt("select * from user_info where id = $1", &[&id])
        .await?
    {
        Ok(UserInfo::from_row(row)?)
    } else {
        Err(Kind::EMPTY_RESULT.into())
    }
}

pub async fn query_user_roles(pg: &PgClient, user_id: Id) -> Result<Vec<Id>, Error> {
    let rows = pg
        .query("select * from user_role where user_id = $1", &[&user_id])
        .await?;

    let mut user_roles = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        user_roles.push(UserRole::from_row_ref(row)?.role_id);
    }

    Ok(user_roles)
}

pub async fn query_user_auth(pg: &PgClient, user_id: Id) -> Result<Vec<UserAuth>, Error> {
    let rows = pg
        .query("select * from user_auth where user_id = $1", &[&user_id])
        .await?;

    let mut auth = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        auth.push(UserAuth::from_row_ref(row)?);
    }

    Ok(auth)
}

pub async fn query_user_perm(pg: &PgClient, user_id: Id) -> Result<Vec<RolePermission>, Error> {
    let rows = pg
        .query("select * from role_permission where role_id in (select role_id from user_role where user_id = $1)", &[&user_id])
        .await?;

    let mut perms = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        perms.push(RolePermission::from_row_ref(row)?);
    }

    Ok(perms)
}
