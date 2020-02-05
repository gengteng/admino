use crate::model::Id;
use crate::util::Phone;
use chrono::{NaiveDate, NaiveDateTime};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio_pg_mapper_derive::PostgresMapper;

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
    pub max_role: i64,
}

/// 授权类型
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSql, FromSql)]
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

/// 角色
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role")]
pub struct Role {
    pub id: Id,
    pub name: String,
    max_user: i64,
    max_permission: i64,
}

/// 用户角色
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "user_role")]
pub struct UserRole {
    pub user_id: Id,
    pub role_id: Id,
}

/// 角色继承关系
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role_ext")]
pub struct RoleExt {
    pub base_id: Id,
    pub derived_id: Id,
}

/// 约束类型
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSql, FromSql)]
pub enum ConstraintType {
    Mutex,
    BaseRequired,
}

/// 角色约束
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role_constraint")]
pub struct RoleConstraint {
    pub id: Id,
    pub constraint_name: String,
    pub constraint_type: ConstraintType,
}

/// 互斥约束
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "constraint_mutex")]
pub struct ConstraintMutex {
    pub constraint_id: Id,
    pub role_id: Id,
}

/// 先决条件约束
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "constraint_base_required")]
pub struct ConstraintBaseRequired {
    pub constraint_id: Id,
    pub role_id: Id,
}

/// 权限
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "permission")]
pub struct Permission {
    pub id: Id,
    pub permission_name: String,
}

/// 角色权限
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role_permission")]
pub struct RolePermission {
    pub role_id: Id,
    pub permission_id: Id,
}

/// -----------------------------------------------------------------

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
pub struct GetAuthCodeParams {
    pub phone: Phone,
}
