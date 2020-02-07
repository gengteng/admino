use super::*;

/// 角色
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role")]
pub struct Role {
    pub id: Id,
    pub name: String,
    max_user: i64,
    max_permission: i64,
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
