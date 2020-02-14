use super::*;
use chrono::{NaiveDate, NaiveDateTime};

/// 用户
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_info")]
pub struct UserInfo {
    pub id: Id,
    pub username: String,
    pub nickname: String,
    pub avatar: Option<String>,
    pub gender: i32,
    pub birthday: Option<NaiveDate>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
    pub max_role: Option<i64>,
}

/// 授权类型
#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Eq, Clone, ToSql, FromSql)]
pub enum AuthType {
    Username,
    Phone,
    Email,
}

/// 用户授权
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_auth")]
pub struct UserAuth {
    pub user_id: Id,
    pub auth_type: AuthType,
    pub identity: String,
    pub credential1: String,
    pub credential2: Option<String>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

/// 用户角色
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_role")]
pub struct UserRole {
    pub user_id: Id,
    pub role_id: Id,
}

/// -----------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SignInParams {
    pub auth_type: AuthType,
    pub identity: String,
    pub credential1: String,
    pub credential2: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RegisterParams {
    pub username: String,
    pub nickname: String,
    pub phone: String,
    pub auth_code: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GetAuthCodeParams {
    pub identity: String,
}
