use crate::model::Id;
use crate::util::Phone;
use chrono::{NaiveDate, NaiveDateTime};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_info")]
pub struct UserInfo {
    pub id: Id,
    pub nickname: String,
    pub phone: Phone,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub gender: i32,
    pub birthday: Option<NaiveDate>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSql, FromSql)]
pub enum AuthType {
    Phone,
    Username,
}

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role")]
pub struct Role {
    pub id: Id,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_role")]
pub struct UserRole {
    pub user_id: Id,
    pub role_id: Id,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role_perm")]
pub struct RolePerm {
    pub role_id: Id,
    pub perm: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AuthParams {
    pub auth_type: AuthType,
    pub identity: String,
    pub credential1: String,
    pub credential2: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RegisterParams {
    pub nickname: String,
    pub phone: Phone,
    pub auth_code: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GetAuthParams {
    pub phone: Phone,
}
