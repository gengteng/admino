//! 用户及登录相关服务
use crate::error::{Error, Kind};
use crate::model::*;
use crate::opt::{PgPool, RedisPool};
use crate::util::types::{AuthCode, Phone, Username};
use deadpool_redis::cmd;
use std::fmt::Display;
use tokio_pg_mapper::FromTokioPostgresRow;

/// 用户及登录相关服务
pub struct UserService {
    pg_pool: PgPool,
    redis_pool: RedisPool,
}

impl UserService {
    pub fn new(pg_pool: PgPool, redis_pool: RedisPool) -> Self {
        Self {
            pg_pool,
            redis_pool,
        }
    }

    const AUTH_CODE_KEY: &'static str = "user:authCode";
    const AUTH_CODE_EXPIRE: &'static str = "300";

    fn gen_auth_code_key<T: Display>(auth_type: AuthType, identity: &T) -> String {
        format!("{}:{}:{}", UserService::AUTH_CODE_KEY, auth_type, identity)
    }

    pub async fn cache_auth_code<T: Display>(
        &self,
        auth_type: AuthType,
        identity: &T,
        auth_code: &AuthCode,
    ) -> Result<(), Error> {
        let mut redis = self.redis_pool.get().await?;
        Ok(cmd("SETEX")
            .arg(Self::gen_auth_code_key(auth_type, identity))
            .arg(UserService::AUTH_CODE_EXPIRE)
            .arg(&auth_code.code)
            .execute_async(&mut redis)
            .await?)
    }

    pub async fn check_auth_code<T: Display>(
        &self,
        auth_type: AuthType,
        identity: &T,
        auth_code: &AuthCode,
    ) -> Result<bool, Error> {
        let key = Self::gen_auth_code_key(auth_type, identity);

        let mut redis = self.redis_pool.get().await?;

        let get_auth_code: Option<String> = cmd("GET").arg(&key).query_async(&mut redis).await?;

        let cached_auth_code = match get_auth_code {
            Some(cached_auth_code) => AuthCode::new(&cached_auth_code)?,
            None => return Err(Kind::INVALID_AUTH_CODE.into()),
        };

        if let Err(e) = cmd("DEL").arg(&key).execute_async(&mut redis).await {
            error!("从 Redis 中删除 {} 时发生错误: {}", key, e);
        }

        Ok(auth_code == &cached_auth_code)
    }

    pub async fn create_user_with_phone(
        &self,
        username: &Username,
        nickname: &str,
        phone: &Phone,
    ) -> Result<UserInfo, Error> {
        let mut pg = self.pg_pool.get().await?;

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
        &self,
        username: &Username,
        _password: &str,
    ) -> Result<UserInfo, Error> {
        let pg = self.pg_pool.get().await?;

        if let Some(row) = pg
            .query_opt("select * from user_info where id in (select user_id from user_auth where auth_type = $1 and identity = $2)",
                       &[&AuthType::Username, &username]).await? {
            Ok(UserInfo::from_row(row)?)
        } else {
            Err(Kind::LOGIN_FAILED.into())
        }
    }

    pub async fn sign_in_with_phone(
        &self,
        phone: &Phone,
        auth_code: &AuthCode,
    ) -> Result<UserInfo, Error> {
        if !self
            .check_auth_code(AuthType::Phone, phone, auth_code)
            .await?
        {
            return Err(Kind::INVALID_AUTH_CODE.into());
        }

        let pg = self.pg_pool.get().await?;
        if let Some(row) = pg
            .query_opt("select * from user_info where id in (select user_id from user_auth where auth_type = $1 and identity = $2)",
                       &[&AuthType::Phone, phone]).await? {
            Ok(UserInfo::from_row(row)?)
        } else {
            Err(Kind::LOGIN_FAILED.into())
        }
    }

    pub async fn query_user_by_id(&self, id: Id) -> Result<UserInfo, Error> {
        let pg = self.pg_pool.get().await?;

        if let Some(row) = pg
            .query_opt("select * from user_info where id = $1", &[&id])
            .await?
        {
            Ok(UserInfo::from_row(row)?)
        } else {
            Err(Kind::EMPTY_RESULT.into())
        }
    }

    pub async fn query_user_roles(&self, user_id: Id) -> Result<Vec<Role>, Error> {
        let pg = self.pg_pool.get().await?;

        let rows = pg
            .query(
                "select * from role where id in (select role_id from user_role where user_id = $1)",
                &[&user_id],
            )
            .await?;

        let mut roles = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            roles.push(Role::from_row_ref(row)?);
        }

        Ok(roles)
    }

    pub async fn query_user_auth(&self, user_id: Id) -> Result<Vec<UserAuth>, Error> {
        let pg = self.pg_pool.get().await?;

        let rows = pg
            .query("select * from user_auth where user_id = $1", &[&user_id])
            .await?;

        let mut auth = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            auth.push(UserAuth::from_row_ref(row)?);
        }

        Ok(auth)
    }

    // TODO: 应该返回 `Vec<Permission>`，待修改
    pub async fn query_user_perm(&self, user_id: Id) -> Result<Vec<RolePermission>, Error> {
        let pg = self.pg_pool.get().await?;

        let rows = pg
            .query("select * from role_permission where role_id in (select role_id from user_role where user_id = $1)", &[&user_id])
            .await?;

        let mut perms = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            perms.push(RolePermission::from_row_ref(row)?);
        }

        Ok(perms)
    }
}
